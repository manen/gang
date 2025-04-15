use azalea::{chat::ChatPacket, pathfinder::PathfinderClientExt, Client};

use crate::State;

pub fn execute<'a, I: IntoIterator<Item = &'a str>>(
	iter: I,
	bot: Client,
	state: State,
	chat: ChatPacket,
) {
	let mut iter = iter.into_iter();
	match iter.next() {
		Some("say") => {
			let text = utils::Join::new(iter, std::iter::repeat(" ")).collect::<String>();
			bot.chat(&text);
		}
		Some("path") => match iter.next() {
			Some("here") => bot.chat("hol up unimplemented"),
			Some("to") => {
				let coords = utils::Join::new(iter, std::iter::repeat(" ")).collect::<String>();
				// coords.
			}
			_ => {}
		},
		_ => (),
	}
}
