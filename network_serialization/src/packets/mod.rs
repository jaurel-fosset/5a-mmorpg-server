use bytes::Bytes;
use crate::SerializationError;

pub mod game_server;

pub trait Packet
{
    fn read(bytes: Bytes) -> Result<Self, SerializationError>
        where Self: Sized;
    fn write(self) -> Result<bytes::Bytes, SerializationError>;
}