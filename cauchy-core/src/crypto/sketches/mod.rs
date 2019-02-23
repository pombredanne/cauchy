pub mod dummy_sketch;
pub mod odd_sketch;
use bytes::Bytes;
use crypto::hashes::blake2b::Blk2bHashable;
use std::collections::HashSet;

pub trait SketchInsertable {
    fn new() -> Self;
    fn insert<T>(&mut self, item: &T)
    where
        T: Blk2bHashable;
    fn insert_id(&mut self, item: &Bytes);
}

pub trait Sketchable {
    fn sketch<T: Blk2bHashable, U>(items: &U) -> Self
    where
        U: IntoIterator<Item = T>,
        U: Clone;
    fn sketch_ids<U>(items: &U) -> Self
    where
        U: IntoIterator<Item = Bytes>,
        U: Clone;
}

impl<V> Sketchable for V
where
    V: SketchInsertable,
{
    fn sketch<T: Blk2bHashable, U>(items: &U) -> Self
    where
        U: IntoIterator<Item = T>,
        U: Clone,
    {
        let mut new_sketch = Self::new();
        for item in items.clone().into_iter() {
            new_sketch.insert(&item);
        }
        new_sketch
    }

    fn sketch_ids<U>(items: &U) -> Self
    where
        U: IntoIterator<Item = Bytes>,
        U: Clone,
    {
        let mut sketch = Self::new();
        for item in items.clone().into_iter() {
            sketch.insert_id(&item);
        }
        sketch
    }
}

pub trait Decodable {
    fn decode(&self) -> Result<(HashSet<Bytes>, HashSet<Bytes>), String>;
}
