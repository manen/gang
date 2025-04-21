// this module contains all the code for cross-process swarm coordination

pub mod client;
pub mod head;

pub use client::Tasks;
pub use head::TasksHead;

use super::Task;

pub const ADDR: &str = "127.0.0.1:8789";

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub enum ServerboundPacket {
	/// says hi to the server. completely optional and expect no return packet
	Hello { inst_id: i32 },
	/// requests the next task for this instance \
	/// server will return ClientboundPacket::AssignTask
	RequestTask { inst_id: i32 },
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub enum ClientboundPacket {
	AssignTask(Option<Task>),
}
