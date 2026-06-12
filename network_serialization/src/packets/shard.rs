use bytes::{Bytes, BytesMut};
use crate::{Deserializable, Serializable, SerializationError};

#[derive(Debug, PartialEq, Clone)]
pub struct ClientInputShardPacket {
    pub client_id : u32,
    pub input: [u8; 16],
}
impl Serializable for ClientInputShardPacket {
    fn serialize(self, bytes: &mut BytesMut) -> Result<(), SerializationError> {
        self.client_id.serialize(bytes)?;
        self.input.serialize(bytes)?;
        Ok(())
    }
}
impl Deserializable for ClientInputShardPacket {
    fn deserialize(bytes: &mut Bytes) -> Result<Self, SerializationError> {
        let client_id = u32::deserialize(bytes)?;
        let input = <[u8; 16]>::deserialize(bytes)?;
        Ok(Self { client_id, input })
    }
}