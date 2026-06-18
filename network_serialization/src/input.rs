use bitflags::bitflags;
use bytes::{Bytes, BytesMut};
use crate::{Deserializable, Serializable, SerializationError};

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
    pub struct DirectionFlags: u8 {
        const UP    = 0b0000_0001;
        const DOWN  = 0b0000_0010;
        const LEFT  = 0b0000_0100;
        const RIGHT = 0b0000_1000;
    }
}

#[derive(Debug, PartialEq, Clone, Copy, Default)]
pub struct InputData {
    pub sequence: u32,
    pub input: DirectionFlags,
}

impl Serializable for InputData {
    fn serialize(self, stream: &mut BytesMut) -> Result<(), SerializationError> {
        self.sequence.serialize(stream)?;
        self.input.bits().serialize(stream)?;
        Ok(())
    }
}

impl Deserializable for InputData {
    fn deserialize(bytes: &mut Bytes) -> Result<Self, SerializationError>
    {
        let sequence = u32::deserialize(bytes)?;
        let input = DirectionFlags::from_bits_truncate(u8::deserialize(bytes)?);
        Ok(InputData { sequence, input })
    }
}