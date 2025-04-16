use std::{collections::VecDeque, future::Future, sync::Arc};

use anyhow::anyhow;
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
		Some("everything") => match iter.next() {
			Some("from") => {
				let from_x = iter
					.next()
					.ok_or_else(|| anyhow!("expected x coordinate"))?
					.parse()?;
				let from_y = iter
					.next()
					.ok_or_else(|| anyhow!("expected y coordinate"))?
					.parse()?;
				let from_z = iter
					.next()
					.ok_or_else(|| anyhow!("expected z coordinate"))?
					.parse()?;

				if iter.next() != Some("to") {
					return Err(anyhow!("expected keyword to after x, y, z coordinates"));
				}

				let to_x = iter
					.next()
					.ok_or_else(|| anyhow!("expected x coordinate"))?
					.parse()?;
				let to_y = iter
					.next()
					.ok_or_else(|| anyhow!("expected y coordinate"))?
					.parse()?;
				let to_z = iter
					.next()
					.ok_or_else(|| anyhow!("expected z coordinate"))?
					.parse()?;

				mine_everything(swarm, (from_x, from_y, from_z), (to_x, to_y, to_z)).await;
			}
			Some(_) => {}
			None => {
				mine_a_lot(
					swarm,
					[
						blocks::GrassBlock { snowy: false }.into(),
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
						blocks::Dirt {}.into(),
					],
					30,
				);
			}
		},
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

async fn mine_everything(swarm: Swarm, from: (i32, i32, i32), to: (i32, i32, i32)) {
	println!("mining everything from {from:?} to {to:?}");

	let from_x = from.0.max(to.0);
	let from_y = from.1.max(to.1);
	let from_z = from.2.max(to.2);
	let to_x = from.0.min(to.0);
	let to_y = from.1.min(to.1);
	let to_z = from.2.min(to.2);

	println!("from: {from_x} {from_y} {from_z}\nto: {to_x} {to_y} {to_z}");

	// why is yield experimental fml
	let mut blocks = VecDeque::new();
	for y in (to_y..from_y + 1).rev() {
		println!("y level {y}");
		for x in to_x..from_x + 1 {
			println!("x {x}");
			for z in to_z..from_z + 1 {
				blocks.push_back(BlockPos { x, y, z });
				println!("block {x} {y} {z}")
			}
		}
	}

	let blocks = Arc::new(Mutex::new(blocks));

	let processes = swarm.into_iter().map(|bot| {
		let blocks = blocks.clone();
		async move {
			loop {
				let block = {
					let mut blocks = blocks.lock().await;
					blocks.pop_front()
				};
				if let Some(pos) = block {
					println!("{} mining {pos:?}", bot.username());

					bot.goto(RadiusGoal {
						pos: pos.center(),
						radius: 3.5,
					})
					.await;

					bot.look_at(pos.center());
					bot.mine(pos).await;
				} else {
					println!("{} finished mining", bot.username());
					break;
				}
			}
		}
	});

	futures::future::join_all(processes).await;
}

fn mine_a_lot<I: IntoIterator<Item = BlockState>>(swarm: Swarm, states: I, count_per_bot: i32) {
	let occupied = Arc::new(Mutex::new(Vec::<BlockPos>::new()));

	let states = states.into_iter();
	let states = BlockStates {
		set: states.collect(),
	};

	for bot in swarm.into_iter() {
		let occupied = occupied.clone();
		let states = states.clone();
		tokio::spawn(async move {
			for _ in 0..count_per_bot {
				let me = bot.eye_position();

				let pos = {
					let mut blocks = {
						let world = bot.world();
						let world = world.read();
						let blocks = world.find_blocks(me, &states);
						blocks.take(10).collect::<Vec<_>>().into_iter()
					};

					let block = loop {
						match blocks.next() {
							Some(pos) => {
								if !{
									let occupied = occupied.lock().await;
									occupied.contains(&pos)
								} {
									let mut occupied = occupied.lock().await;
									occupied.push(pos);
									break Some(pos);
								}
							}
							None => break None,
						}
					};
					block
				};

				if let Some(pos) = pos {
					bot.goto(RadiusGoal {
						pos: pos.center(),
						radius: 3.5,
					})
					.await;

					bot.look_at(pos.center());
					bot.mine(pos).await;
				}
			}
		});
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
