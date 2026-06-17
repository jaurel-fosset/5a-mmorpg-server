mod heartbeat;
mod scaler;
pub mod connections;
mod listener;

use redis::aio::MultiplexedConnection;
use std::env;
use std::net::Ipv4Addr;
use std::str::FromStr;
use std::sync::Arc;
use crate::connections::{broker, shards, spatial};

struct AppState {
    redis_connexion: MultiplexedConnection,
    peer: tokio::sync::Mutex<game_sockets::GamePeer>,
}

#[dotenvy::load(path = ".env", required = false)]
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>>
{
    println!("Orchestrator starting...");
    let mut docker = bollard::Docker::connect_with_socket_defaults()?;

    let redis_dns_ip = connections::redis::launch_redis(&mut docker).await;
    let client = redis::Client::open(env::var("REDIS_URL")?)?;
    let con = client.get_multiplexed_async_connection().await?;

    let orchestrator_ip = Ipv4Addr::new(127, 0, 0, 1);

    let broker_task = broker::BrokerTask::new(&mut docker, orchestrator_ip, redis_dns_ip).await;
    let broker_ip = broker_task.get_broker_ip();
    let broker_commands = broker_task.get_command_channel_handle();
    let broker_handle = tokio::spawn(broker_task.run());


    let spatial_task = spatial::SpatialTask::new(&mut docker, orchestrator_ip, redis_dns_ip, broker_ip).await;
    let spatial_events = spatial_task.get_events_handle();
    let spatial_handle = tokio::spawn(spatial_task.run());

    let shards_task = shards::ShardsTask::new(broker_commands, spatial_events, orchestrator_ip, redis_dns_ip, broker_ip).await;
    let shards_handle = tokio::spawn(shards_task.run());


    match tokio::join!(broker_handle, spatial_handle, shards_handle)
    {
        (Err(error), _, _) => Err(Box::new(error).into()),
        (_, Err(error), _) => Err(Box::new(error).into()),
        (_, _, Err(error)) => Err(Box::new(error).into()),
        _ => Ok(()),
    }
}
