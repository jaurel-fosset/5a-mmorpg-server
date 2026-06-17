use std::collections::HashMap;
use bevy::prelude::*;
use game_sockets;
use crate::network::{NetworkUpdate, ServerListenSocket};

pub struct ClientHandlingPlugin;

impl Plugin for ClientHandlingPlugin
{
    fn build(&self, app: &mut App)
    {
        println!("ClientHandlingPlugin");
        app
            .insert_resource(ActiveClients::default())
            .insert_resource(ConnectingClients::default())
            .add_systems(NetworkUpdate, Self::create_stream)
            .add_systems(NetworkUpdate, Self::switch_client_to_active);
    }
}

impl ClientHandlingPlugin
{
    fn create_stream(mut server: ResMut<ServerListenSocket>, mut connecting_clients: ResMut<ConnectingClients>)
    {
        for (connection, client) in connecting_clients.clients.iter_mut()
        {
            if client.frame_stream.is_none()
            {
                if client.frame_stream_tries > 3
                {
                    error_once!("Could not create client frame stream, tried 3 times");
                    continue;
                }

                if let Err(_) = server.socket.create_stream(*connection, game_sockets::GameStreamReliability::Ordered)
                {
                    client.frame_stream_tries += 1;
                }
            } else if client.input_stream.is_none()
            {
                if client.input_stream_tries > 3
                {
                    error_once!("Could not create input stream, tried 3 times");
                    continue;
                }

                if let Err(_) = server.socket.create_stream(*connection, game_sockets::GameStreamReliability::Ordered)
                {
                    client.input_stream_tries += 1;
                }
            }
        }
    }

    fn switch_client_to_active(mut connecting_clients: ResMut<ConnectingClients>, mut active_clients: ResMut<ActiveClients>)
    {
        let mut free_list: Vec<game_sockets::GameConnection> = Vec::new();

        for (connection, client) in connecting_clients.clients.iter()
        {
            if client.frame_stream.is_none() || client.input_stream.is_none()
            {
                continue;
            }

            // TODO : spawn player
            active_clients.add_client(*connection, client.frame_stream.clone().unwrap(), client.input_stream.clone().unwrap());
            free_list.push(*connection);
        }

        for connection in free_list
        {
            connecting_clients.clients.remove(&connection);
        }
    }
}

#[derive(Resource)]
pub struct ActiveClients
{
    clients: HashMap<game_sockets::GameConnection, ClientNetworkState>,
}

impl ActiveClients
{
    pub fn num_clients(&self) -> u8
    {
        self.clients.len() as u8
    }

    pub fn add_client(&mut self, connection: game_sockets::GameConnection,
                      frame_stream: game_sockets::GameStream,
                      input_stream: game_sockets::GameStream)
    {
        self.clients.insert(connection, ClientNetworkState { frame_stream, input_stream });
    }
}

impl Default for ActiveClients
{
    fn default() -> Self
    {
        Self
        {
            clients: HashMap::new(),
        }
    }
}

struct ClientNetworkState
{
    input_stream: game_sockets::GameStream,
    frame_stream: game_sockets::GameStream,
}

#[derive(Resource)]
pub struct ConnectingClients
{
    clients: HashMap<game_sockets::GameConnection, ConnectingClientState>,
}

impl ConnectingClients
{
    pub fn add_client(&mut self, connection: game_sockets::GameConnection)
    {
        self.clients.insert(connection, ConnectingClientState::default());
    }

    pub fn set_stream(&mut self, connection: game_sockets::GameConnection, stream: game_sockets::GameStream)
    {
        let state = match self.clients.get_mut(&connection)
        {
            Some(state) => state,
            None => return,
        };

        if state.frame_stream.is_none()
        {
            state.frame_stream = Some(stream);
        }
        else if state.input_stream.is_none()
        {
            state.input_stream = Some(stream);
        }
    }
}

impl Default for ConnectingClients
{
    fn default() -> Self
    {
        ConnectingClients { clients: HashMap::new() }
    }
}

struct ConnectingClientState
{
    input_stream: Option<game_sockets::GameStream>,
    input_stream_tries: usize,
    frame_stream: Option<game_sockets::GameStream>,
    frame_stream_tries: usize,
}

impl Default for ConnectingClientState
{
    fn default() -> Self
    {
        Self
        {
            input_stream: None,
            input_stream_tries: 0,
            frame_stream: None,
            frame_stream_tries: 0,
        }
    }
}
