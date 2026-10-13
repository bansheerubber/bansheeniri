use ::image::{imageops, ImageBuffer, Rgba, RgbaImage};
use anyhow::{bail, Result};
use niri_ipc::{
	socket::Socket,
	state::{EventStreamState, EventStreamStatePart},
	Action, Event, Output, Request, Response, SizeChange, Window, WindowLayout, Workspace,
};
use std::{
	collections::{BTreeMap, HashMap, HashSet},
	process::Command,
	sync::{Arc, Mutex},
	thread::{self, sleep},
	time::Duration,
};

use crate::{
	image,
	waybar::{is_waybar_running_for_output, kill_waybar, run_waybar_for_output},
};

pub struct State {
	windows_waiting_for_resize: HashMap<u64, (i32, i32)>,

	map_image: Arc<Mutex<HashMap<String, ImageBuffer<Rgba<u8>, Vec<u8>>>>>,
	map_image_token: Arc<Mutex<HashMap<String, usize>>>,
	outputs: HashMap<String, Output>,
	socket: Socket,
	stream_state: EventStreamState,
}

impl State {
	pub fn new() -> Result<Self> {
		let socket = Socket::connect()?;

		Ok(Self {
			windows_waiting_for_resize: HashMap::new(),

			map_image: Arc::new(Mutex::new(HashMap::new())),
			map_image_token: Arc::new(Mutex::new(HashMap::new())),
			outputs: HashMap::new(),
			socket,
			stream_state: EventStreamState::default(),
		})
	}

	pub fn handle_state_update(&mut self, event: Event) {
		let mut updated_workspaces = HashSet::new();
		match &event {
			Event::WindowsChanged { windows } => {
				for window in windows.iter() {
					self.handle_window_layout_change(window.id, &window.layout);
				}
			}
			Event::WindowClosed { id } => {
				if let Some(window) = self.stream_state.windows.windows.get(id) {
					updated_workspaces.insert(window.workspace_id);
				}
			}
			Event::WindowOpenedOrChanged { window } => {
				self.handle_window_layout_change(window.id, &window.layout);

				if let Some(old_window) = self.stream_state.windows.windows.get(&window.id) {
					if window.workspace_id.is_some()
						&& old_window.workspace_id.is_some()
						&& window.workspace_id != old_window.workspace_id
					{
						updated_workspaces.insert(old_window.workspace_id);
						updated_workspaces.insert(window.workspace_id);
					}
				} else {
					updated_workspaces.insert(window.workspace_id);
				}
			}
			Event::WindowLayoutsChanged { changes } => {
				for (window_id, layout) in changes.iter() {
					self.handle_window_layout_change(*window_id, layout);
				}
			}
			_ => {}
		}

		self.stream_state.apply(event);

		// check if outputs have changed
		for workspace in self.stream_state.workspaces.workspaces.values() {
			if let Some(output) = workspace.output.as_ref()
				&& self.outputs.get(output).is_none()
			{
				match self.update_outputs() {
					Ok(opened_waybar) => {
						// give waybar time to settle or else it fails to open map image
						if opened_waybar {
							sleep(Duration::from_secs(1));
						}
					}
					Err(error) => println!("{error:?}"),
				}

				break;
			}
		}

		let mut windows_to_maximize = Vec::new();

		for workspace in self.stream_state.workspaces.workspaces.values() {
			if !workspace.is_active && !updated_workspaces.contains(&Some(workspace.id)) {
				continue;
			}

			let Some(output) = self
				.outputs
				.get(workspace.output.as_ref().unwrap_or(&String::new()))
			else {
				continue;
			};

			let Some(logical_output) = output.logical else {
				continue;
			};

			let windows = self.get_windows_for_workspace(workspace.id);
			if windows.len() == 1
				&& let Some(window) = windows.get(0)
				&& (updated_workspaces.contains(&Some(workspace.id))
					|| (window.layout.window_size.0 > (logical_output.width as f64 * 0.9) as i32
						&& window.layout.window_size.1
							> (logical_output.height as f64 * 0.8) as i32))
			{
				windows_to_maximize.push((window.id, true));
			} else if windows.len() > 1 {
				for window in windows.iter() {
					windows_to_maximize.push((window.id, false));
				}
			}
		}

		for (window, is_maximized_to_edges) in windows_to_maximize {
			self.set_window_maximized_to_edges(window, is_maximized_to_edges);
		}

		for workspace in self
			.stream_state
			.workspaces
			.workspaces
			.values()
			.filter(|workspace| workspace.is_active)
		{
			let Some(output) = self
				.outputs
				.get(workspace.output.as_ref().unwrap_or(&String::new()))
			else {
				continue;
			};

			self.draw_map_for_output(output, workspace);
		}
	}

	fn set_window_maximized_to_edges(&mut self, window_id: u64, is_maximized_to_edges: bool) {
		let Some(window) = self.stream_state.windows.windows.get(&window_id) else {
			return;
		};

		if let Some(is_maximized) = self.is_window_maximized_to_edges(window)
			&& is_maximized != is_maximized_to_edges
		{
			if is_maximized_to_edges {
				if let Err(error) = self.socket.send(Request::Action(Action::SetWindowWidth {
					id: Some(window.id),
					change: SizeChange::SetProportion(50.0),
				})) {
					println!("Error: {error}");
				}
			}

			if let Err(error) = self
				.socket
				.send(Request::Action(Action::MaximizeWindowToEdges {
					id: Some(window.id),
				})) {
				println!("Error: {error}");
			}

			self.windows_waiting_for_resize
				.insert(window_id, window.layout.window_size);
		}
	}

	fn get_windows_for_workspace(&self, workspace_id: u64) -> Vec<&Window> {
		self.stream_state
			.windows
			.windows
			.values()
			.filter(|window| {
				if let Some(window_workspace_id) = window.workspace_id
					&& window_workspace_id == workspace_id
				{
					true && !window.is_floating
				} else {
					false
				}
			})
			.collect::<Vec<_>>()
	}

	fn handle_window_layout_change(&mut self, window_id: u64, layout: &WindowLayout) {
		let Some(saved_size) = self.windows_waiting_for_resize.get(&window_id) else {
			return;
		};

		if layout.window_size != *saved_size {
			self.windows_waiting_for_resize.remove(&window_id);
		}
	}

	fn is_window_maximized_to_edges(&self, window: &Window) -> Option<bool> {
		if self.windows_waiting_for_resize.contains_key(&window.id) {
			return None;
		}

		let workspace = self
			.stream_state
			.workspaces
			.workspaces
			.get(&window.workspace_id?)?;

		let logical_output = self.outputs.get(workspace.output.as_ref()?)?.logical?;

		Some(window.layout.window_size.0 as u32 == logical_output.width)
	}

	fn draw_map_for_output(&self, output: &Output, workspace: &Workspace) {
		let Some(logical_output) = output.logical else {
			return;
		};

		let mut columns_to_rect = BTreeMap::new();
		let windows = self.get_windows_for_workspace(workspace.id);

		let image_height = std::env::var("BANSHEENIRI_HEIGHT")
			.unwrap_or("18".into())
			.parse::<i32>()
			.unwrap();

		let x_padding = 3;

		let mut image = RgbaImage::new(600, image_height as u32);

		for window in windows {
			let Some((column_id, _)) = window.layout.pos_in_scrolling_layout else {
				continue;
			};

			let rect = columns_to_rect.entry(column_id).or_insert(image::Rect {
				urgent: false,
				focused: false,
				width: 0,
				height: image_height,
				x: 0,
				y: 0,
			});

			rect.width = rect.width.max(window.layout.tile_size.0.ceil() as i32);

			if window.is_urgent {
				rect.urgent = true;
			}

			if window.is_focused {
				rect.focused = true;
			}
		}

		let scale = image_height as f64 / logical_output.height as f64;

		let mut x = 0;
		for mut rect in columns_to_rect.into_values() {
			rect.x = x;
			rect.width = ((rect.width as f64 * scale).floor() as i32).max(4);
			image::draw_rectangle(&mut image, &rect);

			x += rect.width + x_padding;
		}

		let mut map_images = self.map_image.lock().expect("Could not acquire lock");
		map_images.insert(
			output.name.clone(),
			imageops::crop(
				&mut image,
				0,
				0,
				(x as u32).saturating_sub(x_padding as u32).max(1),
				image_height as u32,
			)
			.to_image(),
		);

		let mut tokens = self.map_image_token.lock().expect("Could not acquire lock");
		let token = tokens.entry(output.name.clone()).or_default();

		*token += 1;
		self.debounce_save(*token, &output.name);
	}

	fn debounce_save(&self, token: usize, output: &str) {
		let output = output.to_string();
		let map_image = self.map_image.clone();
		let map_image_token = self.map_image_token.clone();

		thread::spawn(move || {
			thread::sleep(Duration::from_millis(16));

			// quit out of thread if thread's token doesn't match the current token
			let tokens = map_image_token.lock().expect("Could not acquire lock");
			if tokens.get(&output) != Some(&token) {
				return;
			}

			drop(tokens);

			let map_images = map_image.lock().expect("Could not acquire lock");
			let Some(map_image) = map_images.get(&output) else {
				return;
			};

			let home = std::env::var("HOME").unwrap_or("/home/me".into());

			std::fs::create_dir_all(format!("{home}/.local/share/bansheeniri/"));

			map_image
				.save(format!(
					"{home}/.local/share/bansheeniri/map.{}.png",
					output
				))
				.expect("Could not save map");

			if let Ok(Some(waybar_pid)) = is_waybar_running_for_output(&output) {
				let _ = Command::new("kill")
					.arg("-s")
					.arg("35")
					.arg(waybar_pid)
					.output();
			}
		});
	}

	fn update_outputs(&mut self) -> Result<bool> {
		let Ok(response) = self.socket.send(Request::Outputs)? else {
			bail!("Could not get outputs")
		};

		let mut old_outputs = self
			.outputs
			.keys()
			.map(String::clone)
			.collect::<Vec<_>>()
			.clone();

		self.outputs = match response {
			Response::Outputs(outputs) => {
				outputs
					.into_values()
					.fold(HashMap::new(), |mut map, output| {
						map.insert(output.name.clone(), output);
						map
					})
			}
			_ => bail!("Invalid response for outputs request"),
		};

		/*let mut opened_waybar = false;

		for new_output in self.outputs.keys() {
			if let Some(index) = old_outputs.iter().position(|o| o == new_output) {
				old_outputs.remove(index);
			}

			if is_waybar_running_for_output(&new_output)?.is_none() {
				run_waybar_for_output(&new_output)?;
				opened_waybar = true;
			}
		}

		for old_output in old_outputs {
			if let Some(pid) = is_waybar_running_for_output(&old_output)? {
				kill_waybar(&pid)?;
			}
		}*/

		Ok(true)
	}
}
