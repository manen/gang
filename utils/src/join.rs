// https://gist.github.com/manen/7eeb6e04a21306da637c08acdb21581d

use std::iter::Peekable;

enum Next {
	A,
	B,
}
impl Next {
	fn flip(&mut self) {
		*self = match self {
			Next::A => Next::B,
			Next::B => Next::A,
		}
	}
}
pub struct Join<T, A: Iterator<Item = T>, B: Iterator<Item = T>> {
	a: Peekable<A>,
	b: B,
	next: Next,
}
impl<T, A: Iterator<Item = T>, B: Iterator<Item = T>> Join<T, A, B> {
	pub fn new(a: A, b: B) -> Self {
		Self {
			a: a.peekable(),
			b,
			next: Next::A,
		}
	}
}
impl<T, A: Iterator<Item = T>, B: Iterator<Item = T>> Iterator for Join<T, A, B> {
	type Item = T;

	fn next(&mut self) -> Option<Self::Item> {
		let next = match self.next {
			Next::A => self.a.next(),
			Next::B => {
				if self.a.peek().is_some() {
					self.b.next()
				} else {
					None
				}
			}
		};
		self.next.flip();
		next
	}
}

#[test]
fn example() {
	let words: [&str; 3] = ["hello", "i'm", "testing"];
	let words: String = Join::new(words.into_iter(), std::iter::repeat(" ")).collect::<String>();

	// words: "hello i'm testing"
	//
	// this is good i think because it avoids like 1 extra allocation
	// and maybe the compiler optimizes away all this
	// and maybe this is faster than the accepted method
}
