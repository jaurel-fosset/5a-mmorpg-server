use std::collections::HashMap;
use bollard::config::{ContainerCreateBody, HostConfig};
use bollard::Docker;
use bollard::query_parameters::CreateContainerOptions;

async fn startup(mut redis: redis::aio::MultiplexedConnection)
{
    // redis::cmd("SET")
    //     .arg("broker-service")
    //     .arg(ip)
    //     .exec_async(&mut redis)
    //     .await
    //     .expect("Failed to set broker service");

}