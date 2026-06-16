use std::collections::HashMap;
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
    next_id: u32,
}

impl ShardsTask
{
    pub async fn new(broker_commands: sync::mpsc::Sender<broker::Commands>,
                     spatial_events: sync::broadcast::Receiver<spatial::Events>) -> Self
    {
        Self
        {
            broker_commands,
            spatial_events,
            next_id: 1000,
        }
    }

    pub async fn run(mut self)
    {
        loop
        {
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

            task::yield_now().await;
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
            let (ip, port, id) = loop
            {
                match self.spawn_single_shard(&mut docker).await
                {
                    Ok(shard) => break shard,
                    Err(error) => continue,
                }
            };

            init_connection(&ip, port).await;

            // TODO : add to redis
            _ = self.broker_commands.send(broker::Commands::ServerCreation(id, broker::ServerType::Shard)).await;
        }
    }

    async fn spawn_single_shard(&mut self, docker: &mut Docker) -> Result<(String, u16, u32), SpawnShardError>
    {
        let shard_id = self.next_id();
        let container_name = format!("shard-{}", shard_id);

        let config = ContainerCreateBody
        {
            image: Some(String::from("mmorpg-server-game_server")),
            env: Some(vec![
                String::from("IP=0.0.0.0"),
                String::from(&format!("PORT={}", SHARD_PORT)),
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
                            host_port: Some(SHARD_PORT.to_string()),
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

        Ok((get_docker_ip(docker, &response.id).await, SHARD_PORT, shard_id))
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
