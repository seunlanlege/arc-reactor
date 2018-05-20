use bytes::Bytes;
use futures::prelude::*;
use tokio::{
	fs::File,
	io::{AsyncWrite, Error},
};

pub fn write(file: File, buf: Bytes) -> impl Future<Item = File, Error = Error> {
	FileWrite::new(Some(file), buf)
}

struct FileWrite {
	file: Option<File>,
	buf: Bytes,
}

impl FileWrite {
	fn new(file: Option<File>, buf: Bytes) -> Self {
		Self { file, buf }
	}
}

impl Future for FileWrite {
	type Item = File;
	type Error = Error;

	fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
		if let Some(ref mut file) = self.file {
			let _ = try_ready!(file.poll_write(&self.buf));
		}

		Ok(Async::Ready(self.file.take().unwrap()))
	}
}
