use ::bevy::prelude::*;
use rand::RngExt;
use crate::tests::*;

#[test]
fn test_vec3_serialization()
{
    test_serialization::<Vec3>();
}

#[test]
fn test_quat_serialization()
{
    test_serialization::<Quat>();
}

#[test]
fn test_transform_serialization()
{
    test_serialization::<Transform>();
}


impl RandomTestValue for Vec3
{
    fn random_test_value() -> Self
    {
        let mut rng = rand::rng();
        let x = rng.random();
        let y = rng.random();
        let z = rng.random();

        Vec3::new(x, y, z)
    }
}

impl RandomTestValue for Quat
{
    fn random_test_value() -> Self
    {
        let mut rng = rand::rng();
        let x = rng.random();
        let y = rng.random();
        let z = rng.random();
        let w = rng.random();

        Quat::from_xyzw(x, y, z, w)
    }
}

impl RandomTestValue for Transform
{
    fn random_test_value() -> Self
    {
        let translation = Vec3::random_test_value();
        let rotation = Quat::random_test_value();
        let scale = Vec3::random_test_value();

        Transform
        {
            translation,
            rotation,
            scale,
        }
    }
}