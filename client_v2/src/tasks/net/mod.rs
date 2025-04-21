// this module contains all the code for cross-process swarm coordination

pub mod client;
pub mod server;

use azalea::Vec3;
pub use client::Tasks;
pub use server::start_server;

use super::Task;

pub const ADDR: &str = "127.0.0.1:8789";

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub enum ServerboundPacket {
	/// says hi to the server, clients should send this as soon as the socket opens
	Hello { inst_id: i32 },
	/// requests the next task for this instance \
	/// server will return ClientboundPacket::AssignTask
	RequestTask { inst_id: i32 },

	/// reports the owner's position to everyone else
	ReportPosition { username: String, report: PosReport },
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub enum PosReport {
	NotHere,
	Found(Vec3),
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub enum ClientboundPacket {
	/// client responds with ServerboundPacket::ReportPosition
	Find {
		username: String,
	},
	AssignTask(Option<Task>),
}
