use serde::{Serialize, de::DeserializeOwned};
use tokio::io::{AsyncRead, AsyncWrite};

use crate::*;

pub trait PacketWrite: AsyncWrite + Unpin {
	fn write_as_packet<T: Serialize>(
		&mut self,
		data: T,
	) -> impl Future<Output = std::result::Result<(), Error>> {
		async fn internal<W: AsyncWrite, T: Serialize>(w: W, data: T) -> Result<()> {
			let packet = Packet::new(data);
			packet.write_to(w).await?;

			Ok(())
		}

		internal(self, data)
	}
}
pub trait PacketRead: AsyncRead + Unpin {
	fn read_as_packet<T: DeserializeOwned>(
		&mut self,
	) -> impl Future<Output = std::result::Result<T, Error>> {
		async fn internal<R: AsyncRead, T: DeserializeOwned>(r: R) -> Result<T> {
			let packet = Packet::read_from(r).await?;
			Ok(packet.take())
		}

		internal(self)
	}
}
