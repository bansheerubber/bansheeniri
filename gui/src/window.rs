use std::borrow::Cow;

use iced::{
	alignment::{Horizontal, Vertical},
	color,
	theme::{
		palette::{Background, Danger, Extended, Pair, Primary, Secondary, Success, Warning},
		Palette,
	},
	widget::{container, Row},
	Element, Length, Task, Theme,
};
use niri_ipc::{socket::Socket, Output, Request, Response};

use crate::{
	config::{Config, MonitorConfig},
	message::Message,
	monitor::Monitor,
	theme,
	util::get_current_mode,
};

pub struct Window {
	current_config: usize,
	configs: Vec<Config>,
	monitors: Vec<Monitor>,
	outputs: Vec<Output>,
	selected_monitor: Option<String>,
}

impl Window {
	pub fn new() -> (Self, Task<Message>) {
		let mut socket = Socket::connect().expect("Could not connect to niri");

		let response = socket
			.send(Request::Outputs)
			.expect("Could not send outputs request")
			.expect("Could not get outputs");

		let outputs = match response {
			Response::Outputs(outputs) => outputs.into_values().collect::<Vec<_>>(),
			_ => panic!("Invalid response for outputs request"),
		};

		let mut current_config = None;
		let mut configs = Vec::new();
		for file in std::fs::read_dir("/home/me/.config/bansheeniri").expect("Could not read dir") {
			let Ok(file) = file else {
				println!("File entry error");
				continue;
			};

			let Ok(contents) = std::fs::read_to_string(file.path()) else {
				println!("Failed to read file '{}'", file.path().to_string_lossy());
				continue;
			};

			let Ok(mut config) = toml::from_str::<Config>(&contents) else {
				println!("Failed to parse '{}'", file.path().to_string_lossy());
				continue;
			};

			config.file_name = file.path().to_string_lossy().into();

			if config.matches_outputs(&outputs) {
				current_config = Some(configs.len());
			}

			configs.push(config);
		}

		let current_config = if let Some(current_config) = current_config {
			current_config
		} else {
			let config = Config {
				file_name: "/home/me/.config/bansheeniri/the-config.toml".to_string(),
				outputs: outputs
					.iter()
					.map(|output| {
						let mode = get_current_mode(output).unwrap();

						return MonitorConfig {
							name: output.name.clone(),
							width: mode.width,
							height: mode.height,
							rate: mode.refresh_rate,
							rotation: None,
						};
					})
					.collect(),
			};

			config.save();

			configs.push(config);
			configs.len() - 1
		};

		let mut monitors = Vec::new();
		for monitor_config in configs.get(current_config).unwrap().outputs.iter() {
			monitors.push(Monitor::new(
				monitor_config.clone(),
				outputs
					.iter()
					.find(|output| output.name == monitor_config.name)
					.unwrap()
					.clone(),
			));
		}

		(
			Self {
				configs,
				current_config,
				monitors,
				outputs,
				selected_monitor: None,
			},
			Task::none(),
		)
	}

	fn get_current_config(&mut self) -> &mut Config {
		self.configs.get_mut(self.current_config).unwrap()
	}

	fn update_from_config(&mut self) {
		let config = self.configs.get_mut(self.current_config).unwrap();

		let mut monitors = Vec::new();
		for monitor_config in config.outputs.iter() {
			monitors.push(Monitor::new(
				monitor_config.clone(),
				self.outputs
					.iter()
					.find(|output| output.name == monitor_config.name)
					.unwrap()
					.clone(),
			));
		}

		self.monitors = monitors;
	}

	pub fn update(&mut self, message: Message) -> Task<Message> {
		match message {
			Message::MoveLeft(output) => {
				let config = self.get_current_config();
				let index = config
					.outputs
					.iter()
					.position(|monitor_config| monitor_config.name == output)
					.unwrap();

				let new_index = index.saturating_sub(1);
				let monitor_config = config.outputs.remove(index);
				config.outputs.insert(new_index, monitor_config);
				config.save();

				self.update_from_config();
			}
			Message::MoveRight(output) => {
				let config = self.get_current_config();
				let index = config
					.outputs
					.iter()
					.position(|monitor_config| monitor_config.name == output)
					.unwrap();

				let new_index = (index + 1).min(config.outputs.len() - 1);
				let monitor_config = config.outputs.remove(index);
				config.outputs.insert(new_index, monitor_config);
				config.save();

				self.update_from_config();
			}
			Message::Noop => {}
			Message::Rotate(output) => {
				let config = self.get_current_config();
				let monitor_config = config
					.outputs
					.iter_mut()
					.find(|monitor_config| monitor_config.name == output)
					.unwrap();

				let rotation = if let Some(rotation) = monitor_config.rotation {
					rotation
				} else {
					0
				};

				let new_rotation = rotation + 90;
				if new_rotation == 360 {
					monitor_config.rotation = None;
				} else {
					monitor_config.rotation = Some(new_rotation);
				}

				config.save();
				self.update_from_config();
			}
			Message::SelectMonitor(monitor) => {
				if let Some(selected_monitor) = self.selected_monitor.as_ref()
					&& selected_monitor == &monitor
				{
					self.selected_monitor = None;
				} else {
					self.selected_monitor = Some(monitor);
				}
			}
		}

		Task::none()
	}

	pub fn view(&self) -> Element<'_, Message> {
		let mut row = Row::new().align_y(Vertical::Top).spacing(5);
		for i in 0..self.monitors.len() {
			let monitor = &self.monitors[i];

			row = row.push(monitor.view(
				self.selected_monitor.as_ref() == Some(&monitor.output.name),
				i != 0,
				i != self.monitors.len() - 1,
			));
		}

		container(row)
			.align_x(Horizontal::Center)
			.width(Length::Fill)
			.height(Length::Fill)
			.style(|theme: &Theme| theme::window(theme))
			.into()
	}

	pub fn theme(&self) -> Theme {
		let name: &str = "bansheetheme";

		Theme::custom_with_fn(
			Cow::Borrowed(name),
			Palette {
				background: color!(0x38263F),
				text: color!(0xE2C5D8),
				primary: color!(0xE059E0),
				success: color!(0x2CBA60),
				danger: color!(0xDD4460),
				warning: color!(0xFF644F),
			},
			|palette| Extended {
				background: Background {
					base: Pair::new(palette.background, palette.text),
					weakest: Pair {
						color: color!(0x7F568F),
						text: color!(0xFFB3FF),
					},
					weaker: Pair {
						color: color!(0x6A4A77),
						text: color!(0xFFB3FF),
					},
					weak: Pair {
						color: color!(0x583C63),
						text: color!(0xFFB3FF),
					},
					neutral: Pair::new(palette.background, palette.text),
					strong: Pair::new(color!(0x322238), palette.text),
					stronger: Pair::new(color!(0x261A2B), palette.text),
					strongest: Pair::new(color!(0x1E121E), palette.text),
				},
				primary: Primary {
					strong: Pair::new(color!(0xFF66FF), palette.text),
					base: Pair::new(palette.primary, palette.text),
					weak: Pair::new(color!(0x8A76E0), palette.text),
				},
				secondary: Secondary {
					strong: Pair::new(color!(0x58B7CE), palette.text),
					base: Pair::new(color!(0xDBCD51), palette.text),
					weak: Pair::new(color!(0xD796F2), palette.text),
				},
				success: Success {
					strong: Pair::new(palette.success, palette.text),
					base: Pair::new(palette.success, palette.text),
					weak: Pair::new(palette.success, palette.text),
				},
				danger: Danger {
					strong: Pair::new(palette.danger, palette.text),
					base: Pair::new(palette.danger, palette.text),
					weak: Pair::new(palette.danger, palette.text),
				},
				warning: Warning {
					strong: Pair::new(palette.warning, palette.text),
					base: Pair::new(palette.warning, palette.text),
					weak: Pair::new(palette.warning, palette.text),
				},
				is_dark: true,
			},
		)
	}
}
