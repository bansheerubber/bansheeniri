use niri_ipc::Output;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct Config {
	#[serde(skip)]
	pub file_name: String,
	pub outputs: Vec<MonitorConfig>,
}

impl Config {
	pub fn save(&self) {
		let Ok(toml) = toml::to_string_pretty(&self) else {
			return;
		};

		std::fs::write(&self.file_name, toml).expect("Could not write file");
	}

	pub fn matches_outputs(&self, outputs: &Vec<Output>) -> bool {
		outputs.iter().fold(0, |total, output| {
			if self
				.outputs
				.iter()
				.position(|o| o.name == output.name)
				.is_some()
			{
				println!("contains {}", &output.name);
				total + 1
			} else {
				total
			}
		}) == outputs.len()
	}
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct MonitorConfig {
	pub name: String,
	pub width: u16,
	pub height: u16,
	pub rate: u32,
	pub rotation: Option<u32>,
}
