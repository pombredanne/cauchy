use bytes::Bytes;

pub trait Sketchable<T>: Into<Vec<T>> where T: Into<Bytes> {
	fn odd_sketch(&self) -> Bytes;
}

// impl<T: Into<Bytes> + Clone> Sketchable<T> for Vec<T> {
// 	fn odd_sketch(&self) -> Bytes {
		
// 	}

// }