use crate::AppState;
use network_serialization::packet::{PacketData, PacketMessage};
use network_serialization::packets::Packet;
use network_serialization::packets::orchestrator::OrchestratorHelloPacket;
use std::net::{Ipv4Addr, ToSocketAddrs};
use std::str::FromStr;
use std::sync::Arc;
use std::time::Duration;

pub async fn listen(state: Arc<AppState>) {
    let mut con = state.redis_connexion.clone();

    loop {
        let event = {
            let mut peer = state.peer.lock().await;
            peer.poll()
        };

        match event {
            Ok(Some(game_sockets::GameNetworkEvent::Connected(conn))) => {
                println!("GameServer connected: {:?}", conn);
            }

            Ok(Some(game_sockets::GameNetworkEvent::Message {
                connection,
                stream,
                data,
            })) => {
                let Ok(msg) = PacketMessage::read(data) else {
                    println!("Erreur en lisant data");
                    continue;
                };

                match msg.data {
                    PacketData::AllocateShards(packet) => {
                        let shard_count = packet.shard_count;

                        let socket = std::net::UdpSocket::bind("0.0.0.0:0").unwrap();
                        socket.connect("8.8.8.8:80").unwrap();
                        //let ip = socket.local_addr().unwrap().ip().to_string();

                        let ip = "127.0.0.1";
                        println!("ip {}", ip);

                        let ipv4_orchestrator = Ipv4Addr::from_str(&ip).unwrap();

                        let ipv4_redis = Ipv4Addr::from_str(&ip).unwrap();

                        let ipv4_broker = Ipv4Addr::from_str(&ip).unwrap();

                        let packet = PacketMessage::new(PacketData::OrchestratorHello(
                            OrchestratorHelloPacket {
                                orchestrator: ipv4_orchestrator,
                                redis_dns: ipv4_redis,
                                broker: ipv4_broker,
                            },
                        ));
                        let bytes = packet.write().unwrap();


                        let peer = state.peer.lock().await;
                        peer.send(&connection,&stream,bytes).unwrap();

                        println!("shard_count {}", shard_count);

                    }
                    _ => (),
                }
            }
            Ok(Some(e)) => println!("Event: {:?}", e),
            Ok(None) => tokio::time::sleep(Duration::from_millis(10)).await,
            Err(e) => println!("Error: {}", e),
        }
    }
}
