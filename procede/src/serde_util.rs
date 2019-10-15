use cgmath::Vector2;
use serde::{Deserialize, Serialize};
use serde::Serializer;
use serde::ser::SerializeStruct;


#[derive(Serialize, Deserialize, Debug)]
pub struct SerializableVector2<T> {
    x: T,
    y: T
}

impl<T> SerializableVector2<T> {
    pub fn new(v: &Vector2<T>) -> SerializableVector2<T>
        where T: Copy
    {
        SerializableVector2 { x: v.x, y: v.y }
    }

    // TODO serialize()
}
