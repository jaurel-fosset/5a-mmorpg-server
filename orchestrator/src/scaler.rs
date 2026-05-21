use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use bollard::config::ContainerCreateBody;
use bollard::Docker;
use bollard::models::HostConfig;
use bollard::query_parameters::{CreateContainerOptions, StartContainerOptions};
use crate::AppState;

pub async fn run(state: Arc<AppState>) {
    let mut interval = tokio::time::interval(
        Duration::from_secs(10)
    );

    let mut game_server_port :u16 = 7001;
    let mut con = state.redis_connexion.clone();

    loop {
        interval.tick().await;
        println!("Starting scaler");
        let list_server : Vec<String> = redis::cmd("ZRANGE")
            .arg(&["servers:available", "0", "-1"])
            .query_async(&mut con)
            .await
            .unwrap();

        let mut number_server = list_server.len();

        for server in &list_server {
            let info_server : Vec<String> = redis::cmd("HGETALL")
                .arg(server)
                .query_async(&mut con)
                .await
                .unwrap();

            if info_server.is_empty() {
                redis::cmd("ZREM")
                    .arg(&["servers:available", server])
                    .exec_async(&mut con)
                    .await
                    .unwrap();
                number_server = number_server - 1;
            }
            println!("{}", server);
        }

        if number_server < 3 {
            println!("Not enough servers. Start one.");
            spawn_game_server(state.clone(),game_server_port).await;
            game_server_port +=1;
        }
    }
}

pub async fn spawn_game_server(
    state: Arc<AppState>,
    port: u16
) {
    let docker = Docker::connect_with_socket_defaults().unwrap();

    let container_name = format!("game_server-{}", port);

    let config = ContainerCreateBody {
        image: Some(String::from("mmorpg-server-game_server")),
        env: Some(vec![
            String::from("IP=0.0.0.0"),
            String::from(&format!("PORT={}",port)),
            String::from("PLAYER_CAPACITY=120")
        ]),
        host_config: Some(HostConfig {
            port_bindings: Some({
                let mut map = HashMap::new();
                map.insert(
                    format!("{}/udp", port),
                    Some(vec![bollard::models::PortBinding {
                        host_ip: Some("0.0.0.0".to_string()),
                        host_port: Some(port.to_string()),
                    }]),
                );
                map
            }),
            network_mode: Some("mmorpg-server_default".to_string()),
            ..Default::default()
        }),
        ..Default::default()
    };

    docker.create_container(
        Some(CreateContainerOptions {
            name: Some(container_name.clone()),
            platform: String::new(),
        }),
        config,
    ).await.unwrap();

    docker.start_container(
        &container_name,
        None::<StartContainerOptions>,
    ).await.unwrap();

    tokio::time::sleep(Duration::from_secs(2)).await;

    // Récupère l'IP du conteneur
    let inspect = docker.inspect_container(&container_name, None).await.unwrap();
    let ip = inspect
        .network_settings
        .unwrap()
        .networks
        .unwrap()
        .get("mmorpg-server_default")
        .unwrap()
        .ip_address
        .clone()
        .unwrap();

    println!("GameServer IP: {}", ip);

    // Connexion via game_sockets
    let mut peer = state.peer.lock().await;
    peer.connect(&ip, port).unwrap();

    // Récupère le Connected local
    let conn: game_sockets::GameConnection = loop {
        if let Ok(Some(game_sockets::GameNetworkEvent::Connected(conn))) = peer.poll() {
            break conn;
        }
        tokio::time::sleep(Duration::from_millis(10)).await;
    };

    // Crée un stream
    peer.create_stream(conn.clone(), game_sockets::GameStreamReliability::Unreliable).unwrap();
    let stream: game_sockets::GameStream = loop {
        if let Ok(Some(game_sockets::GameNetworkEvent::StreamCreated(_, stream))) = peer.poll() {
            break stream;
        }
        tokio::time::sleep(Duration::from_millis(10)).await;
    };

    // Envoie le handshake et c'est tout
    peer.send(&conn, &stream, bytes::Bytes::from("hello")).unwrap();
    println!("Handshake sent to game_server-{}", port);
}