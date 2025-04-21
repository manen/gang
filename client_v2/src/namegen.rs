use std::borrow::Cow;

pub const NAMES: &str = include_str!("../../names.txt");

#[derive(Debug, Clone)]
pub struct NameGen<'a> {
	names: Vec<Cow<'a, str>>,
	i: usize,
	rep: usize,
}
impl Default for NameGen<'static> {
	fn default() -> Self {
		Self::new_from_names(NAMES.split('\n'))
	}
}
impl<'a> NameGen<'a> {
	pub fn new_from_names<S: Into<Cow<'a, str>>>(names: impl IntoIterator<Item = S>) -> Self {
		let names = names.into_iter();
		let names = names.map(|a| a.into());
		Self {
			names: names.collect(),
			i: 0,
			rep: 0,
		}
	}
}

impl<'a> Iterator for NameGen<'a> {
	type Item = String;

	fn next(&mut self) -> Option<Self::Item> {
		if self.names.len() == 0 {
			return None;
		}

		let process = |i: Cow<'a, str>| {
			let i = i.to_lowercase();
			match self.rep {
				0 => i,
				rep => format!("{i}_{rep}"),
			}
		};

		let next = self.names.iter().nth(self.i);
		if let Some(next) = next {
			self.i += 1;
			Some(process(next.clone()))
		} else {
			self.rep += 1;
			self.i = 0;
			self.next()
		}
	}
}
