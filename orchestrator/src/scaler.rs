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

        // TODO : change if for while
        if number_server < 1 {
            println!("Not enough servers. Start one.");
            spawn_game_server(game_server_port).await;
            game_server_port +=1;
        }
    }
}

pub async fn spawn_game_server(port: u16) {
    let docker = Docker::connect_with_socket_defaults().unwrap();

    let container_name = format!("game_server-{}", port);

    let config = ContainerCreateBody {
        image: Some(String::from("mmorpg-game_server")),
        env: Some(vec![
            String::from("IP=127.0.0.1"),
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
            platform: String::from("None"),
        }),
        config,
    ).await.unwrap();

    docker.start_container(
        &container_name,
        None::<StartContainerOptions>,
    ).await.unwrap();
}