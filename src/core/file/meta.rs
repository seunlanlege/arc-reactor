use futures::prelude::*;
use std::fs::Metadata;
use tokio::{fs::File, io::Error};

struct FileMeta(Option<File>);

impl Future for FileMeta {
	type Item = (File, Metadata);
	type Error = Error;
	fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
		let mut file = self.0.take().unwrap();
		let metadata = match try!(file.poll_metadata()) {
			Async::Ready(m) => m,
			Async::NotReady => {
				self.0 = Some(file);
				return Ok(Async::NotReady);
			}
		};

		Ok(Async::Ready((file, metadata)))
	}
}

pub fn metadata(file: File) -> impl Future<Item = (File, Metadata), Error = Error> {
	FileMeta(Some(file))
}
