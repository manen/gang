#![feature(duration_constructors)]

use std::{ops::Deref, sync::Arc, time::Duration};

use anyhow::anyhow;
use azalea::{
	Account, BotClientExt, Client, Event,
	entity::EntityUuid,
	protocol::packets::game::ClientboundGamePacket,
	swarm::{Swarm, SwarmBuilder, SwarmEvent},
	world::MinecraftEntityId,
};
use tasks::{
	Task,
	net::{Tasks, start_server},
};
use tokio::{sync::Mutex, task::JoinHandle};

const DEFAULT_OWNER: &str = "manen_";
const ACCOUNTS: usize = 20;

pub mod namegen;
pub mod tasks;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
	println!("Hello, world!");

	std::thread::spawn(move || {
		loop {
			std::thread::sleep(Duration::from_secs(10));
			let deadlocks = parking_lot::deadlock::check_deadlock();
			if deadlocks.is_empty() {
				continue;
			}

			println!("{} deadlocks detected", deadlocks.len());
			for (i, threads) in deadlocks.iter().enumerate() {
				println!("Deadlock #{}", i);
				for t in threads {
					println!("Thread Id {:#?}", t.thread_id());
					println!("{:#?}", t.backtrace());
				}
			}
		}
	});

	let accounts = namegen::NameGen::default()
		.take(ACCOUNTS)
		.map(|name| Account::offline(name.as_ref()));

	let mut builder = SwarmBuilder::new()
		.set_handler(handle)
		.set_swarm_handler(swarm_handler);

	// tasks are created here, execution starts on Event::Spawn
	start_server(DEFAULT_OWNER.into()).await?;

	for (i, account) in accounts.enumerate() {
		let tasks = Tasks::new(i as _).await?;
		builder = builder.add_account_with_state(
			account,
			State {
				tasks: Some(Arc::new(Mutex::new(tasks))),
				handle: Arc::new(Mutex::new(None)),
				self_eid: Arc::new(Mutex::new(None)),
			},
		)
	}

	builder
		.set_swarm_state(State {
			tasks: Some(Arc::new(Mutex::new(Tasks::new(-1).await?))),
			handle: Arc::new(Mutex::new(None)),
			self_eid: Arc::new(Mutex::new(None)),
		})
		.join_delay(Duration::from_millis(50))
		.start("localhost")
		.await?;
}

#[derive(Default, Clone, bevy_ecs_macros::Component, bevy_ecs_macros::Resource)]
pub struct State {
	tasks: Option<Arc<Mutex<Tasks>>>,
	handle: Arc<Mutex<Option<JoinHandle<()>>>>,
	self_eid: Arc<Mutex<Option<MinecraftEntityId>>>,
}

async fn swarm_handler(swarm: Swarm, event: SwarmEvent, state: State) {
	match event {
		SwarmEvent::Chat(m) => {
			let content = m.content();
			let mut words = content.split(' ');

			if words.next() == Some("gang") {
				// match state.tasks.handle_command(words).await {
				// 	Ok(a) => a,
				// 	Err(err) => eprintln!("couldn't parse command {}: {err}", m.content()),
				// };
				println!("todo: not handling command {content}");
			}
		}
		_ => {}
	}
}

async fn handle(bot: Client, event: Event, state: State) -> anyhow::Result<()> {
	match event {
		Event::Chat(m) => {
			let content = m.content();
			let mut words = content.split(' ');

			let username = bot.username();
			let name = words.next();
			if name == Some("gang") || name == Some(&username) {
				if words.next() == Some("say") {
					let rest = utils::Join::new(words, std::iter::repeat(" ")).collect::<String>();
					bot.chat(&rest);
				}
			}
		}
		Event::Spawn => {
			if let Some(tasks) = &state.tasks {
				let mut tasks = tasks.lock().await;
				tasks.tick(&bot).await?;
			}

			if state.handle.lock().await.is_none() {
				let mut handle = state.handle.lock().await;
				*handle = Some(tokio::spawn(async move {
					let internal = async move || -> anyhow::Result<()> {
						loop {
							let task = {
								let tasks = match &state.tasks {
									Some(a) => a,
									None => return Err(anyhow!("state.tasks is None")),
								};
								let mut tasks = tasks.lock().await;
								tasks.next(&bot).await?
							};
							if task == Task::Halt {
								break;
							}
							let fluid_kind = {
								let at = bot.position().to_block_pos_floor();
								let world = bot.world();
								let world = world.read();
								world.get_fluid_state(&at).map(|a| a.kind)
							};
							{
								use azalea::blocks::fluid_state::FluidKind;
								match fluid_kind {
									Some(FluidKind::Water) => bot.set_jumping(true),
									_ => bot.set_jumping(false),
								}
							}
							match task.execute(&bot).await {
								Ok(a) => a,
								Err(err) => {
									eprintln!("{} couldn't execute {task:?}: {err}", bot.username())
								}
							};
						}
						Ok(())
					};
					match internal().await {
						Ok(a) => a,
						Err(err) => {
							eprintln!("error on Event::Spawn: {err}");
						}
					}
				}));
			}
		}
		Event::Tick => {
			// todo state.tasks.tick(&bot).await;
		}
		Event::Packet(p) => match p.as_ref() {
			ClientboundGamePacket::Login(login) => {
				let mut eid = state.self_eid.lock().await;
				*eid = Some(login.player_id);
			}
			ClientboundGamePacket::DamageEvent(dmg) => {
				let self_eid = {
					let self_eid = state.self_eid.lock().await;
					*self_eid
				};
				if let Some(self_eid) = self_eid {
					if dmg.entity_id == self_eid {
						// i'm taking damage!!!
						// println!("{dmg:#?}");

						if let Some(source_id) = dmg.source_cause_id.0 {
							let damager = {
								let world = bot.world();
								let world = world.read();
								world
									.entity_by_id
									.get(&MinecraftEntityId(source_id as i32))
									.cloned()
							};

							if let Some(damager) = damager {
								let uuid: Option<EntityUuid> = bot.get_entity_component(damager);

								if let Some(uuid) = uuid {
									// send the signal for the others to attack
									// todo state.tasks.agro(&bot, uuid.deref().clone()).await;
								} else {
									eprintln!(
										"got damaged and could identify the entity doing the damaging but that entity doesn't have a EntityUuid component"
									)
								}
							}
						}
					}
				}
			}
			_ => {}
		},
		_ => {}
	}
	Ok(())
}
