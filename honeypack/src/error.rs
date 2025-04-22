#[derive(Debug, thiserror::Error)]
pub enum Error {
	#[error("honeypack io error: {0}")]
	IO(#[from] std::io::Error),
	#[error("honeypack bincode error: {0}")]
	Bincode(#[from] bincode::Error),
	#[error("{ctx}\n{err}")]
	WithContext { ctx: String, err: Box<Self> },
}
impl Error {
	pub fn with_context(self, ctx: impl Into<String>) -> Self {
		Self::WithContext {
			ctx: ctx.into(),
			err: Box::new(self),
		}
	}
}

pub type Result<T, E = Error> = std::result::Result<T, E>;
