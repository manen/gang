use tokio::net::TcpListener;

use crate::tasks::net::ServerboundPacket;

use honeypack::PacketRead;

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
					loop {
						let packet: ServerboundPacket = match socket.read_as_packet().await {
							Ok(a) => a,
							Err(err) => {
								eprintln!(
									"failed to read ServerboundPacket from {addr}, closing its socket\n{err}"
								);
								return;
							}
						};
					}
				});
			}
		});
		println!("server listening on {}", super::ADDR);

		Ok(Self {})
	}
}
