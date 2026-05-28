use std::net::Ipv4Addr;
use crate::tests::*;

#[test]
fn test_ipv4_serialization()
{
    test_serialization::<Ipv4Addr>()
}

impl RandomTestValue for Ipv4Addr
{
    fn random_test_value() -> Self
    {
        let octets =
        [
            u8::random_test_value(),
            u8::random_test_value(),
            u8::random_test_value(),
            u8::random_test_value()
        ];
        
        Ipv4Addr::from_octets(octets)
    }
}