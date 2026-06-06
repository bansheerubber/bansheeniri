#[derive(Clone)]
pub enum Message {
	MoveLeft(String),
	MoveRight(String),
	Noop,
	Rotate(String),
	SelectMonitor(String),
}
