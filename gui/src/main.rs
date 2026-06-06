use iced::{
	font::{Family, Stretch, Style, Weight},
	Font,
};

mod config;
mod message;
mod monitor;
mod theme;
mod util;
mod window;

pub const ICON_FONT: Font = Font {
	family: Family::Name("Material Icons"),
	weight: Weight::Normal,
	stretch: Stretch::Normal,
	style: Style::Normal,
};

fn main() {
	iced::application(
		window::Window::new,
		window::Window::update,
		window::Window::view,
	)
	.font(include_bytes!("../../fonts/NotoSans-Medium.ttf"))
	.font(include_bytes!("../../fonts/NotoSans-Bold.ttf"))
	.font(include_bytes!("../../fonts/NotoSans-Regular.ttf"))
	.font(include_bytes!("../../fonts/MaterialIcons-Regular.ttf"))
	.default_font(Font {
		family: Family::Name("Noto Sans"),
		weight: Weight::Medium,
		stretch: Stretch::Normal,
		style: Style::Normal,
	})
	.window_size((800.0, 400.0))
	.scale_factor(|_| 1.0)
	.decorations(false)
	.resizable(false)
	.antialiasing(true)
	.theme(window::Window::theme)
	.title("bansheeniri")
	.run()
	.expect("Could not open bansheeniri");
}
