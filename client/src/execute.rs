use azalea::swarm::Swarm;
use azalea::BotClientExt;
use azalea::{chat::ChatPacket, Client};

use crate::{modules, State};

pub async fn execute<'a, I: IntoIterator<Item = &'a str>>(
	iter: I,
	bot: Client,
	state: State,
	chat: ChatPacket,
) -> anyhow::Result<()> {
	let mut iter = iter.into_iter();
	match iter.next() {
		Some("say") => {
			let text = utils::Join::new(iter, std::iter::repeat(" ")).collect::<String>();
			bot.chat(&text);
			Ok(())
		}
		Some("jump") => {
			bot.set_jumping(true);
			Ok(())
		}
		Some("path") => modules::path::path(iter, bot, state, chat).await,
		_ => Ok(()),
	}
}
pub async fn execute_swarm<'a, I: IntoIterator<Item = &'a str>>(
	iter: I,
	swarm: Swarm,
	state: State,
	chat: ChatPacket,
) -> anyhow::Result<()> {
	let mut iter = iter.into_iter();
	match iter.next() {
		Some("mine") => modules::mine::mine(iter, swarm, state, chat).await,
		_ => Ok(()),
	}
}
