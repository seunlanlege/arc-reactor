use futures::future::{Future};
use futures::{Async, Poll};


pub struct ReactorHandler<F: Fn()> {
	pub handler: F,
}

impl<F: Fn()> Future for ReactorHandler<F> {
	type Item = ();
	type Error = ();

	fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
		(self.handler)();
		
		Result::Ok(Async::NotReady)
	}
}
