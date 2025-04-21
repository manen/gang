use tokio::net::TcpListener;

use crate::tasks::{
	Task,
	net::{ClientboundPacket, ServerboundPacket},
};

use honeypack::{PacketRead, PacketWrite};

#[derive(Debug)]
pub struct TasksHead {}
impl TasksHead {
	pub async fn new() -> anyhow::Result<Self> {
		let listener = TcpListener::bind(super::ADDR).await?;

		tokio::spawn(async move {
			loop {
				let (mut socket, addr) = match listener.accept().await {
					Ok(a) => a,
					Err(err) => {
						eprintln!("server failed to accept connection: {err}");
						continue;
					}
				};

				tokio::spawn(async move {
					let mut internal = async move || -> anyhow::Result<()> {
						loop {
							let packet: ServerboundPacket = socket.read_as_packet().await?;

							match packet {
								ServerboundPacket::Hello { inst_id } => {
									println!("{inst_id} said hi")
								}
								ServerboundPacket::RequestTask { inst_id } => {
									let response = ClientboundPacket::AssignTask(Some(Task::Jump));
									socket.write_as_packet(response).await?;
								}
							}
						}
					};
					match internal().await {
						Ok(a) => a,
						Err(err) => {
							eprintln!("server error while handling {addr}: {err}");
							return;
						}
					}
				});
			}
		});
		println!("server listening on {}", super::ADDR);

		Ok(Self {})
	}
}
