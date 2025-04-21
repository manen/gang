// this module contains all the code for cross-process swarm coordination

pub mod client;
pub mod head;

pub use client::TasksClient;
pub use head::TasksHead;

pub const ADDR: &str = "127.0.0.1:8789";

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub enum ServerboundPacket {}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub enum ClientboundPacket {}
