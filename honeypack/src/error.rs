#[derive(Debug, thiserror::Error)]
pub enum Error {
	#[error("honeypack io error: {0}")]
	IO(#[from] std::io::Error),
	#[error("honeypack bincode error: {0}")]
	Bincode(#[from] bincode::Error),
}

pub type Result<T, E = Error> = std::result::Result<T, E>;
