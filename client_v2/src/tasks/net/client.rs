use anyhow::anyhow;
use honeypack::{PacketRead, PacketWrite};
use tokio::net::TcpStream;

use crate::tasks::net::ServerboundPacket;

use super::ClientboundPacket;

#[derive(Debug)]
/// a client for communicating with a TasksHead
pub struct Tasks {
	inst_id: i32,
	stream: TcpStream,
	owner: Option<String>,
}
impl Tasks {
	pub async fn new(inst_id: i32) -> anyhow::Result<Self> {
		let mut stream = TcpStream::connect(super::ADDR).await?;
		println!("client connected to {}", super::ADDR);

		let hello = ServerboundPacket::Hello { inst_id };
		stream.write_as_packet(hello).await?;

		Ok(Self {
			inst_id,
			stream,
			owner: None,
		})
	}

	pub async fn next(&mut self) -> anyhow::Result<crate::tasks::Task> {
		let request = ServerboundPacket::RequestTask {
			inst_id: self.inst_id,
		};
		self.stream.write_as_packet(request).await?;

		loop {
			let response: ClientboundPacket = self.stream.read_as_packet().await?;
			match response {
				ClientboundPacket::AssignTask(task) => {
					break task.ok_or_else(|| anyhow!("task is None"));
				}
				_ => self.handle_other(response),
			}
		}
	}
	pub fn handle_other(&mut self, packet: ClientboundPacket) {
		match packet {
			ClientboundPacket::OwnerIs { username } => {
				self.owner = Some(username);
			}
			ClientboundPacket::AssignTask(_) => {}
		}
	}

	pub async fn tick(&self, bot: &azalea::Client) {}
}
