use iced::{
	alignment::{Horizontal, Vertical},
	widget::{
		button, column, container, row,
		space::{horizontal, vertical},
		text, Space, Text,
	},
	Element, Length, Theme,
};
use niri_ipc::Output;

use iced_aw::menu::{self, Item, Menu};
use iced_aw::style::{menu_bar::primary, Status};
use iced_aw::{iced_aw_font, menu_bar, menu_items, ICED_AW_FONT_BYTES};

use crate::{config::MonitorConfig, message::Message, theme, util::get_current_mode, ICON_FONT};

pub struct Monitor {
	pub config: MonitorConfig,
	pub output: Output,
}

impl Monitor {
	pub fn new(config: MonitorConfig, output: Output) -> Self {
		Self { config, output }
	}

	pub fn view(
		&self,
		selected: bool,
		can_move_left: bool,
		can_move_right: bool,
	) -> Element<'_, Message> {
		const SCALE: f32 = 0.4;

		let mode = get_current_mode(&self.output).unwrap();
		let (physical_width, _) = self.output.physical_size.unwrap();
		let mm_per_pixel = physical_width as f32 / mode.width as f32;

		let rotation = if let Some(rotation) = self.config.rotation {
			rotation
		} else {
			0
		};

		let (width, height) = if rotation == 0 || rotation == 180 {
			(
				mode.width as f32 * mm_per_pixel * SCALE,
				mode.height as f32 * mm_per_pixel * SCALE,
			)
		} else {
			(
				mode.height as f32 * mm_per_pixel * SCALE,
				mode.width as f32 * mm_per_pixel * SCALE,
			)
		};

		let arrow_icon = match rotation {
			0 => "\u{e316}".to_string(),
			90 => "\u{e315}".to_string(),
			180 => "\u{e313}".to_string(),
			270 => "\u{e314}".to_string(),
			_ => unreachable!(),
		};

		button(
			container(column![
				row![
					menu_bar!((
						button(text!("\u{e5d4}").font(ICON_FONT).size(20))
							.width(20)
							.padding(0)
							.style(|theme: &Theme, _style| theme::invisible_button(theme)),
						{
							Menu::new(menu_items!((
								button(text!("Modes").size(14).align_x(Horizontal::Left))
									.width(Length::Fill)
									.padding(0)
									.style(|theme: &Theme, _style| theme::invisible_button(theme)),
								Menu::new(menu_items!((button(text!("hey2").size(20))))).width(50)
							)))
							.width(100)
						}
					))
					.safe_bounds_margin(500.0)
					.close_on_background_click(true)
					.close_on_background_click_global(true),
					horizontal(),
					Text::new(arrow_icon).font(ICON_FONT).size(20),
					button(text!("\u{e5d5}").font(ICON_FONT).size(20))
						.width(20)
						.padding(0)
						.style(|theme: &Theme, _style| theme::invisible_button(theme))
						.on_press(Message::Rotate(self.output.name.clone())),
				]
				.align_y(Vertical::Top)
				.height(20),
				vertical(),
				row![
					if can_move_left {
						Element::<'_, Message>::from(
							button(text!("\u{e5cb}").font(ICON_FONT).size(20))
								.width(20)
								.padding(0)
								.style(|theme: &Theme, _style| theme::invisible_button(theme))
								.on_press(Message::MoveLeft(self.output.name.clone())),
						)
					} else {
						Element::<'_, Message>::from(Space::new().width(20))
					},
					Text::new(&self.output.name)
						.width(Length::Fill)
						.size(12)
						.align_x(Horizontal::Center),
					if can_move_right {
						Element::<'_, Message>::from(
							button(text!("\u{e5cc}").font(ICON_FONT).size(20))
								.width(20)
								.padding(0)
								.style(|theme: &Theme, _style| theme::invisible_button(theme))
								.on_press(Message::MoveRight(self.output.name.clone())),
						)
					} else {
						Element::<'_, Message>::from(Space::new().width(20))
					},
				]
				.align_y(Vertical::Center),
				vertical(),
				row![].height(20),
			])
			.align_y(Vertical::Center)
			.width(width)
			.height(height)
			.style(move |theme: &Theme| {
				if selected {
					theme::selected_monitor(theme)
				} else {
					theme::monitor(theme)
				}
			}),
		)
		.padding(0)
		.style(|theme: &Theme, _style| theme::invisible_button(theme))
		.on_press(Message::SelectMonitor(self.output.name.clone()))
		.into()
	}
}
