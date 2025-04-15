use std::{
	borrow::Cow,
	cell::RefCell,
	rc::Rc,
	sync::{Arc, Mutex},
	time::Duration,
};

use azalea::{
	prelude::Component, protocol::packets::game::ClientboundGamePacket, swarm::SwarmBuilder,
	world::MinecraftEntityId, Account, BlockPos, BotClientExt, Client, ClientBuilder, Event, Vec3,
};

pub mod execute;
use execute::*;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
	println!("Hello, world!");

	let accounts = |suffix| {
		[
			"fasz",
			"pocs",
			"spectator",
			"bot",
			"mike",
			"john_deer",
			"popcorn",
			"popbob",
		]
		.into_iter()
		.map(move |base| format!("{base}{suffix}"))
	};
	let accounts = accounts("")
		.chain(accounts("_1"))
		.chain(accounts("_2"))
		.chain(accounts("_fuk12"))
		.map(|name| Account::offline(name.as_ref()))
		.collect::<Vec<_>>();

	SwarmBuilder::new()
		.add_accounts(accounts)
		.set_handler(handle)
		.join_delay(Duration::from_millis(50))
		.start("localhost")
		.await?
}

#[derive(Default, Clone, bevy_ecs_macros::Component)]
pub struct State;

async fn handle(bot: Client, event: Event, state: State) -> anyhow::Result<()> {
	match event {
		Event::Chat(m) => {
			let content = m.content();
			let mut words = content.split(' ');

			if words.next() == Some("gang") {
				if words.next() == Some("listen") {
					execute(words, bot, state, m);
				}
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
