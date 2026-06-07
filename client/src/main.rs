use std::time::{Duration, Instant};
use bevy::prelude::*;
use bevy_egui::egui::{Align2, Context};
use bevy_egui::{EguiContexts, EguiPlugin, EguiPrimaryContextPass, egui};
use game_sockets::{GameConnection, GameNetworkEvent, GamePeer, GameStream, GameStreamReliability};
use network_serialization::packet::{PacketData, PacketMessage};
use network_serialization::packets::broker::ClientHelloPacket;
use network_serialization::packets::Packet;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(EguiPlugin::default())

        .init_resource::<ConnectionSettings>()
        .init_resource::<NetworkState>()
        .init_resource::<PlayerInput>()

        .add_systems(Startup, setup_camera_system)
        .add_systems(Update, (handle_input_system, send_inputs_to_network_system))
        .add_systems(EguiPrimaryContextPass, ui_example_system)
        .run();
}

#[derive(Resource)]
struct ConnectionSettings {
    ip_address: String,
    ip_port: String,
}

fn setup_camera_system(mut commands: Commands) {
    commands.spawn(Camera2d);
}

impl Default for ConnectionSettings {
    fn default() -> Self {
        Self {
            ip_address: "127.0.0.1".to_string(),
            ip_port: "12345".to_string(),
        }
    }
}

#[derive(Resource, Default)]
enum NetworkState {
    #[default]
    Disconnected,
    Connected {
        connection: GameConnection,
        peer: GamePeer,
        stream: GameStream,
    },
}


use bitflags::bitflags;

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
    pub struct DirectionFlags: u8 {
        const UP    = 0b0000_0001;
        const DOWN  = 0b0000_0010;
        const LEFT  = 0b0000_0100;
        const RIGHT = 0b0000_1000;
    }
}

#[derive(Resource, Default)]
pub struct PlayerInput {
    pub flags: DirectionFlags,
}

fn ui_example_system(
    mut context: EguiContexts,
    mut settings: ResMut<ConnectionSettings>,
    network_state: ResMut<NetworkState>,
) -> Result {
    match *network_state {
        NetworkState::Connected { .. } => {return Ok(())}
        _ => {}
    }

    egui::Window::new("Fenêtre de connexion")
        .anchor(Align2::CENTER_CENTER, [0.0, 0.0])
        .collapsible(false)
        .resizable(false)
        .show(
            context.ctx_mut()?,
            |ui| {
                ui.label("IP");
                ui.text_edit_singleline(&mut settings.ip_address);
                ui.add_space(3.0);

                ui.label("Port");
                ui.text_edit_singleline(&mut settings.ip_port);
                ui.add_space(10.0);

                if ui.button("Connect to server").clicked() {
                    println!("{:?}", settings.ip_address);
                    connect_to_server(settings,network_state);
                };
            }
        );
    Ok(())
}

fn connect_to_server(
    connection_settings: ResMut<ConnectionSettings>,
    mut network_state: ResMut<NetworkState>,
)  {
    let ip = connection_settings.ip_address.clone();
    let port = connection_settings.ip_port.clone();

    let backend = game_sockets::protocols::QuicBackend::new();
    let mut peer = game_sockets::GamePeer::new(backend);

    peer.connect(
        &*ip,
        port.parse::<u16>().unwrap()
    ).unwrap();

    let conn: GameConnection = loop {
        if let Ok(Some(GameNetworkEvent::Connected(conn))) = peer.poll() {
            println!("Connected! {:?}", conn);
            break conn;
        }
        std::thread::sleep(Duration::from_millis(10));
    };

    peer.create_stream(conn.clone(), GameStreamReliability::Unreliable).unwrap();
    let stream: GameStream = loop {
        if let Ok(Some(GameNetworkEvent::StreamCreated(_, stream))) = peer.poll() {
            break stream;
        }
        std::thread::sleep(Duration::from_millis(10));
    };

    let data = PacketData::ClientHello(ClientHelloPacket {});
    let packet = PacketMessage::new(data);

    peer.send(&conn, &stream, packet.write().unwrap()).unwrap();
    println!("Packet sent, waiting for response...");

    let timeout = Instant::now();
    loop {
        if timeout.elapsed() > Duration::from_secs(15) {
            println!("Timeout — no response");
            break;
        }

        match peer.poll() {
            Ok(Some(GameNetworkEvent::Message { connection, stream, data })) => {
                println!("Response from {:?}: {:?}", connection, data);
                let msg = PacketMessage::read(data).unwrap();

                match msg.data {
                    PacketData::ClientHandshake(_) => {
                        *network_state = NetworkState::Connected { connection,peer,stream };
                        break;
                    }
                    _ => println!("Unexpected packet type"),
                }

            }
            Ok(Some(e)) => println!("Event: {:?}", e),
            Ok(None) => std::thread::sleep(Duration::from_millis(10)),
            Err(e) => {
                println!("Error: {}", e);
                break;
            }
        }
    }

    let NetworkState::Connected { .. } = &mut *network_state else { return; };
    println!("Connected to server, close window");
}

fn handle_input_system(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut player_input: ResMut<PlayerInput>
) {
    let mut inputs = DirectionFlags::empty();
    if keyboard.pressed(KeyCode::ArrowUp) { inputs.insert(DirectionFlags::UP); }
    if keyboard.pressed(KeyCode::ArrowDown) { inputs.insert(DirectionFlags::DOWN); }
    if keyboard.pressed(KeyCode::ArrowLeft) { inputs.insert(DirectionFlags::LEFT); }
    if keyboard.pressed(KeyCode::ArrowRight) { inputs.insert(DirectionFlags::RIGHT); }

    player_input.flags = inputs;
}

fn send_inputs_to_network_system(
    player_input: Res<PlayerInput>,
    network_state: Res<NetworkState>,
) {
    let NetworkState::Connected{ connection, ref peer, ref stream } = *network_state else { return; };
    let byte_to_send: u8 = player_input.flags.bits();

    if byte_to_send != 0 {
        println!("Envoi au serveur : {:08b}", byte_to_send);
    }

}