use std::{
	sync::Arc,
	time::{Duration, Instant},
};

use azalea::{Vec3, pathfinder::goals::RadiusGoal};
use tokio::{net::TcpListener, sync::Mutex};

use crate::tasks::{
	Task,
	net::{ClientboundPacket, ServerboundPacket},
};

use honeypack::{PacketRead, PacketWrite};

use super::PosReport;

#[derive(Debug)]
struct ServerData {
	owner: String,
	owner_pos: (Instant, Vec3),
}

/// functional baby
pub async fn start_server(owner: String) -> anyhow::Result<()> {
	let listener = TcpListener::bind(super::ADDR).await?;

	let data = ServerData {
		owner,
		owner_pos: (Instant::now() - Duration::from_hours(1), Vec3::default()),
	};
	let data = Arc::new(Mutex::new(data));
	let clients: Vec<Arc<Mutex<tokio::net::TcpStream>>> = Vec::new();
	let clients = Arc::new(Mutex::new(clients));
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
				let (socket, addr) = match listener.accept().await {
					Ok(a) => a,
					Err(err) => {
						eprintln!("server failed to accept connection: {err}");
						continue;
					}
				};

				let socket = Arc::new(Mutex::new(socket));
				{
					clients.lock().await.push(socket.clone());
				}

				let data = data.clone();
				tokio::spawn(async move {
					let internal = async || -> anyhow::Result<()> {
						loop {
							let packet: ServerboundPacket = {
								let mut socket = socket.lock().await;
								socket.read_as_packet().await?
							};

							match packet {
								ServerboundPacket::Hello { inst_id } => {
									println!("{inst_id} said hi");
								}
								ServerboundPacket::RequestTask { inst_id } => {
									let task = {
										let data = data.lock().await;
										let (time, pos) = data.owner_pos;
										if time.elapsed() < Duration::from_secs(30) {
											Task::Goto(RadiusGoal { pos, radius: 10.0 })
										} else {
											Task::Jump
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
