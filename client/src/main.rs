use std::time::Duration;

use azalea::{
	protocol::packets::game::ClientboundGamePacket,
	swarm::{Swarm, SwarmBuilder, SwarmEvent},
	Account, Client, Event,
};

pub mod execute;
use execute::*;

pub mod modules;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
	println!("Hello, world!");

	std::thread::spawn(move || loop {
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
	});

	let accounts = |suffix| {
		["pop", "bob", "test", "bot", "stick"]
			.into_iter()
			.map(move |base| format!("{base}{suffix}"))
	};
	let accounts = accounts("")
		.chain(accounts("_1"))
		.map(|name| Account::offline(name.as_ref()))
		.collect::<Vec<_>>();

	SwarmBuilder::new()
		.add_accounts(accounts)
		.set_handler(handle)
		.set_swarm_handler(swarm_handler)
		.join_delay(Duration::from_millis(50))
		.start("localhost")
		.await?
}

#[derive(Default, Clone, bevy_ecs_macros::Component, bevy_ecs_macros::Resource)]
pub struct State;

async fn swarm_handler(swarm: Swarm, event: SwarmEvent, state: State) {
	match event {
		SwarmEvent::Chat(m) => {
			let content = m.content();
			let mut words = content.split(' ');

			if words.next() == Some("gang") {
				match execute_swarm(words, swarm, state, m).await {
					Ok(_) => {}
					Err(err) => {
						eprintln!("swarm command failed: \"{content}\"\n{err}")
					}
				}
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
				match execute(words, bot, state, m).await {
					Ok(_) => {}
					Err(err) => {
						eprintln!("{} failed at: \"{}\"\n{err}", username, content)
					}
				};
			}
		}
		Event::Packet(packet) => match packet.as_ref() {
			ClientboundGamePacket::Login(event) => {
				println!("i'm {}", event.player_id)
			}
			// ClientboundGamePacket::DamageEvent(event) => {
			// 	if let Some(direct_id) = event.source_cause_id.0 {
			// 		bot.
			// 	}
			// }
			_ => {}
		},
		_ => {}
	}
	Ok(())
}
