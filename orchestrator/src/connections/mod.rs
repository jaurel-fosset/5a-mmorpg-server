pub mod broker;

use std::time::Duration;
use game_sockets as gs;

async fn init_connection(ip: String, port: u16) -> (gs::GamePeer, gs::GameConnection, gs::GameStream)
{
    let mut socket =
    {
        let backend = gs::protocols::QuicBackend::new();
        gs::GamePeer::new(backend)
    };

    socket.connect(&ip, port).unwrap();

    let connection = loop
    {
        if let Ok(Some(game_sockets::GameNetworkEvent::Connected(conn))) = socket.poll()
        {
            break conn;
        }
        tokio::time::sleep(Duration::from_millis(10)).await;
    };

    socket.create_stream(connection.clone(), game_sockets::GameStreamReliability::Unreliable).unwrap();
    let stream: game_sockets::GameStream = loop
    {
        if let Ok(Some(game_sockets::GameNetworkEvent::StreamCreated(_, stream))) = socket.poll()
        {
            break stream;
        }
        tokio::time::sleep(Duration::from_millis(10)).await;
    };

    (socket, connection, stream)
}