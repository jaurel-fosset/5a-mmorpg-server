use std::collections::HashMap;
use std::net::Ipv4Addr;
use bollard::config::{ContainerCreateBody, HostConfig};
use bollard::query_parameters::{CreateContainerOptions, StartContainerOptions};
use crate::connections::get_docker_ip;


const REDIS_PORT: u16 = 6379;

pub async fn launch_redis(docker: &mut bollard::Docker) -> Ipv4Addr
{
    let container_name = "redis-dns-service";

    let config = ContainerCreateBody
    {
        image: Some(String::from("redis:7-alpine")),
        host_config: Some(HostConfig
        {
            port_bindings: Some(
                {
                    HashMap::from
                        ([
                            (
                                format!("{}", REDIS_PORT),
                                Some(vec![bollard::models::PortBinding
                                {
                                    host_ip: Some("127.0.0.1".to_string()),
                                    host_port: Some(REDIS_PORT.to_string()),
                                }])
                            ),
                        ])
                }),
            network_mode: Some("mmorpg-server_default".to_string()),
            ..Default::default()
        }),
        ..Default::default()
    };

    let response = docker.create_container
    (
        Some(CreateContainerOptions
        {
            name: Some(container_name.to_string()),
            platform: String::new(),
        }),
        config,
    ).await.unwrap();

    docker.start_container
    (
        &container_name,
        None::<StartContainerOptions>,
    ).await.unwrap();


    get_docker_ip(docker, &response.id).await
}