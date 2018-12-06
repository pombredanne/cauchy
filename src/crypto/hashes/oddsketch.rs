use bytes::Bytes;

pub trait Sketchable<T>: Into<Vec<T>> where T: Into<Bytes> {

}