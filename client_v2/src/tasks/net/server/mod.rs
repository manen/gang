use std::{
	collections::VecDeque,
	iter::Enumerate,
	sync::Arc,
	time::{Duration, Instant},
};

use anyhow::anyhow;
use azalea::{BlockPos, Vec3, pathfinder::goals::RadiusGoal};
use tokio::{net::TcpListener, sync::Mutex};

pub mod per_inst;

use crate::{
	namegen::NameGen,
	tasks::{
		Task,
		net::{
			ClientboundHelloPacket, ClientboundPacket, ServerboundHelloPacket, ServerboundPacket,
		},
	},
};

use honeypack::{PacketRead, PacketWrite};

use super::PosReport;

#[derive(Debug)]
struct ServerData {
	namegen: Enumerate<NameGen<'static>>,
	owner: String,
	owner_pos: (Instant, Vec3),
	chat_hash_handled: Vec<u64>,

	task_queue: VecDeque<Task>,
	per_inst: per_inst::PerInstanceTasks,
}

/// functional baby
pub async fn start_server(owner: String) -> anyhow::Result<()> {
	let listener = TcpListener::bind(super::ADDR).await?;

	let data = ServerData {
		owner,
		owner_pos: (Instant::now() - Duration::from_hours(1), Vec3::default()),
		namegen: NameGen::default().enumerate(),
		chat_hash_handled: Vec::new(),
		task_queue: VecDeque::new(),
		per_inst: per_inst::PerInstanceTasks::default(),
	};
	let data = Arc::new(Mutex::new(data));
	let clients: Vec<Arc<Mutex<tokio::net::TcpStream>>> = Vec::new();
	let clients = Arc::new(Mutex::new(clients));

	let handle_chat = {
		let data = data.clone();
		async move |sender, content: String| {
			if let Some(sender) = sender {
				let mut data = data.lock().await;
				if data.owner == sender {
					println!("handling command {content}");

					let mut words = content.split(' ');

					match words.next() {
						Some("gang") => match words.next() {
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

								let to_add = (to_y..from_y + 1)
									.rev()
									.map(move |y| {
										(to_x..from_x + 1).map(move |x| {
											(to_z..from_z + 1)
												.map(move |z| Task::Mine(BlockPos { x, y, z }))
										})
									})
									.flatten()
									.flatten();

								let queue_taken = std::mem::take(&mut data.task_queue);
								data.task_queue = queue_taken.into_iter().chain(to_add).collect();

								println!("{:?}", data.task_queue);
							}
							_ => {}
						},
						_ => {}
					}
				}
			}

			anyhow::Ok(())
		}
	};
	let handle_chat = Arc::new(handle_chat);

	{
		let data = data.clone();
		tokio::spawn(async move {
			tokio::time::sleep(Duration::from_secs(15)).await;
			{
				let mut data = data.lock().await;
				data.chat_hash_handled.clear();
			}
		});
	}
	{
		let data = data.clone();
		let clients = clients.clone();
		// owner position check
		tokio::spawn(async move {
			let internal = async move || -> anyhow::Result<()> {
				loop {
					tokio::time::sleep(Duration::from_millis(300)).await;

					{
						let request = ClientboundPacket::Find {
							username: data.lock().await.owner.clone(),
						};

						for client in clients.lock().await.iter() {
							let mut client = client.lock().await;
							client.write_as_packet(&request).await?;

							// the response shouldn't be handled by the general request handler because it's locked even before
							// we send the request and we hold that lock till after their response has been read
							let resp: ServerboundPacket = client.read_as_packet().await?;
							match resp {
								ServerboundPacket::ReportPosition { username, report }
									if username == data.lock().await.owner =>
								{
									match report {
										PosReport::NotHere => continue,
										PosReport::Found(pos) => {
											let mut data = data.lock().await;
											data.owner_pos = (Instant::now(), pos);
											break;
										}
									}
								}
								_ => {
									eprintln!("whereis thread dropped non-report packet: {resp:?}")
								}
							}
						}
					}
				}
			};
			match internal().await {
				Ok(a) => a,
				Err(err) => {
					eprintln!("error in owner finding routine: {err}")
				}
			}
		});
	}
	{
		let data = data.clone();
		let clients = clients.clone();
		// request handler
		tokio::spawn(async move {
			loop {
				let (mut socket, addr) = match listener.accept().await {
					Ok(a) => a,
					Err(err) => {
						eprintln!("server failed to accept connection: {err}");
						continue;
					}
				};
				let mut hi = async || -> anyhow::Result<()> {
					let hello: ServerboundHelloPacket = socket.read_as_packet().await?;

					let name = {
						let mut data = data.lock().await;
						data.namegen.next()
					};
					let (i, name) = name.expect("namegen is never supposed to return none");
					println!("[{hello:?}] hello {i}: {name}");

					let hello_resp = ClientboundHelloPacket {
						name,
						inst_id: i as _,
					};
					socket.write_as_packet(&hello_resp).await?;
					Ok(())
				};
				match hi().await {
					Ok(a) => a,
					Err(err) => {
						eprintln!("error while exchanging Hello packets: {err}")
					}
				}

				let socket = Arc::new(Mutex::new(socket));
				{
					clients.lock().await.push(socket.clone());
				}

				let data = data.clone();
				let handle_chat = handle_chat.clone();
				tokio::spawn(async move {
					let internal = async || -> anyhow::Result<()> {
						loop {
							let packet: ServerboundPacket = {
								let mut socket = socket.lock().await;
								socket.read_as_packet().await?
							};

							match packet {
								ServerboundPacket::ChatMessage {
									hash,
									sender,
									content,
								} => {
									{
										let mut data = data.lock().await;
										if data.chat_hash_handled.contains(&hash) {
											// it's cool
											continue;
										} else {
											data.chat_hash_handled.push(hash);
										}
									}
									println!("{}: {content}", sender.clone().unwrap_or_default());

									handle_chat(sender, content).await?;
								}
								ServerboundPacket::Agro { uuid } => {
									let mut data = data.lock().await;
									data.per_inst.new_task_times(Task::Attack(uuid), 3);
								}
								ServerboundPacket::RequestTask { inst_id } => {
									let task = {
										let mut data = data.lock().await;

										if let Some(per_inst) = data.per_inst.task_for(inst_id) {
											per_inst
										} else {
											let from_queue = data.task_queue.pop_front();
											if let Some(from_queue) = from_queue {
												from_queue
											} else {
												let (time, pos) = data.owner_pos;
												if time.elapsed() < Duration::from_secs(30) {
													Task::Goto(RadiusGoal { pos, radius: 10.0 })
												} else {
													Task::Jump
												}
											}
										}
									};

									let mut socket = socket.lock().await;
									let response = ClientboundPacket::AssignTask(Some(task));
									socket.write_as_packet(response).await?;
								}
								ServerboundPacket::ReportPosition { .. } => {
									// handled elsewhere
								}
							}
						}
					};
					match internal().await {
						Ok(a) => a,
						Err(err) => {
							eprintln!("server error while handling {addr}: {err}");
							return;
						}
					}
				});
			}
		});

		println!("server listening on {}", super::ADDR);
		Ok(())
	}
}
