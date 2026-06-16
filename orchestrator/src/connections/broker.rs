use std::collections::HashMap;
use bollard::config::{ContainerCreateBody, HostConfig};
use bollard::query_parameters::CreateContainerOptions;
use game_sockets as gs;
use crate::connections::{get_docker_ip, init_connection};
use tokio::sync;
use network_serialization::packet::{PacketData, PacketMessage};
use network_serialization::packets::broker::BroadcastPacket;
use network_serialization::packets::Packet;
use network_serialization::packets::topic::TopicTree;

const BROKER_PORT: u16 = 10_001;
const EVENTS_BUFFER_SIZE: usize = 20;

#[repr(u8)]
#[derive(int_enum::IntEnum, Debug, Eq, PartialEq)]
enum ServerType
{
    Orchestrator = 0,
    Broker,
    Spatial,
    Shard,
}

pub enum Commands
{
    ServerCreation(u32, ServerType),
    ServerDestruction(u32, ServerType),
}


struct BrokerTask
{
    address: String,
    port: u16,
    command_receiver: sync::mpsc::Receiver<Commands>,
    command_sender: sync::mpsc::Sender<Commands>,
}

impl BrokerTask
{
    pub async fn new(docker: &mut bollard::Docker) -> Self
    {
        const BROKER_PORT: u16 = 10_001;

        let container_name = "broker-service";

        let config = ContainerCreateBody
        {
            image: Some(String::from("broker")),
            host_config: Some(HostConfig
            {
                port_bindings: Some(
                {
                    HashMap::from
                    ([
                        (
                            format!("{}/udp", BROKER_PORT),
                            Some(vec![bollard::models::PortBinding
                            {
                                host_ip: Some("0.0.0.0".to_string()),
                                host_port: Some(BROKER_PORT.to_string()),
                            }])
                        ),
                    ])
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

        let (event_sender, event_receiver) = sync::mpsc::channel(EVENTS_BUFFER_SIZE);

        Self
        {
            address: ip,
            port: BROKER_PORT,
            command_receiver: event_receiver,
            command_sender: event_sender,
        }
    }

    fn get_command_channel_handle(&self) -> sync::mpsc::Sender<Commands>
    {
        self.command_sender.clone()
    }

    async fn run(mut self)
    {
        let (socket, connection, stream) = init_connection(self.address, self.port).await;

        while let Some(event) = self.command_receiver.recv().await
        {
            match event
            {
                Commands::ServerDestruction(id, server_type) =>
                {
                    Self::publish_server_destruction(&socket, &connection, &stream, id, server_type);
                }
                Commands::ServerCreation(id, server_type) =>
                {
                    Self::publish_server_creation(&socket, &connection, &stream, id, server_type);
                }
            }
        }
    }

    fn publish_server_creation(socket: &gs::GamePeer, connection: &gs::GameConnection, stream: &gs::GameStream, id: u32, server_type: ServerType)
    {
        let mut server_creation = TopicTree::new_empty("server_creation".to_string());
        server_creation.add_leaf(format!("{}", id), vec![server_type.into()]);

        let mut orchestrator = TopicTree::new_empty("orchestrator".to_string());
        orchestrator.add_tree(server_creation);


        let packet = PacketMessage::new
        (
            PacketData::Broadcast
            (
                BroadcastPacket
                {
                    topic: orchestrator,
                }
            )
        ).write().unwrap();

        socket.send(connection, stream, packet).unwrap();
    }

    fn publish_server_destruction(socket: &gs::GamePeer, connection: &gs::GameConnection, stream: &gs::GameStream, id: u32, server_type: ServerType)
    {
        let mut server_destruction = TopicTree::new_empty("server_destruction".to_string());
        server_destruction.add_leaf(format!("{}", id), vec![server_type.into()]);

        let mut orchestrator = TopicTree::new_empty("orchestrator".to_string());
        orchestrator.add_tree(server_destruction);

        let packet = PacketMessage::new
        (
            PacketData::Broadcast
            (
                BroadcastPacket
                {
                    topic: orchestrator,
                }
            )
        ).write().unwrap();

        socket.send(connection, stream, packet).unwrap();
    }
}