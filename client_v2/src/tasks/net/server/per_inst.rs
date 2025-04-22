use std::collections::HashMap;

use crate::tasks::Task;

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
