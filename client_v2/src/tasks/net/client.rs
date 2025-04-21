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
}
impl Tasks {
	pub async fn new(inst_id: i32) -> anyhow::Result<Self> {
		let mut stream = TcpStream::connect(super::ADDR).await?;
		println!("client connected to {}", super::ADDR);

		let hello = ServerboundPacket::Hello { inst_id };
		stream.write_as_packet(hello).await?;

		Ok(Self { inst_id, stream })
	}

	pub async fn next(&mut self) -> anyhow::Result<crate::tasks::Task> {
		let request = ServerboundPacket::RequestTask {
			inst_id: self.inst_id,
		};
		self.stream.write_as_packet(request).await?;

		let response: ClientboundPacket = self.stream.read_as_packet().await?;
		match response {
			ClientboundPacket::AssignTask(task) => task.ok_or_else(|| anyhow!("task is None")),
		}
	}

	pub async fn tick(&self, bot: &azalea::Client) {}
}
