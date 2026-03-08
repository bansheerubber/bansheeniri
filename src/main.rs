use std::process::exit;

use anyhow::Result;
use niri_ipc::{socket::Socket, Request};

mod image;
mod state;

fn main() -> Result<()> {
	let mut stream_socket = Socket::connect()?;
	let mut state = state::State::new()?;

	if let Err(error) = stream_socket.send(Request::EventStream)? {
		println!("Error: {error}");
		exit(5);
	}

	let mut read_event = stream_socket.read_events();
	while let Ok(event) = read_event() {
		state.handle_state_update(event);
	}

	Ok(())
}
