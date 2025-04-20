#![feature(duration_constructors)]

use std::{sync::Arc, time::Duration};

use azalea::{
	Account, BotClientExt, Client, Event,
	swarm::{Swarm, SwarmBuilder, SwarmEvent},
};
use tasks::{Task, Tasks};
use tokio::{sync::Mutex, task::JoinHandle};

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

	let accounts = |suffix| {
		["pop", "bob", "test", "bot", "stick"]
			.into_iter()
			.map(move |base| format!("{base}{suffix}"))
	};
	let accounts = accounts("")
		.chain(accounts("_1"))
		.chain(accounts("_2"))
		// .chain(accounts("_3"))
		.map(|name| Account::offline(name.as_ref()))
		.collect::<Vec<_>>();

	let mut builder = SwarmBuilder::new()
		.set_handler(handle)
		.set_swarm_handler(swarm_handler);

	// tasks are created here, execution starts on Event::Spawn

	let tasks = Tasks {
		owner: Arc::new(Mutex::new("manen_".into())),
		..Default::default()
	};

	for account in accounts {
		builder = builder.add_account_with_state(
			account,
			State {
				tasks: tasks.clone(),
				handle: Arc::new(Mutex::new(None)),
			},
		)
	}

	builder
		.set_swarm_state(State {
			tasks: tasks.clone(),
			handle: Arc::new(Mutex::new(None)),
		})
		.join_delay(Duration::from_millis(50))
		.start("localhost")
		.await?;
}

#[derive(Default, Clone, bevy_ecs_macros::Component, bevy_ecs_macros::Resource)]
pub struct State {
	tasks: Tasks,
	handle: Arc<Mutex<Option<JoinHandle<()>>>>,
}

async fn swarm_handler(swarm: Swarm, event: SwarmEvent, state: State) {
	match event {
		SwarmEvent::Chat(m) => {
			let content = m.content();
			let mut words = content.split(' ');

			if words.next() == Some("gang") {
				match state.tasks.handle_command(words).await {
					Ok(a) => a,
					Err(err) => eprintln!("couldn't parse command {}: {err}", m.content()),
				};
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
			state.tasks.tick(&bot).await;
			if state.handle.lock().await.is_none() {
				let mut handle = state.handle.lock().await;
				*handle = Some(tokio::spawn(async move {
					loop {
						let next = state.tasks.next().await;
						if next == Some(Task::Halt) {
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
						if let Some(task) = next {
							match task.execute(&bot).await {
								Ok(a) => a,
								Err(err) => {
									eprintln!("{} couldn't execute {task:?}: {err}", bot.username())
								}
							};
						}
					}
				}));
			}
		}
		Event::Tick => {
			state.tasks.tick(&bot).await;
		}
		_ => {}
	}
	Ok(())
}
