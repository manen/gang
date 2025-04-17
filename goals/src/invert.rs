use azalea::pathfinder::goals::Goal;

#[derive(Clone, Debug)]
pub struct Invert<G: Goal> {
	goal: G,
}
impl<G: Goal> Goal for Invert<G> {
	fn heuristic(&self, n: azalea::BlockPos) -> f32 {
		-self.goal.heuristic(n)
	}

	fn success(&self, n: azalea::BlockPos) -> bool {
		false
	}
}
