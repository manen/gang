use serde::{Serialize, de::DeserializeOwned};
use tokio::io::{AsyncRead, AsyncWrite};

use crate::*;

pub trait PacketWrite: AsyncWrite + Unpin {
	fn write_as_packet<T: Serialize>(
		&mut self,
		data: T,
	) -> impl Future<Output = std::result::Result<(), Error>>;
}
pub trait PacketRead: AsyncRead + Unpin {
	fn read_as_packet<T: DeserializeOwned>(
		&mut self,
	) -> impl Future<Output = std::result::Result<T, Error>>;
}

impl<W: AsyncWrite + Unpin> PacketWrite for W {
	async fn write_as_packet<T: Serialize>(&mut self, data: T) -> Result<()> {
		let packet = Packet::new(data);
		packet.write_to(self).await?;

		Ok(())
	}
}
impl<R: AsyncRead + Unpin> PacketRead for R {
	async fn read_as_packet<T: DeserializeOwned>(&mut self) -> Result<T> {
		let packet = Packet::read_from(self).await?;
		Ok(packet.take())
	}
}
