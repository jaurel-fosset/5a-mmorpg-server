use crate::Deserializable;
use crate::packets::Packet;
use crate::Serializable;

pub struct HeartbeatPacket
{
    pub player_number: u8,
    pub player_capacity: u8,
    pub cpu_load: u8,
    pub ram_load: u8,
}

impl Packet for HeartbeatPacket
{
    fn read(mut bytes: bytes::Bytes) -> Self
    {
        let player_number = u8::deserialize(&mut bytes);
        let player_capacity = u8::deserialize(&mut bytes);
        let cpu_load = u8::deserialize(&mut bytes);
        let ram_load = u8::deserialize(&mut bytes);

        Self
        {
            player_number,
            player_capacity,
            cpu_load,
            ram_load,
        }
    }

    fn write(self) -> bytes::Bytes
    {
        let mut buffer = bytes::BytesMut::new();
        self.player_number.serialize(&mut buffer).unwrap();
        self.player_capacity.serialize(&mut buffer).unwrap();
        self.cpu_load.serialize(&mut buffer).unwrap();
        self.ram_load.serialize(&mut buffer).unwrap();

        buffer.freeze()
    }
}