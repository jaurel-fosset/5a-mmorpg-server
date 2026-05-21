pub mod client;
pub mod orchestrator;
mod heartbeat;

use bevy::prelude::*;
use bevy::ecs::schedule::ScheduleLabel;
use game_sockets;

use crate::env_parameter::Environment;
use crate::network::client::{ClientHandlingPlugin, ConnectingClients};
use crate::network::heartbeat::HeartbeatNetworkPlugin;
use crate::network::orchestrator::{HeartbeatStreamFactory, OrchestratorHandlingPlugin};

use crate::schedule_handling;

pub struct NetworkPluginGroup;


#[derive(ScheduleLabel, Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub struct PreNetworkUpdate;
#[derive(ScheduleLabel, Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub struct NetworkUpdate;
#[derive(ScheduleLabel, Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub struct PostNetworkUpdate;

impl NetworkPluginGroup
{
    fn listen(environment: Res<Environment>, server: Res<ServerListenSocket>)
    {
        if let Err(error) = server.socket.listen(environment.ip.as_str(), environment.port)
        {
            panic!("{}", error);
        }
    }

    fn get_packets(mut server: ResMut<ServerListenSocket>, mut commands: Commands, mut orchestrator: Option<ResMut<orchestrator::Orchestrator>>,
                   mut clients: ResMut<ConnectingClients>,
                   heartbeat_stream_factory: Res<HeartbeatStreamFactory>)
    {
        while let Some(event) = server.socket.poll().transpose()
        {
            let event = match event
            {
                Ok(event) => event,
                Err(error) =>
                {
                    error!("Error with a received packet : {}", error);
                    continue;
                },
            };

            match event
            {
                game_sockets::GameNetworkEvent::Connected(connection) =>
                {
                    if orchestrator.is_none()
                    {
                        continue;
                    }

                    clients.add_client(connection);
                }
                game_sockets::GameNetworkEvent::Disconnected(_) => {}
                game_sockets::GameNetworkEvent::Message { .. } =>
                {

                }
                game_sockets::GameNetworkEvent::Error { .. } => {}
                game_sockets::GameNetworkEvent::StreamCreated(connection, stream) =>
                {
                    let orchestrator = match &mut orchestrator
                    {
                        Some(orchestrator) => orchestrator,
                        None =>
                        {
                            commands.insert_resource(orchestrator::Orchestrator::new(connection, stream));
                            heartbeat_stream_factory.create_heartbeat_stream(&mut commands);
                            return;
                        }
                    };

                    if orchestrator.is_orchestrator_connection(connection)
                    {
                        orchestrator.register_heartbeat_stream(stream);
                        continue;
                    }

                    clients.set_stream(connection, stream);
                }
                game_sockets::GameNetworkEvent::StreamClosed(_, _) => {}
            }
        }
    }
}

impl Plugin for NetworkPluginGroup
{
    fn build(&self, app: &mut App)
    {
        schedule_handling::ScheduleFactory::register(app, NetworkUpdate).after(PostUpdate);
        
        schedule_handling::ScheduleFactory::register(app, PreNetworkUpdate).before(NetworkUpdate);
        schedule_handling::ScheduleFactory::register(app, PostNetworkUpdate).after(NetworkUpdate);

        app
            .add_plugins(OrchestratorHandlingPlugin)
            .add_plugins(ClientHandlingPlugin)
            .add_plugins(HeartbeatNetworkPlugin)
            .add_systems(Startup, Self::listen)
            .add_systems(NetworkUpdate, Self::get_packets)
            .insert_resource(ServerListenSocket::default());
    }
}


#[derive(Resource)]
struct ServerListenSocket
{
    socket: game_sockets::GamePeer,
}

impl Default for ServerListenSocket
{
    fn default() -> Self {
        let backend = game_sockets::protocols::QuicBackend::new();
        let socket = game_sockets::GamePeer::new(backend);

        Self { socket }
    }
}






