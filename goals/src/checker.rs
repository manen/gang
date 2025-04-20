use azalea::pathfinder::goals::Goal;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct CheckerGoal;
impl Goal for CheckerGoal {
	fn heuristic(&self, n: azalea::BlockPos) -> f32 {
		if n.x % 3 == 1 { 100.0 } else { 0.0 }
	}
	fn success(&self, n: azalea::BlockPos) -> bool {
		n.x % 3 == 1
	}
}
