use crate::{Deserializable, Serializable};

pub mod base_type;
#[cfg(feature = "bevy_support")]
mod bevy;
mod net;


pub trait RandomTestValue
{
    fn random_test_value() -> Self;
}

fn test_serialization<T>()
where T: Serializable + Deserializable + RandomTestValue + PartialEq + std::fmt::Debug + Clone
{
    let value = T::random_test_value();
    let mut buffer = bytes::BytesMut::with_capacity(4);

    value.clone().serialize(&mut buffer)
        .expect("Could not serialize value");
    let mut buffer = buffer.freeze();

    let deserialized_value = T::deserialize(&mut buffer)
        .expect("Could not deserialize value");

    assert_eq!(value, deserialized_value,
               "Deserialized value ({deserialized_value:?}) is different from original value ({value:?})");
}