use std::net::Ipv4Addr;
use bytes;
use crate::packets::Packet;
use crate::{Serializable, SerializationError};
use crate::Deserializable;

#[derive(Debug)]
pub struct HelloPacket
{
    pub allocated_ip: Ipv4Addr,
    pub allocated_port: u16,
    pub service_db_ip: Ipv4Addr,
    pub service_db_port: u16,
}

impl Packet for HelloPacket
{
    fn read(mut bytes: bytes::Bytes) -> Result<Self, SerializationError>
    {
        let allocated_ip = Ipv4Addr::deserialize(&mut bytes)?;
        let allocated_port = u16::deserialize(&mut bytes)?;
        let service_db_ip = Ipv4Addr::deserialize(&mut bytes)?;
        let service_db_port = u16::deserialize(&mut bytes)?;
        
        Ok(Self
        {
            allocated_ip,
            allocated_port,
            service_db_ip,
            service_db_port,
        })
    }
    fn write(self) -> Result<bytes::Bytes, SerializationError>
    {
        let mut buffer = bytes::BytesMut::new();
        self.allocated_ip.serialize(&mut buffer)?;
        self.allocated_port.serialize(&mut buffer)?;
        self.service_db_ip.serialize(&mut buffer)?;
        self.service_db_port.serialize(&mut buffer)?;

        Ok(buffer.freeze())
    }
}