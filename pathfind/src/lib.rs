use azalea::{
	BlockPos, BotClientExt, Client, Vec3,
	auto_tool::AutoToolClientExt,
	core::direction::Direction,
	entity::Pose,
	pathfinder::goals::{BlockPosGoal, Goal, RadiusGoal},
	prelude::{ContainerClientExt, PathfinderClientExt},
};

pub async fn path_to(bot: &Client) {
	let pos = bot.position();

	let target = BlockPos {
		z: pos.z as i32 + 1,
		y: pos.y as i32 - 1,
		..pos.to_block_pos_floor()
	};

	bot.goto(BlockPosGoal(pos.to_block_pos_floor())).await;

	bot.look_at(target.center());
	bot.block_interact(target);
}

pub async fn place_block(bot: &Client, pos: BlockPos) {
	// goal: place blocks until it's possible to place the block at pos
	// non-goal (for now): anything else, anything relating to hotbar slots and inventory, will assume block is already in hand

	loop {
		let (right_click_block, look_at, done) = nearest_block(bot, pos).await;
		bot.look_at(look_at);
		// sneaking seems impossible but is required for all this to work and never open a furnace or something by accident
		bot.block_interact(right_click_block);

		if done {
			break;
		}
	}
}

/// returns (block we can right click, where to look when clicking, are we done)
async fn nearest_block(bot: &Client, pos: BlockPos) -> (BlockPos, Vec3, bool) {
	let dirs = [
		Direction::Down,
		Direction::Up,
		Direction::North,
		Direction::South,
		Direction::West,
		Direction::East,
	]
	.iter()
	.copied();

	{
		let mut dirs = dirs.clone();
		let world = bot.world();
		let world = world.read();
		loop {
			if let Some(dir) = dirs.next() {
				let test = pos.offset_with_direction(dir);
				if let Some(_) = world.get_block_state(&test) {
					return (test, test.center(), true);
				}
			} else {
			}
		}
	}
	todo!()
}
