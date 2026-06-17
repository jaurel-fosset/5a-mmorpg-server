use std::net;
use std::ops::{Deref, DerefMut};
use std::time::Duration;
use bevy::diagnostic::{DiagnosticsStore, SystemInfo, SystemInformationDiagnosticsPlugin};
use bevy::prelude::*;
use game_sockets::GameStreamReliability;
use network_serialization;
use network_serialization::packet::{PacketData, PacketMessage};
use network_serialization::packets::game_server::HeartbeatPacket;
use network_serialization::packets::Packet;
use crate::env_parameter::Environment;
use crate::network::client::ActiveClients;
use crate::network::orchestrator::Orchestrator;
use crate::network::{NetworkUpdate, ServerListenSocket};

pub struct HeartbeatNetworkPlugin;

impl Plugin for HeartbeatNetworkPlugin
{
    fn build(&self, app: &mut App)
    {
        app
            .insert_resource(HeartbeatTimer::default())
            .add_systems(NetworkUpdate, Self::send_heartbeat);
    }
}

impl HeartbeatNetworkPlugin
{
    fn send_heartbeat(environment: Res<Environment>, server: Res<ServerListenSocket>, clients: Res<ActiveClients>, diagnostics: Res<DiagnosticsStore>,
                      mut timer: ResMut<HeartbeatTimer>, time: Res<Time>, orchestrator: Option<Res<Orchestrator>>)
    {
        timer.tick(time.delta());
        if !timer.is_finished()
        {
            return;
        }

        let (orchestrator, heartbeat_stream) = match &orchestrator
        {
            Some(orchestrator) =>
            {
                let connection = orchestrator.get_connection();
                match orchestrator.get_heartbeat_stream()
                {
                    Some(stream) => (connection, stream),
                    None => return,
                }
            },
            None => return,
        };

        let cpu_load_diagnostic = diagnostics.get(&SystemInformationDiagnosticsPlugin::SYSTEM_CPU_USAGE).unwrap();
        let ram_load_diagnostic = diagnostics.get(&SystemInformationDiagnosticsPlugin::SYSTEM_MEM_USAGE).unwrap();

        let cpu_load = cpu_load_diagnostic.smoothed().unwrap_or(0.0);
        let cpu_load = (cpu_load * 1000.0).trunc();
        let cpu_load = (cpu_load * 255.0) / 1000.0;

        let ram_load = ram_load_diagnostic.smoothed().unwrap_or(0.0);
        let ram_load = (ram_load * 1000.0).trunc();
        let ram_load = (ram_load * 255.0) / 1000.0;
        
        let packet = PacketMessage::new
        (
            PacketData::Heartbeat
            (
                HeartbeatPacket
                {
                    ip: environment.ip.to_string().parse().unwrap(),
                    port: environment.port,
                    player_number: clients.num_clients(),
                    player_capacity: environment.player_capacity,
                    cpu_load: cpu_load as u8,
                    ram_load: ram_load as u8,
                }
            )
        ).write();
        let packet = match packet
        {
            Ok(packet) => packet,
            Err(_) =>
            {
                info!("Failed to write heartbeat packet");
                return;
            },
        };

        trace!("Sending heartbeat");

        if let Err(error) = server.socket.send(&orchestrator, heartbeat_stream, packet)
        {
            error!("Error sending heartbeat: {}", error);
        }
    }
}


#[derive(Resource)]
struct HeartbeatTimer(Timer);

impl Default for HeartbeatTimer
{
    fn default() -> Self
    {
        Self(Timer::from_seconds(5.0, TimerMode::Repeating))
    }
}

impl Deref for HeartbeatTimer
{
    type Target = Timer;
    fn deref(&self) -> &Self::Target
    {
        &self.0
    }
}

impl DerefMut for HeartbeatTimer
{
    fn deref_mut(&mut self) -> &mut Self::Target
    {
        &mut self.0
    }
}