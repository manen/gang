// this module contains all the code for cross-process swarm coordination

pub mod client;
pub mod server;

use std::borrow::Cow;

use azalea::{Vec3, chat::ChatPacket, core::math::lcm};
pub use client::Tasks;
pub use server::start_server;
use uuid::Uuid;

use super::Task;

pub const ADDR: &str = "127.0.0.1:8789";

// protocol looks something like this
// 1. client - hello -> server
// 2. server - hello, your name is x and your id is y -> client
//
// from then on:
// ServerboundPacket & ClientboundPacket

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct ServerboundHelloPacket {
	/// not used for anything but something shits itself if we send empty packets
	lucky_number: i32,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct ClientboundHelloPacket {
	name: String,
	inst_id: i32,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub enum ServerboundPacket {
	ChatMessage {
		/// see hash_chat function
		hash: u64,
		sender: Option<String>,
		content: String,
	},
	/// signals others to attack the entity with the given uuid
	Agro { uuid: Uuid },

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

pub fn hash_chat(m: &ChatPacket) -> u64 {
	use std::hash::{DefaultHasher, Hash, Hasher};
	use std::time::{SystemTime, UNIX_EPOCH};

	let seconds = SystemTime::now()
		.duration_since(UNIX_EPOCH)
		.expect("time went backwards")
		.as_secs();

	let mut hasher = DefaultHasher::new();

	seconds.hash(&mut hasher);
	m.content().hash(&mut hasher);
	m.sender().hash(&mut hasher);

	hasher.finish()
}
