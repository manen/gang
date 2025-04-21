use std::{
	borrow::Cow,
	collections::{HashMap, VecDeque},
	ops::Deref,
	sync::Arc,
	time::{Duration, Instant},
};

use anyhow::anyhow;
use azalea::{
	BlockPos, BotClientExt, Client, GameProfileComponent, Vec3,
	entity::{EyeHeight, Position, metadata::Player},
	pathfinder::goals::{Goal, RadiusGoal},
	prelude::PathfinderClientExt,
	world::MinecraftEntityId,
};
use bevy_ecs::query::With;
use tokio::sync::Mutex;
use uuid::Uuid;

#[derive(Copy, Clone, Debug)]
pub struct OwnerPos {
	time: Instant,
	pos: Vec3,
}
impl Default for OwnerPos {
	fn default() -> Self {
		Self {
			time: Instant::now() - Duration::new(60 * 60 * 10, 0),
			pos: Default::default(),
		}
	}
}
impl OwnerPos {
	pub fn new(pos: Vec3) -> Self {
		Self {
			time: Instant::now(),
			pos,
		}
	}
}

#[derive(Clone, Debug, Default)]
pub struct PerInstanceTasks {
	tasks: Vec<PerInstanceTask>,
}
impl PerInstanceTasks {
	pub fn new_task(&mut self, task: Task) {
		self.tasks.push(PerInstanceTask::new(task));
	}
	pub fn new_task_times(&mut self, task: Task, times: i32) {
		self.tasks.push(PerInstanceTask::new_times(task, times));
	}
	pub fn clear(&mut self) {
		self.tasks.clear()
	}
	pub fn task_for(&mut self, id: i32) -> Option<Task> {
		for per_inst in self.tasks.iter_mut() {
			if let Some(per_inst) = per_inst.task_for(id) {
				return Some(per_inst);
			}
		}
		return None;
	}
}

#[derive(Clone, Debug)]
pub struct PerInstanceTask {
	already_executed: HashMap<i32, i32>,
	/// how many times an instance is required to complete this task
	times: i32,
	task: Task,
}
impl PerInstanceTask {
	pub fn new(task: Task) -> Self {
		Self::new_times(task, 1)
	}
	pub fn new_times(task: Task, times: i32) -> Self {
		Self {
			already_executed: Default::default(),
			times,
			task,
		}
	}
	pub fn assign_new(&mut self, task: Task) {
		self.already_executed.clear();
		self.task = task;
	}

	pub fn task_for(&mut self, id: i32) -> Option<Task> {
		if let Some(entry) = self.already_executed.get_mut(&id) {
			if *entry >= self.times {
				return None;
			}
			*entry += 1;
			return Some(self.task.clone());
		}

		self.already_executed.insert(id, 1);
		Some(self.task.clone())
	}
}

#[derive(Default, Clone, Debug)]
pub struct Tasks {
	inst_id: Option<i32>,

	owner: Arc<Mutex<Cow<'static, str>>>,
	owner_pos: Arc<Mutex<OwnerPos>>,
	queue: Arc<Mutex<VecDeque<Task>>>,
	per_instance_task: Arc<Mutex<PerInstanceTasks>>,
}
impl Tasks {
	pub fn with_owner(owner: impl Into<Cow<'static, str>>) -> Self {
		Self {
			owner: Arc::new(Mutex::new(owner.into())),
			..Default::default()
		}
	}
	pub fn set_inst_id(&mut self, inst_id: Option<i32>) {
		self.inst_id = inst_id;
	}

	pub async fn append_to_queue(&self, new_queue: impl IntoIterator<Item = Task>) {
		{
			let mut queue = self.queue.lock().await;
			let taken_queue = std::mem::replace(&mut *queue, Default::default());
			*queue = taken_queue.into_iter().chain(new_queue).collect();
		}
	}

	pub async fn next(&self) -> Option<Task> {
		if let Some(inst_id) = self.inst_id {
			let per_instance = {
				let mut per_instance = self.per_instance_task.lock().await;
				per_instance.task_for(inst_id)
			};
			if let Some(per_instance) = per_instance {
				return Some(per_instance);
			}
		}

		let from_queue = {
			let mut queue = self.queue.lock().await;
			queue.pop_front()
		};
		if let Some(from_queue) = from_queue {
			return Some(from_queue);
		}

		let owner_pos = self.owner_pos.lock().await.clone();
		if owner_pos.time.elapsed() < Duration::from_mins(2) {
			return Some(Task::Goto(RadiusGoal {
				pos: owner_pos.pos,
				radius: 10.0,
			}));
		}

		Some(Task::Jump)
	}

	pub async fn tick(&self, bot: &Client) {
		let age = {
			let owner_pos = self.owner_pos.lock().await;
			owner_pos.time.elapsed()
		};

		if age > Duration::from_millis(300) {
			let entity = {
				let find = self.owner.lock().await;
				bot.entity_by::<With<Player>, &GameProfileComponent>(
					|profile: &&GameProfileComponent| profile.name == *find,
				)
			};
			if let Some(player) = entity {
				let pos: Option<Position> = bot.get_entity_component(player);
				if let Some(pos) = pos {
					let mut owner_pos = self.owner_pos.lock().await;
					*owner_pos = OwnerPos::new(pos.down(0.0));
				}
			}
		}
	}
	pub async fn agro(&self, _bot: &Client, uuid: Uuid) {
		let mut per_inst = self.per_instance_task.lock().await;
		per_inst.new_task_times(Task::Attack(uuid), 3);
	}

	pub async fn handle_command<'a, I: IntoIterator<Item = &'a str>>(
		&self,
		words: I,
	) -> anyhow::Result<()> {
		let mut words = words.into_iter();
		match words.next() {
			Some("demolish") => {
				let from_x: i32 = words
					.next()
					.ok_or_else(|| anyhow!("expected x coordinate"))?
					.parse()?;
				let from_y: i32 = words
					.next()
					.ok_or_else(|| anyhow!("expected y coordinate"))?
					.parse()?;
				let from_z: i32 = words
					.next()
					.ok_or_else(|| anyhow!("expected z coordinate"))?
					.parse()?;

				let to_x: i32 = words
					.next()
					.ok_or_else(|| anyhow!("expected x coordinate"))?
					.parse()?;
				let to_y: i32 = words
					.next()
					.ok_or_else(|| anyhow!("expected y coordinate"))?
					.parse()?;
				let to_z: i32 = words
					.next()
					.ok_or_else(|| anyhow!("expected z coordinate"))?
					.parse()?;

				let from = (from_x, from_y, from_z);
				let to = (to_x, to_y, to_z);

				let from_x = from.0.max(to.0);
				let from_y = from.1.max(to.1);
				let from_z = from.2.max(to.2);
				let to_x = from.0.min(to.0);
				let to_y = from.1.min(to.1);
				let to_z = from.2.min(to.2);

				// {
				// 	let mut new_queue = VecDeque::new();
				// 	for y in (to_y..from_y + 1).rev() {
				// 		for x in to_x..from_x + 1 {
				// 			for z in to_z..from_z + 1 {
				// 				new_queue.push_back(Task::Mine(BlockPos { x, y, z }));
				// 			}
				// 		}
				// 	}
				// }
				self.append_to_queue(
					(to_y..from_y + 1)
						.rev()
						.map(move |y| {
							(to_x..from_x + 1).map(move |x| {
								(to_z..from_z + 1).map(move |z| Task::Mine(BlockPos { x, y, z }))
							})
						})
						.flatten()
						.flatten(),
				)
				.await;
			}
			Some("follow") => {
				let name = words
					.next()
					.ok_or_else(|| anyhow!("expected player name after follow"))?;
				let name = name.to_owned();
				let mut owner = self.owner.lock().await;
				*owner = Cow::Owned(name)
			}
			Some("stop") => {
				{
					let mut queue = self.queue.lock().await;
					queue.clear();
				}
				{
					let mut per_inst = self.queue.lock().await;
					per_inst.clear();
				}
			}
			_ => {}
		}
		Ok(())
	}
}
