use bytes::Bytes;
use crate::SerializationError;

pub mod game_server;
pub mod broker;
pub mod shard;
pub mod spatial_server;
pub mod orchestrator;

pub trait Packet
{
    fn read(bytes: Bytes) -> Result<Self, SerializationError>
        where Self: Sized;
    fn write(self) -> Result<bytes::Bytes, SerializationError>;
}