use serde::{Serialize, de::DeserializeOwned};
use tokio::{
	io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt},
	pin,
};

use crate::*;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Packet<T> {
	data: T,
}
impl<T> Packet<T> {
	pub fn new(data: T) -> Self {
		Self { data }
	}
	pub fn take(self) -> T {
		self.data
	}
}
impl<T> AsRef<T> for Packet<T> {
	fn as_ref(&self) -> &T {
		&self.data
	}
}

impl<T: DeserializeOwned> Packet<T> {
	pub async fn read_from<R: AsyncRead>(read: R) -> Result<Self> {
		pin!(read);

		let len = read.read_u32().await?;
		let mut buf = vec![0_u8; len as usize];
		read.read(&mut buf).await?;

		let data: T = bincode::deserialize(&buf)?;

		Ok(Self { data })
	}
}
impl<T: Serialize> Packet<T> {
	pub async fn write_to<W: AsyncWrite>(&self, write: W) -> Result<()> {
		pin!(write);

		let buf = bincode::serialize(&self.data)?;
		let len = buf.len() as u32;

		write.write_u32(len).await?;
		write.write(&buf).await?;

		Ok(())
	}
}
