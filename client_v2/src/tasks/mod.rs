pub mod net;
pub mod task;
pub use task::Task;

use anyhow::anyhow;
use net::{Tasks, TasksHead};

pub trait TasksTrait {
	async fn next(&mut self) -> anyhow::Result<Task>;

	async fn tick(&self, bot: &azalea::Client);
}
