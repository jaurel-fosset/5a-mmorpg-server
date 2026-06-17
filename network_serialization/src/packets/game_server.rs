use crate::{Deserializable, SerializationError};
use crate::packets::Packet;
use crate::Serializable;

use std::net;
use bytes::Bytes;

#[derive(Debug)]
pub struct HeartbeatPacket
{
    pub ip : net::Ipv4Addr,
    pub port : u16,
    pub player_number: u8,
    pub player_capacity: u8,
    pub cpu_load: u8,
    pub ram_load: u8,
}

impl Serializable for HeartbeatPacket
{
    fn serialize(self, stream: &mut bytes::BytesMut) -> Result<(), SerializationError>
    {
        self.ip.serialize(stream)?;
        self.port.serialize(stream)?;
        self.player_number.serialize(stream)?;
        self.player_capacity.serialize(stream)?;
        self.cpu_load.serialize(stream)?;
        self.ram_load.serialize(stream)?;

        Ok(())
    }
}

impl Deserializable for HeartbeatPacket
{
    fn deserialize(bytes: &mut Bytes) -> Result<Self, SerializationError>
    where
        Self: Sized
    {
        let ip = net::Ipv4Addr::deserialize(bytes)?;
        let port = u16::deserialize(bytes)?;
        let player_number = u8::deserialize(bytes)?;
        let player_capacity = u8::deserialize(bytes)?;
        let cpu_load = u8::deserialize(bytes)?;
        let ram_load = u8::deserialize(bytes)?;

        Ok(Self
        {
            ip,
            port,
            player_number,
            player_capacity,
            cpu_load,
            ram_load,
        })
    }
}
