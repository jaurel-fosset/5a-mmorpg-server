use std::env;
use std::sync::Arc;
use serde::Deserialize;
use network_serialization::Deserializable;
use crate::AppState;

// TODO: remove and place it in network_serialization
#[derive(Debug)]
struct Heartbeat {
    id: u32,           // 10001
    port: u16,         // 7001
    cpu: u8,           // 51
    ram: u8,           // 20
    players: u16,      // 64
    max_players: u16,  // 120
}

// TODO: remove and place it in network_serialization
impl Deserializable for Heartbeat {
    fn deserialize(bytes: &mut bytes::Bytes) -> Self {
        Self {
            id: <u32 as Deserializable>::deserialize(bytes),
            port: <u16 as Deserializable>::deserialize(bytes),
            cpu: <u8 as Deserializable>::deserialize(bytes),
            ram: <u8 as Deserializable>::deserialize(bytes),
            players: <u16 as Deserializable>::deserialize(bytes),
            max_players: <u16 as Deserializable>::deserialize(bytes),
        }
    }
}


pub async fn listen(
    state: Arc<AppState>,
){
    let port : u16 = env::var("ORCH_PORT").unwrap().parse::<u16>().unwrap();
    let listener = tokio::net::UdpSocket::bind(("0.0.0.0", port)).await.unwrap();

    let mut con = state.redis_connexion.clone();


    let mut buf = [0; 1024];
    loop {
        let (len, addr) = listener.recv_from(&mut buf).await.unwrap();
        println!("{:?} bytes received from {:?}", len, addr);

        let mut bytes : bytes::Bytes = bytes::Bytes::copy_from_slice(&buf[..len]);
        let heartbeat = Heartbeat::deserialize(&mut bytes);

        println!("heartbeat {:?}", heartbeat);

        let _result : String = redis::cmd("PING")
            .query_async(&mut con)
            .await
            .unwrap();

        redis::cmd("HSET")
            .arg(&[
                format!("gameserver:gs-{}", heartbeat.id).as_str(),
                "ip", addr.ip().to_string().as_str(),
                "port", heartbeat.port.to_string().as_str(),
                "cpu", &heartbeat.cpu.to_string().as_str(),
                "ram", &heartbeat.ram.to_string().as_str(),
                "players", &heartbeat.players.to_string().as_str(),
                "max_players", heartbeat.max_players.to_string().as_str(),
                "zone", "America/Toronto"
            ])
            .exec_async(&mut con)
            .await
            .unwrap();

        redis::cmd("EXPIRE")
            .arg(&[format!("gameserver:gs-{}", heartbeat.id).as_str(),"15"])
            .exec_async(&mut con)
            .await
            .unwrap();

        let score = (heartbeat.cpu as f32 * 0.4) + (heartbeat.ram as f32 * 0.3) + (heartbeat.players as f32 / heartbeat.max_players as f32 * 100.0 * 0.3);
        println!("score {:?}", score);

        redis::cmd("ZADD")
            .arg(&[
                "servers:available",
                &score.to_string().as_str(),
                format!("gameserver:gs-{}", heartbeat.id).as_str()
            ])
            .exec_async(&mut con)
            .await
            .unwrap();
    }
}
