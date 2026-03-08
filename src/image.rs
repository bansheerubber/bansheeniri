use image::{ImageBuffer, Rgba};

pub struct Rect {
	pub focused: bool,
	pub width: i32,
	pub height: i32,
	pub x: i32,
	pub y: i32,
}

fn safe_put_pixel(image: &mut ImageBuffer<Rgba<u8>, Vec<u8>>, x: i32, y: i32, color: Rgba<u8>) {
	let image_width = image.width() as i32;
	let image_height = image.height() as i32;

	if x >= image_width || y >= image_height || x < 0 || y < 0 {
		return;
	}

	image.put_pixel(x as u32, y as u32, color);
}

pub fn draw_rectangle(image: &mut ImageBuffer<Rgba<u8>, Vec<u8>>, rect: &Rect, y_offset: i32) {
	for x in 0..rect.width {
		for y in 0..rect.height {
			if rect.focused {
				safe_put_pixel(
					image,
					x + rect.x,
					y + rect.y + y_offset,
					Rgba([0x7f, 0x56, 0x8f, 255]),
				);
			} else {
				safe_put_pixel(
					image,
					x + rect.x,
					y + rect.y + y_offset,
					Rgba([0x4A, 0x34, 0x53, 255]),
				);
			}
		}
	}
}
