use std::sync::Arc;

use tokio::{net::TcpListener, sync::Mutex};

use crate::tasks::{
	Task,
	net::{ClientboundPacket, ServerboundPacket},
};

use honeypack::{PacketRead, PacketWrite};

#[derive(Debug)]
struct ServerData {
	owner: String,
}

/// functional baby
pub async fn start_server(owner: String) -> anyhow::Result<()> {
	let listener = TcpListener::bind(super::ADDR).await?;

	let data = ServerData { owner };
	let data = Arc::new(Mutex::new(data));

	tokio::spawn(async move {
		loop {
			let (mut socket, addr) = match listener.accept().await {
				Ok(a) => a,
				Err(err) => {
					eprintln!("server failed to accept connection: {err}");
					continue;
				}
			};

			let data = data.clone();
			tokio::spawn(async move {
				let mut internal = async || -> anyhow::Result<()> {
					loop {
						let packet: ServerboundPacket = socket.read_as_packet().await?;

						match packet {
							ServerboundPacket::Hello { inst_id } => {
								println!("{inst_id} said hi");
								let response = ClientboundPacket::OwnerIs {
									username: data.lock().await.owner.clone(),
								};
								socket.write_as_packet(response).await?;
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
	Ok(())
}
