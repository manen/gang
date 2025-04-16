use std::sync::Arc;

use azalea::{
	blocks::{BlockState, BlockStates},
	chat::ChatPacket,
	pathfinder::goals::{BlockPosGoal, RadiusGoal},
	prelude::PathfinderClientExt,
	swarm::Swarm,
	BlockPos, BotClientExt, Client,
};
use tokio::sync::Mutex;

use crate::State;

pub async fn mine<'a, I: IntoIterator<Item = &'a str>>(
	iter: I,
	swarm: Swarm,
	state: State,
	chat: ChatPacket,
) -> anyhow::Result<()> {
	use azalea::blocks::blocks;
	let mut iter = iter.into_iter();
	match iter.next() {
		Some("grass") => {
			mine_a_lot(swarm, [blocks::GrassBlock { snowy: false }.into()], 10);
		}
		Some("leaves") => mine_specific(
			swarm,
			[blocks::OakLeaves {
				distance: azalea::blocks::properties::OakLeavesDistance::_1,
				persistent: false,
				waterlogged: false,
			}
			.into()],
		),
		Some("wood") => mine_specific(
			swarm,
			[
				blocks::OakLog {
					axis: azalea::blocks::properties::Axis::X,
				}
				.into(),
				blocks::OakLog {
					axis: azalea::blocks::properties::Axis::Y,
				}
				.into(),
				blocks::OakLog {
					axis: azalea::blocks::properties::Axis::Z,
				}
				.into(),
			],
		),
		_ => {}
	}
	Ok(())
}

fn mine_a_lot<I: IntoIterator<Item = BlockState>>(swarm: Swarm, states: I, count_per_bot: i32) {
	let mut occupied = Arc::new(Mutex::new(Vec::<BlockPos>::new()));

	let states = states.into_iter();
	let states = BlockStates {
		set: states.collect(),
	};

	for bot in swarm.into_iter() {
		tokio::spawn(async move {});
	}
}

fn mine_specific<I: IntoIterator<Item = BlockState>>(swarm: Swarm, states: I) {
	let mut occupied = vec![];

	let states = states.into_iter();
	let states = BlockStates {
		set: states.collect(),
	};

	for bot in swarm.into_iter() {
		let pos = {
			let world = bot.world();
			let world = world.read();
			let mut blocks = world.find_blocks(bot.eye_position(), &states);

			let block = loop {
				match blocks.next() {
					Some(pos) => {
						if !occupied.contains(&pos) {
							occupied.push(pos);
							break Some(pos);
						}
					}
					None => break None,
				}
			};
			block
		};
		tokio::spawn(async move {
			if let Some(pos) = pos {
				bot.goto(RadiusGoal {
					pos: pos.center(),
					radius: 3.5,
				})
				.await;

				bot.look_at(pos.center());
				bot.mine(pos).await;
			}
		});
	}
}
