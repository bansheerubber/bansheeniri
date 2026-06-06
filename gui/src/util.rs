use niri_ipc::{Mode, Output};

pub fn get_current_mode(output: &Output) -> Option<Mode> {
	for mode in output.modes.iter() {
		if mode.is_preferred {
			return Some(mode.clone());
		}
	}

	return None;
}
