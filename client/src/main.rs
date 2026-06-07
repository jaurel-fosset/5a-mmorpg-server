use std::time::{Duration, Instant};
use bevy::prelude::*;
use bevy_egui::egui::{Align2, Context};
use bevy_egui::{EguiContexts, EguiPlugin, EguiPrimaryContextPass, egui};
use game_sockets::{GameConnection, GameNetworkEvent, GameStream, GameStreamReliability};
use network_serialization::packet::{PacketData, PacketMessage};
use network_serialization::packets::broker::ClientHelloPacket;
use network_serialization::packets::Packet;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(EguiPlugin::default())
        .init_resource::<ConnectionSettings>()
        .add_systems(Startup, setup_camera_system)
        .add_systems(EguiPrimaryContextPass, ui_example_system)
        .run();
}

#[derive(Resource)]
struct ConnectionSettings {
    ip_address: String,
    ip_port: String,
    is_connected: bool,
}

fn setup_camera_system(mut commands: Commands) {
    commands.spawn(Camera2d);
}

impl Default for ConnectionSettings {
    fn default() -> Self {
        Self {
            ip_address: "127.0.0.1".to_string(),
            ip_port: "12345".to_string(),
            is_connected: false,
        }
    }
}

fn ui_example_system(
    mut context: EguiContexts,
    mut settings: ResMut<ConnectionSettings>,
) -> Result {
    if settings.is_connected {
        return Ok(());
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
                    connect_to_server(settings);
                };
            }
        );
    Ok(())
}

fn connect_to_server(
    mut connection_settings: ResMut<ConnectionSettings>,
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
                        connection_settings.is_connected = true;
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

    if connection_settings.is_connected {
        println!("Connected to server, close window");
    }
}