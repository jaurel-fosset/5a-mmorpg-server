use std::env;
use std::sync::Arc;
use std::time::Duration;
use network_serialization::packets::game_server::HeartbeatPacket;
use network_serialization::packets::Packet;
use crate::AppState;

pub async fn listen(
    state: Arc<AppState>,
){
    let mut con = state.redis_connexion.clone();

    loop {
        let event = {
            let mut peer = state.peer.lock().await;
            peer.poll()
        };

        match event {
            Ok(Some(game_sockets::GameNetworkEvent::Message { connection, stream, data })) => {
                if data.len() < 6 {
                    println!("Packet too small: {} bytes, skipping", data.len());
                    continue;
                }

                let bytes = data;
                let heartbeat = HeartbeatPacket::read(bytes);
                println!("heartbeat {:?}", heartbeat);

                redis::cmd("HSET")
                    .arg(&[
                        format!("gameserver:gs-{}", heartbeat.port).as_str(),
                        "ip", "0.0.0.0",
                        "port", heartbeat.port.to_string().as_str(),
                        "cpu", heartbeat.cpu_load.to_string().as_str(),
                        "ram", heartbeat.ram_load.to_string().as_str(),
                        "players", heartbeat.player_number.to_string().as_str(),
                        "max_players", heartbeat.player_capacity.to_string().as_str(),
                        "zone", "America/Toronto"
                    ])
                    .exec_async(&mut con)
                    .await
                    .unwrap();

                redis::cmd("EXPIRE")
                    .arg(&[format!("gameserver:gs-{}", heartbeat.port).as_str(), "15"])
                    .exec_async(&mut con)
                    .await
                    .unwrap();

                let score = (heartbeat.cpu_load as f32 * 0.4)
                    + (heartbeat.ram_load as f32 * 0.3)
                    + (heartbeat.player_number as f32 / heartbeat.player_capacity as f32 * 100.0 * 0.3);

                redis::cmd("ZADD")
                    .arg(&[
                        "servers:available",
                        score.to_string().as_str(),
                        format!("gameserver:gs-{}", heartbeat.port).as_str()
                    ])
                    .exec_async(&mut con)
                    .await
                    .unwrap();

                println!("score {:?}", score);
            }
            Ok(Some(game_sockets::GameNetworkEvent::Connected(conn))) => {
                println!("GameServer connected: {:?}", conn);
            }
            Ok(Some(e)) => println!("Event: {:?}", e),
            Ok(None) => tokio::time::sleep(Duration::from_millis(10)).await,
            Err(e) => println!("Error: {}", e),
        }
    }
}
