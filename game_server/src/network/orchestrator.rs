use bevy::prelude::*;
use game_sockets::GameStreamReliability;
use crate::network::ServerListenSocket;

pub struct OrchestratorHandlingPlugin;

impl Plugin for OrchestratorHandlingPlugin
{
    fn build(&self, app: &mut App)
    {
        HeartbeatStreamFactory::insert(app);
    }
}

#[derive(Resource)]
pub struct Orchestrator
{
    connection: game_sockets::GameConnection,
    state: OrchestratorNetworkState,
}

impl Orchestrator
{
    pub fn new(connection: game_sockets::GameConnection, command_stream: game_sockets::GameStream) -> Self
    {
        Self
        {
            connection,
            state: OrchestratorNetworkState
            {
                heartbeat_stream: None,
                heartbeat_stream_tries: 0,
                command_stream,
            },
        }
    }

    pub fn register_heartbeat_stream(&mut self, stream: game_sockets::GameStream)
    {
        self.state.heartbeat_stream.get_or_insert(stream);
    }
    
    pub fn is_orchestrator_connection(&self, connection: game_sockets::GameConnection) -> bool
    {
        self.connection == connection
    }
    
    pub fn get_heartbeat_stream(&self) -> Option<&game_sockets::GameStream>
    {
        self.state.heartbeat_stream.as_ref()
    }
    
    pub fn get_connection(&self) -> &game_sockets::GameConnection
    {
        &self.connection
    }
}

struct OrchestratorNetworkState
{
    heartbeat_stream: Option<game_sockets::GameStream>,
    heartbeat_stream_tries: usize,
    command_stream: game_sockets::GameStream,
}

#[derive(Resource)]
pub struct HeartbeatStreamFactory
{
    system_id: bevy::ecs::system::SystemId,
}

impl HeartbeatStreamFactory
{
    pub fn insert(app: &mut App)
    {
        let system_id = app.register_system(Self::try_create_heartbeat_stream);

        let factory = HeartbeatStreamFactory
        {
            system_id,
        };

        app.insert_resource(factory);
    }
    pub fn create_heartbeat_stream(&self, commands: &mut Commands)
    {
        commands.run_system(self.system_id)
    }

    fn try_create_heartbeat_stream(mut server: ResMut<ServerListenSocket>, one_offs: Res<HeartbeatStreamFactory>, orchestrator: Option<ResMut<Orchestrator>>, mut commands: Commands)
    {
        let mut orchestrator = match orchestrator
        {
            Some(orchestrator) => orchestrator,
            None => return,
        };

        if let Err(_) = server.socket.create_stream(orchestrator.connection, GameStreamReliability::Unreliable)
        {
            if orchestrator.state.heartbeat_stream_tries > 3
            {
                error!("Could not create heartbeat stream, tried 3 times");
                return;
            }

            orchestrator.state.heartbeat_stream_tries += 1;
            commands.run_system(one_offs.system_id);
        }
    }
}