use futures::{prelude::*};
use hyper::{Chunk};
use tokio::{
	fs::File,
	io::{AsyncRead, Error},
};
use std::fs::Metadata;
/// wraps a tokio::fs::File as a futures::Stream
/// will produce an error if this stream isn't polled in the context of a tokio
/// executor
pub struct FileStream {
	file: File,
	buf: [u8; 4096],
}

impl FileStream {
	pub fn new(file: File) -> Self {
		Self {
			file,
			buf: [0; 4096],
		}
	}
}

impl Stream for FileStream {
	type Item = Chunk;
	type Error = Error;

	fn poll(&mut self) -> Poll<Option<Self::Item>, Self::Error> {
		let n_bytes = try_ready!(self.file.poll_read(&mut self.buf));
		if n_bytes > 0 {
			Ok(Async::Ready(Some(Chunk::from(self.buf.to_vec()))))
		} else {
			Ok(Async::Ready(None))
		}
	}
}

pub struct FileMeta(pub File);

impl Future for FileMeta {
	type Item = (File, Metadata);
	type Error = Error;
	fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
		let metadata = try_ready!(self.0.poll_metadata());
		let file = try_ready!(self.0.poll_try_clone());

		Ok(Async::Ready((file, metadata)))
	}
}
