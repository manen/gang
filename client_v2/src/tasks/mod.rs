pub mod head;
pub mod net;
pub mod task;
pub use task::Task;

#[derive(Clone, Debug)]
pub enum Tasks {
	Head(head::Tasks),
}
impl Tasks {
	pub async fn create_or_connect(default_owner: &'static str) -> anyhow::Result<Self> {
		Ok(Self::Head(head::Tasks::with_owner(default_owner)))
	}
}
