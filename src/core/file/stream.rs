use bytes::{BufMut, BytesMut};
use futures::prelude::*;
use hyper::Chunk;
use tokio::{
	fs::File,
	io::{AsyncRead, AsyncWrite, Error},
};
/// wraps a tokio::fs::File as a futures::Stream
/// will produce an error if this stream isn't polled in the context of a tokio
/// executor
pub struct FileStream {
	file: File,
	buf: BytesMut,
	flushed: bool,
}

pub fn stream(file: File) -> impl Stream<Item = Chunk, Error = Error> {
	FileStream::new(file)
}

impl FileStream {
	fn new(file: File) -> Self {
		Self {
			file,
			buf: BytesMut::with_capacity(0),
			flushed: true,
		}
	}
}

impl Stream for FileStream {
	type Item = Chunk;
	type Error = Error;

	fn poll(&mut self) -> Poll<Option<Self::Item>, Self::Error> {
		let n_bytes = try_ready!(self.file.poll_read(&mut self.buf));
		if n_bytes > 0 {
			Ok(Async::Ready(Some(Chunk::from(self.buf.take().freeze()))))
		} else {
			Ok(Async::Ready(None))
		}
	}
}

impl Sink for FileStream {
	type SinkError = Error;
	type SinkItem = Chunk;

	fn start_send(&mut self, chunk: Chunk) -> StartSend<Self::SinkItem, Self::SinkError> {
		if !self.flushed {
			match try!(self.poll_complete()) {
				Async::Ready(()) => {}
				Async::NotReady => return Ok(AsyncSink::NotReady(chunk)),
			};
		}

		self.buf.put(&*chunk);
		self.flushed = false;
		Ok(AsyncSink::Ready)
	}

	fn poll_complete(&mut self) -> Poll<(), Self::SinkError> {
		let n_bytes = try_ready!(self.file.poll_write(&self.buf));
		println!(
			"poll write => buf length {}, bytes written {}",
			self.buf.len(),
			n_bytes
		);
		self.flushed = true;
		Ok(Async::Ready(()))
	}
}
