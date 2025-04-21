use anyhow::anyhow;
use azalea::Client;
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

	pub async fn next(&mut self, bot: &Client) -> anyhow::Result<crate::tasks::Task> {
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
				_ => self.handle_other(response, bot).await?,
			}
		}
	}
	pub async fn handle_other(
		&mut self,
		packet: ClientboundPacket,
		bot: &Client,
	) -> anyhow::Result<()> {
		match packet {
			ClientboundPacket::Find { username } => {
				use azalea::{
					GameProfileComponent,
					entity::{Position, metadata::Player},
				};
				use bevy_ecs::prelude::With;

				let entity = bot.entity_by::<With<Player>, &GameProfileComponent>(
					|profile: &&GameProfileComponent| profile.name == username,
				);
				let report = if let Some(player) = entity {
					let pos: Option<Position> = bot.get_entity_component(player);
					if let Some(pos) = pos {
						crate::tasks::net::PosReport::Found(pos.down(0.0))
					} else {
						crate::tasks::net::PosReport::NotHere
					}
				} else {
					crate::tasks::net::PosReport::NotHere
				};
				let report = ServerboundPacket::ReportPosition { username, report };
				self.stream.write_as_packet(report).await?;
			}
			ClientboundPacket::AssignTask(_) => {}
		}
		Ok(())
	}

	pub async fn tick(&mut self, bot: &azalea::Client) -> anyhow::Result<()> {
		Ok(())
	}
}
