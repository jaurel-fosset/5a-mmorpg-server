use crate::{Deserializable, SerializationError};
use crate::packets::Packet;
use crate::Serializable;

#[derive(Debug)]
pub struct HeartbeatPacket
{
    pub port : u16,
    pub player_number: u8,
    pub player_capacity: u8,
    pub cpu_load: u8,
    pub ram_load: u8,
}

impl Packet for HeartbeatPacket
{
    fn read(mut bytes: bytes::Bytes) -> Result<Self, SerializationError>
    {
        let port = u16::deserialize(&mut bytes)?;
        let player_number = u8::deserialize(&mut bytes)?;
        let player_capacity = u8::deserialize(&mut bytes)?;
        let cpu_load = u8::deserialize(&mut bytes)?;
        let ram_load = u8::deserialize(&mut bytes)?;

        Ok(Self
        {
            port,
            player_number,
            player_capacity,
            cpu_load,
            ram_load,
        })
    }

    fn write(self) -> Result<bytes::Bytes, SerializationError>
    {
        let mut buffer = bytes::BytesMut::new();
        self.port.serialize(&mut buffer)?;
        self.player_number.serialize(&mut buffer)?;
        self.player_capacity.serialize(&mut buffer)?;
        self.cpu_load.serialize(&mut buffer)?;
        self.ram_load.serialize(&mut buffer)?;

        Ok(buffer.freeze())
    }
}