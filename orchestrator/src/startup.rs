use std::collections::HashMap;
use bollard::config::{ContainerCreateBody, HostConfig};
use bollard::Docker;
use bollard::query_parameters::CreateContainerOptions;

async fn startup(mut redis: redis::aio::MultiplexedConnection)
{
    let mut docker = Docker::connect_with_socket_defaults().unwrap();

    spawn_broker_service(&mut docker).await;
    let ip = get_docker_ip(&mut docker, "broker-service".to_string()).await;

    redis::cmd("SET")
        .arg("broker-service")
        .arg(ip)
        .exec_async(&mut redis)
        .await
        .expect("Failed to set broker service");

    spawn_spatial_service(&mut docker).await;
}

async fn spawn_broker_service(docker: &mut Docker)
{

    let container_name = "broker-service";

    let config = ContainerCreateBody {
        image: Some(String::from("broker")),
        host_config: Some(HostConfig {
            port_bindings: Some({
                let mut map = HashMap::new();
                map.insert(
                    format!("{}/udp", BROKER_PORT),
                    Some(vec![bollard::models::PortBinding {
                        host_ip: Some("0.0.0.0".to_string()),
                        host_port: Some(BROKER_PORT.to_string()),
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
            name: Some(container_name.to_string()),
            platform: String::new(),
        }),
        config,
    ).await.unwrap();
}

async fn spawn_spatial_service(docker: &mut Docker)
{
    let container_name = "spatial-service";

    const ORCHESTRATOR_PORT :u16 = 50_000;
    const BROKER_PORT: u16 = 10_001;

    let config = ContainerCreateBody
    {
        image: Some(String::from("spatial")),
        host_config: Some(HostConfig
        {
            port_bindings: Some(
            {
                let mut map = HashMap::new();

                map.insert
                (
                    format!("{}/udp", BROKER_PORT),
                    Some(vec![bollard::models::PortBinding
                    {
                        host_ip: Some("0.0.0.0".to_string()),
                        host_port: Some(BROKER_PORT.to_string()),
                    }]),
                );
                map.insert(
                    format!("{}/udp", ORCHESTRATOR_PORT),
                    Some(vec![bollard::models::PortBinding
                    {
                        host_ip: Some("0.0.0.0".to_string()),
                        host_port: Some(ORCHESTRATOR_PORT.to_string()),
                    }])
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
            name: Some(container_name.to_string()),
            platform: String::new(),
        }),
        config,
    ).await.unwrap();
}

async fn get_docker_ip(docker: &mut Docker, container_name: String) -> String
{
    let inspect = docker.inspect_container(&container_name, None).await.unwrap();

    inspect
        .network_settings.unwrap()
        .networks.unwrap()
        .get("mmorpg-server_default").unwrap()
        .ip_address.clone().unwrap()
}

async fn send_hello(ip: &str, port: u16)
{

}