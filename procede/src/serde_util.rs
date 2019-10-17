use cgmath::{Vector2, Vector3};
use serde::{Deserialize, Serialize};
use serde::Serializer;
use serde::ser::SerializeStruct;


#[derive(Serialize, Deserialize, Debug)]
pub struct SerializableVector2<T> {
    x: T,
    y: T
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SerializableVector3<T> {
    x: T,
    y: T,
    z: T
}

impl<T> SerializableVector2<T> {
    pub fn new(v: &Vector2<T>) -> SerializableVector2<T>
        where T: Copy
    {
        SerializableVector2 { x: v.x, y: v.y }
    }
}

impl<T> SerializableVector3<T> {
    pub fn new(v: &Vector3<T>) -> SerializableVector3<T>
        where T: Copy
    {
        SerializableVector3 { x: v.x, y: v.y, z: v.z }
    }
}

