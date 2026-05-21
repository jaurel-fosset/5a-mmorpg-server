use std::sync::Arc;
use std::time::Duration;
use crate::AppState;

pub async fn run(state: Arc<AppState>) {
    let mut interval = tokio::time::interval(
        Duration::from_secs(10)
    );

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
            // TODO : logic to create server
        }
    }
}