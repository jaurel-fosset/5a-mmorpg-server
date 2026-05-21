use bytes::Bytes;

pub mod game_server;

pub trait Packet
{
    fn read(bytes: Bytes) -> Self;
    fn write(self) -> bytes::Bytes;
}