use std::collections::HashMap;
use bollard::config::{ContainerCreateBody, HostConfig};
use bollard::query_parameters::CreateContainerOptions;
use tokio::{sync, task};
use log::error;
use game_sockets::GameNetworkEvent;
use network_serialization::packet::{PacketData, PacketMessage};
use network_serialization::packets::Packet;
use crate::connections::{get_docker_ip, init_connection};


const EVENTS_BUFFER_SIZE: usize = 20;

#[derive(Debug, Copy, Clone)]
pub enum Events
{
    ShardCreationRequest(u64),
}

pub struct SpatialTask
{
    address: String,
    port: u16,
    event_receiver: sync::broadcast::Receiver<Events>,
    event_sender: sync::broadcast::Sender<Events>,
}

impl SpatialTask
{
    pub async fn new(docker: &mut bollard::Docker) -> SpatialTask
    {
        let container_name = "spatial-service";

        const ORCHESTRATOR_PORT: u16 = 50_000;
        const BROKER_PORT: u16 = 10_001;

        let config = ContainerCreateBody
        {
            image: Some(String::from("spatial")),
            host_config: Some(HostConfig
            {
                port_bindings: Some(
                {
                    let mut map = HashMap::new();

                    map.insert
                    (
                        format!("{}/udp", BROKER_PORT),
                        Some(vec![bollard::models::PortBinding
                        {
                            host_ip: Some("0.0.0.0".to_string()),
                            host_port: Some(BROKER_PORT.to_string()),
                        }]),
                    );
                    map.insert(
                        format!("{}/udp", ORCHESTRATOR_PORT),
                        Some(vec![bollard::models::PortBinding
                        {
                            host_ip: Some("0.0.0.0".to_string()),
                            host_port: Some(ORCHESTRATOR_PORT.to_string()),
                        }])
                    );

                    map
                }),
                network_mode: Some("mmorpg-server_default".to_string()),
                ..Default::default()
            }),
            ..Default::default()
        };

        let response = docker.create_container(
            Some(CreateContainerOptions {
                name: Some(container_name.to_string()),
                platform: String::new(),
            }),
            config,
        ).await.unwrap();

        let ip = get_docker_ip(docker, &response.id).await;
        let (event_sender, event_receiver) = sync::broadcast::channel(EVENTS_BUFFER_SIZE);

        Self
        {
            address: ip,
            port: ORCHESTRATOR_PORT,
            event_receiver,
            event_sender,
        }
    }
    
    pub fn get_events_handle(&self) -> sync::broadcast::Receiver<Events>
    {
        self.event_sender.subscribe()
    }

    pub async fn run(self)
    {
        let (mut socket, spatial_connection, spatial_stream) = init_connection(&self.address, self.port).await;

        // TODO : add to redis

        loop
        {
            while let Some(connection_event) = socket.poll().transpose()
            {
                let connection_event = match connection_event
                {
                    Ok(event) => event,
                    Err(error) =>
                        {
                            error!("Error while polling spatial connection: {}", error);
                            continue;
                        }
                };

                match connection_event
                {
                    GameNetworkEvent::Connected(_) => {}
                    GameNetworkEvent::Disconnected(_) => {}
                    GameNetworkEvent::Message { connection, stream, data } =>
                        {
                            if connection != spatial_connection { continue; }
                            if stream != spatial_stream { continue; }

                            let packet = match PacketMessage::read(data)
                            {
                                Ok(packet) => packet,
                                Err(error) =>
                                    {
                                        error!("[Spatial] Ill formed packet received");
                                        continue;
                                    }
                            };

                            match packet.data
                            {
                                PacketData::AllocateShards(allocate_shards) =>
                                    {
                                        match self.event_sender.send(Events::ShardCreationRequest(allocate_shards.shard_count()))
                                        {
                                            Ok(_) => {}
                                            Err(error) =>
                                                {
                                                    log::info!("[Spatial] No more listener for shard creation, continuing anyway");
                                                }
                                        }
                                    }
                                _ => (),
                            }
                        }
                    GameNetworkEvent::Error { .. } => {}
                    GameNetworkEvent::StreamCreated(_, _) => {}
                    GameNetworkEvent::StreamClosed(_, _) => {}
                }
                task::yield_now().await;
            }
            task::yield_now().await;
        }
    }
}