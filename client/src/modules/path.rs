use anyhow::anyhow;
use azalea::{
	chat::ChatPacket,
	pathfinder::goals::{BlockPosGoal, XZGoal, YGoal},
	prelude::PathfinderClientExt,
	BlockPos, Client,
};

use crate::State;

pub async fn path<'a, I: IntoIterator<Item = &'a str>>(
	iter: I,
	bot: Client,
	state: State,
	chat: ChatPacket,
) -> anyhow::Result<()> {
	let mut iter = iter.into_iter();
	match iter.next() {
		Some("here") => {}
		Some("around") => {
			let radius = if let Some(radius) = iter.next() {
				radius.parse()?
			} else {
				8
			};
			let (x_offset, z_offset) = {
				// Obtain the current time since the UNIX epoch in nanoseconds
				let nanos = std::time::SystemTime::now()
					.duration_since(std::time::UNIX_EPOCH)
					.expect("Time went backwards")
					.subsec_nanos();

				// Use the nanoseconds as a seed for the PRNG
				let mut seed = nanos as u64;

				// Generate a pseudo-random number using a simple linear congruential generator (LCG)
				seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1);

				let x_offset = (seed % radius) as u32;

				// Generate a pseudo-random number using a simple linear congruential generator (LCG)
				seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1);

				(x_offset, (seed % radius) as u32)
			};

			bot.goto(XZGoal {
				x: (bot.position().x + x_offset as f64 - x_offset as f64 / 2.0).round() as i32,
				z: (bot.position().z + z_offset as f64 - x_offset as f64 / 2.0).round() as i32,
			})
		}
		Some("to") => {
			{
				let first = iter.next().ok_or_else(|| anyhow!("expected coordinates"))?;
				if let Some(second) = iter.next() {
					if let Some(third) = iter.next() {
						bot.goto(BlockPosGoal(BlockPos {
							x: first.parse()?,
							y: second.parse()?,
							z: third.parse()?,
						}))
					} else {
						bot.goto(XZGoal {
							x: first.parse()?,
							z: second.parse()?,
						})
					}
				} else {
					bot.goto(YGoal { y: first.parse()? })
				}
			};
		}
		_ => {}
	}
	Ok(())
}
