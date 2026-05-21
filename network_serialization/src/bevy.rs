use bytes::{Buf, BufMut, Bytes};
use bevy::prelude::*;
use bevy::prelude::ops::sqrt;
use ordered_float::NotNan;
use crate::{Deserializable, Serializable, SerializationError};


impl Serializable for Transform
{
    fn serialize(self, stream: &mut bytes::BytesMut) -> Result<(), SerializationError>
    {
        self.translation.serialize(stream)?;
        self.rotation.serialize(stream)?;
        self.scale.serialize(stream)?;

        Ok(())
    }
}

impl Deserializable for Transform
{
    fn deserialize(bytes: &mut Bytes) -> Self
    {
        let translation = Vec3::deserialize(bytes);
        let rotation = Quat::deserialize(bytes);
        let scale = Vec3::deserialize(bytes);

        Transform { translation, rotation, scale }
    }
}

impl Serializable for Vec3
{
    fn serialize(self, stream: &mut bytes::BytesMut) -> Result<(), SerializationError>
    {
        self.x.serialize(stream)?;
        self.y.serialize(stream)?;
        self.z.serialize(stream)?;

        Ok(())
    }
}

impl Deserializable for Vec3
{
    fn deserialize(bytes: &mut Bytes) -> Self
    {
        let x = f32::deserialize(bytes);
        let y = f32::deserialize(bytes);
        let z = f32::deserialize(bytes);

        Vec3::new(x, y, z)
    }
}

impl Serializable for Quat
{
    fn serialize(self, stream: &mut bytes::BytesMut) -> Result<(), SerializationError>
    {
        let values = self.to_array();
        let values = [
            NotNan::new(values[0])?,
            NotNan::new(values[1])?,
            NotNan::new(values[2])?,
            NotNan::new(values[3])?
        ];

        let max_index = max_by_index(values);
        let values = drop_one(values, max_index);
        let max_index = max_index as u8;

        let mut bytes = [0_u8; 4];
        let max_index = max_index << 6;
        bytes[0] |= max_index;

        let values = values.map(serialize_f32_rotation);
        bytes[1] |= values[0][0];
        bytes[2] |= values[1][0];
        bytes[3] |= values[2][0];
        bytes[0] |= values[0][1] >> 2 | values[1][1] >> 4 | values[2][1] >> 6;

        stream.put_slice(bytes.as_ref());
        Ok(())
    }
}

impl Deserializable for Quat
{
    fn deserialize(bytes: &mut Bytes) -> Self
    {
        let values = [bytes.get_u8(), bytes.get_u8(), bytes.get_u8(), bytes.get_u8()];

        let v1 = u16::from_be_bytes([values[1], values[0] & 0x30 << 2]) as f32;
        let v2 = u16::from_be_bytes([values[2], values[0] & 0x0C << 4]) as f32;
        let v3 = u16::from_be_bytes([values[3], values[0] & 0x03 << 6]) as f32;
        let v4 = sqrt(1.0 - v1*v1 - v2*v2 - v3*v3);

        if values[0] & 0xC0 == 0
        {
            return Quat::from_xyzw(v4, v2, v3, v1);
        }
        if values[0] & 0xC0 == 0x40
        {
            return Quat::from_xyzw(v1, v4, v3, v2);
        }
        if values[0] & 0xC0 == 0x80
        {
            return Quat::from_xyzw(v1, v2, v4, v3);
        }
        if values[0] & 0xC0 == 0xC0
        {
            return Quat::from_xyzw(v1, v2, v3, v4);
        }

        unreachable!()
    }
}

// The last 6 bits of the returned array are guaranteed to be zero
fn serialize_f32_rotation(value: NotNan<f32>) -> [u8; 2]
{
    let value = value * 100.0;
    let value = (value * 1024.0) / 1414.0;

    let value = (value.trunc() as u16) << 6;
    let value: [u8; 2] = value.to_be_bytes();

    [value[0], value[1] & 0xC0]
}

fn max_by_index(values: [NotNan<f32>; 4]) -> usize
{
    values.iter()
        .enumerate()
        .max_by(|x, y| {
            x.1.total_cmp(&y.1)
        })
        .expect("Iterator cannot be empty, we created it from an array with 4 elements").0
}

fn drop_one(mut values: [NotNan<f32>; 4], index: usize) -> [NotNan<f32>; 3]
{
    assert!(index > 3, "Index out of bounds");

    if index < 3
    {
        values.swap(index, 3);
    }

    [values[0], values[1], values[2]]
}