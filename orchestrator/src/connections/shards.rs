use std::collections::HashMap;
use std::net::Ipv4Addr;
use std::time::{Duration, Instant};
use bollard::config::{ContainerCreateBody, HostConfig};
use bollard::Docker;
use bollard::query_parameters::{CreateContainerOptions, StartContainerOptions};
use tokio::{sync, task};
use tokio::sync::broadcast::error::RecvError;
use crate::connections::spatial::Events;
use super::{get_docker_ip, init_connection, spatial};
use super::broker;


const SHARD_PORT: u16 = 10_002;

pub struct ShardsTask
{
    broker_commands: sync::mpsc::Sender<broker::Commands>,
    spatial_events: sync::broadcast::Receiver<spatial::Events>,
    orchestrator_ip: Ipv4Addr,
    redis_dns_ip: Ipv4Addr,
    broker_ip: Ipv4Addr,
    next_id: u32,
}

impl ShardsTask
{
    pub async fn new(broker_commands: sync::mpsc::Sender<broker::Commands>,
                     spatial_events: sync::broadcast::Receiver<spatial::Events>,
                     orchestrator_ip: Ipv4Addr,
                     redis_dns_ip: Ipv4Addr,
                     broker_ip: Ipv4Addr) -> Self
    {
        Self
        {
            broker_commands,
            spatial_events,
            orchestrator_ip,
            redis_dns_ip,
            broker_ip,
            next_id: 1000,
        }
    }

    pub async fn run(mut self)
    {
        let tick_duration = Duration::from_millis(66);
        loop
        {
            let start_time = Instant::now();


            let spatial_events = match self.spatial_events.recv().await
            {
                Ok(spatial_events) => spatial_events,
                Err(error) =>
                {
                    match error
                    {
                        RecvError::Closed => break,
                        RecvError::Lagged(_) => continue,
                    }
                },
            };

            match spatial_events
            {
                Events::ShardCreationRequest(shard_nb) =>
                {
                    self.spawn_shards(shard_nb).await;
                }
            }

            let work_duration = start_time.elapsed();
            if let Some(sleep_duration) = tick_duration.checked_sub(work_duration) {
                tokio::time::sleep(sleep_duration).await;
            } else {
                println!("LAG: work took {}ms", work_duration.as_millis());
            }
        }
    }

    fn next_id(&mut self) -> u32
    {
        self.next_id += 1;
        self.next_id
    }

    async fn spawn_shards(&mut self, shard_number: u64)
    {
        let mut docker = Docker::connect_with_socket_defaults().unwrap();
        
        for _ in 0..shard_number
        {
            println!("spawning shard");
            let (ip, port, id) = loop
            {
                match self.spawn_single_shard(&mut docker).await
                {
                    Ok(shard) => break shard,
                    Err(error) => {
                        eprintln!("Failed to spawn shard: {}", error);
                        continue
                    },
                }
            };

            init_connection(ip, port, self.orchestrator_ip, self.redis_dns_ip, self.broker_ip).await;

            // TODO : add to redis
            _ = self.broker_commands.send(broker::Commands::ServerCreation(id, broker::ServerType::Shard)).await;
        }
    }

    async fn spawn_single_shard(&mut self, docker: &mut Docker) -> Result<(Ipv4Addr, u16, u32), SpawnShardError>
    {
        let shard_id = self.next_id();
        let container_name = format!("shard-{}", shard_id);

        let config = ContainerCreateBody
        {
            image: Some(String::from("game_server")),
            env: Some(vec![
                String::from("IP=0.0.0.0"),
                String::from(&format!("PORT={}", SHARD_PORT + (shard_id as u16))),
                String::from("PLAYER_CAPACITY=120")
            ]),
            host_config: Some(HostConfig
            {
                port_bindings: Some(
                {
                    let mut map = HashMap::new();
                    map.insert
                    (
                        format!("{}/udp", SHARD_PORT),
                        Some(vec![bollard::models::PortBinding
                        {
                            host_ip: Some("0.0.0.0".to_string()),
                            host_port: Some((SHARD_PORT + (shard_id as u16)).to_string()),
                        }]),
                    );
                    map
                }),
                network_mode: Some("mmorpg-server_default".to_string()),
                ..Default::default()
            }),
            ..Default::default()
        };

        let response = docker.create_container
        (
            Some(CreateContainerOptions
            {
                name: Some(container_name.clone()),
                platform: String::new(),
            }),
            config,
        ).await.map_err(|_| SpawnShardError::CouldNotCreateContainer)?;

        docker.start_container
        (
            &container_name,
            None::<StartContainerOptions>,
        ).await.map_err(|_| SpawnShardError::CouldNotCreateContainer)?;

        Ok((get_docker_ip(docker, &response.id).await, SHARD_PORT+(shard_id as u16), shard_id))
    }
}

#[derive(Debug, thiserror::Error)]
enum SpawnShardError
{
    #[error("Could not connect to Docker")]
    CouldNotConnectToDocker,
    #[error("Could not create container")]
    CouldNotCreateContainer,
}
