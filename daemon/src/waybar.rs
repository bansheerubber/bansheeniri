use std::{io::BufRead, process::Command};

use anyhow::Result;

pub fn run_waybar_for_output(output: &str) -> Result<()> {
	let config = std::fs::read_to_string("/home/me/.config/waybar/config.base.jsonc")?;
	let config = config.replace("%output%", output);

	std::fs::write(
		format!("/home/me/.config/waybar/config.{output}.jsonc"),
		config,
	)?;

	Command::new("waybar")
		.args(&[
			"-c",
			&format!("/home/me/.config/waybar/config.{output}.jsonc"),
		])
		.spawn()?;

	Ok(())
}

pub fn is_waybar_running_for_output(output: &str) -> Result<Option<String>> {
	let pgrep = Command::new("pgrep").args(&["-ia", "waybar"]).output()?;

	let mut lines = pgrep.stdout.lines();
	while let Some(Ok(line)) = lines.next() {
		if line.contains(&format!("config.{output}.jsonc")) {
			return Ok(Some(line.split(" ").nth(0).unwrap().to_string()));
		}
	}

	Ok(None)
}

pub fn kill_waybar(pid: &str) -> Result<()> {
	Command::new("kill").args(&["-9", pid]).spawn()?;
	Ok(())
}
