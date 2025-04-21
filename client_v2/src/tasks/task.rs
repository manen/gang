#[derive(Clone, Debug, PartialEq)]
pub enum Task {
	/// halts task execution. if a bot receives this task it will not poll or execute any further tasks
	Halt,
	Jump,
	Goto(RadiusGoal),
	Mine(BlockPos),
	Attack(Uuid),
}
impl Task {
	pub async fn execute(&self, bot: &Client) -> anyhow::Result<()> {
		match self {
			Self::Attack(uuid) => {
				let entity = bot
					.entity_by_uuid(*uuid)
					.ok_or_else(|| anyhow!("couldn't find an entity with uuid {uuid}"))?;

				let eid: MinecraftEntityId = bot.get_entity_component(entity).ok_or_else(|| {
					anyhow!(
						"there wasn't an entityid component on the entity {} was supposed to attack",
						bot.username()
					)
				})?;

				let eye_offset: Option<EyeHeight> = bot.get_entity_component(entity);
				let eye_offset = eye_offset.map(|a| a.deref().clone()).unwrap_or_default();

				let pos_now = || {
					let pos: Position = bot.get_entity_component(entity).ok_or_else(|| {
						anyhow!(
							"there wasn't a position component on the entity {} was supposed to attack",
							bot.username()
						)
					})?;
					let pos = pos.down(0.0);
					anyhow::Ok(pos)
				};

				let pos = loop {
					let start_pos = pos_now()?;
					let goal = RadiusGoal {
						pos: start_pos,
						radius: 3.5,
					};
					if goal.success(bot.position().to_block_pos_floor()) {
						break start_pos;
					} else {
						// this is a reimplementation of bot.goto that will change the target if it moved too far
						bot.start_goto(goal);
						bot.wait_one_update().await;

						let mut tick_broadcaster = bot.get_tick_broadcaster();
						'pathing: while !bot.is_goto_target_reached() {
							// check every tick
							match tick_broadcaster.recv().await {
								Ok(_) => (),
								Err(_err) => (),
							};
							let pos = pos_now()?;
							if pos.distance_to(&start_pos) >= 5.0 {
								bot.stop_pathfinding();
								println!(
									"looping again in Attack, since the target entity has moved at least 5 blocks"
								);
								tokio::time::sleep(Duration::from_millis(100)).await;
								break 'pathing;
							}
						}
					}
				};

				bot.look_at(pos.up(eye_offset as _));
				bot.attack(eid);
				tokio::time::sleep(Duration::from_millis(400)).await
			}
			Self::Jump => {
				bot.jump();
			}
			Self::Goto(goal) => {
				if !goal.success(bot.position().to_block_pos_floor()) {
					bot.start_goto(*goal);
				}
				tokio::time::sleep(Duration::from_millis(500)).await
			}
			Self::Mine(pos) => {
				if !bot
					.world()
					.read()
					.get_block_state(pos)
					.map(|state| state.is_air())
					.unwrap_or(false)
				{
					let goal = RadiusGoal {
						pos: pos.center(),
						radius: 3.5,
					};

					if !goal.success(bot.position().to_block_pos_floor()) {
						bot.goto(goal).await;
					}
					bot.look_at(pos.center());
					bot.mine(*pos).await;

					tokio::time::sleep(Duration::from_millis(50)).await;
				}
			}
			Self::Halt => {}
		}
		Ok(())
	}
}
