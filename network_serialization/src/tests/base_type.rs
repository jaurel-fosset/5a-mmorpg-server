use rand::RngExt;
use crate::base_type::{deserialize_f32_with_precision_loss, serialize_f32_with_precision_loss};
use crate::tests::*;

#[test]
fn test_f32_serialization()
{
    test_serialization::<f32>();
}

#[test]
fn test_f32_serialization_precision_loss()
{
    let value = f32::random_test_value();
    let mut buffer = bytes::BytesMut::with_capacity(4);

    serialize_f32_with_precision_loss(value, &mut buffer)
        .expect("Could not serialize value");
    let mut buffer = buffer.freeze();

    let value_with_precision_loss = value * 100.0;
    let value_with_precision_loss = (value_with_precision_loss * i16::MAX as f32) / f32::MAX;
    let value_with_precision_loss = value_with_precision_loss.trunc();

    let deserialized_value = deserialize_f32_with_precision_loss(&mut buffer)
        .expect("Could not deserialize value");

    assert_eq!(value_with_precision_loss, deserialized_value,
               "Deserialized value ({deserialized_value:?}) is different from original value ({value_with_precision_loss:?})");
}

#[test]
fn test_u8_serialization()
{
    test_serialization::<u8>();
}

#[test]
fn test_u16_serialization()
{
    test_serialization::<u16>();
}

#[test]
fn test_u32_serialization()
{
    test_serialization::<u32>();
}

#[test]
fn test_u64_serialization()
{
    test_serialization::<u64>();
}

#[test]
fn test_u128_serialization()
{
    test_serialization::<u128>();
}


impl RandomTestValue for f32
{
    fn random_test_value() -> Self
    {
        let mut rng = rand::rng();
        rng.random::<Self>()
    }
}

impl RandomTestValue for u8
{
    fn random_test_value() -> Self
    {
        let mut rng = rand::rng();
        rng.random::<Self>()
    }
}

impl RandomTestValue for u16
{
    fn random_test_value() -> Self
    {
        let mut rng = rand::rng();
        rng.random::<Self>()
    }
}

impl RandomTestValue for u32
{
    fn random_test_value() -> Self
    {
        let mut rng = rand::rng();
        rng.random::<Self>()
    }
}

impl RandomTestValue for u64
{
    fn random_test_value() -> Self
    {
        let mut rng = rand::rng();
        rng.random::<Self>()
    }
}

impl RandomTestValue for u128
{
    fn random_test_value() -> Self
    {
        let mut rng = rand::rng();
        rng.random::<Self>()
    }
}