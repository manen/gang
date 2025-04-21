use honeypack::PacketRead;
use tokio::net::TcpStream;

use crate::tasks::net::ClientboundPacket;

#[derive(Debug)]
pub struct TasksClient {}
impl TasksClient {
	pub async fn new() -> anyhow::Result<Self> {
		let mut stream = TcpStream::connect(super::ADDR).await?;
		println!("client connected to {}", super::ADDR);

		tokio::spawn(async move {
			loop {
				let packet: ClientboundPacket = match stream.read_as_packet().await {
					Ok(a) => a,
					Err(err) => {
						eprintln!(
							"failed to read ClientboundPacket from {}, closing the socket\n{err}",
							super::ADDR
						);
						return;
					}
				};
			}
		});

		Ok(Self {})
	}
}
