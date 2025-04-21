pub mod net;
pub mod task;
pub use task::Task;

use net::{TasksClient, TasksHead};

#[derive(Clone, Debug)]
pub enum Tasks {
	Head(TasksHead),
	Client(TasksClient),
}
impl Tasks {
	pub async fn create_or_connect(default_owner: &'static str) -> anyhow::Result<Self> {
		todo!()
	}
}

pub trait TasksInterface {
	async fn next(&mut self) -> Option<Task>;
}
