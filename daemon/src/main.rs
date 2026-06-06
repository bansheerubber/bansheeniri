use anyhow::Result;
use niri_ipc::{socket::Socket, Request};
use std::process::exit;

mod image;
mod state;
mod waybar;

fn main() -> Result<()> {
	let mut stream_socket = Socket::connect()?;
	let mut state = state::State::new()?;

	if let Err(error) = stream_socket.send(Request::EventStream)? {
		println!("Error: {error}");
		exit(5);
	}

	let mut read_event = stream_socket.read_events();
	loop {
		match read_event() {
			Ok(event) => {
				state.handle_state_update(event);
			}
			Err(error) => {
				Err(error)?;
				break;
			},
		}
	}

	Ok(())
}
