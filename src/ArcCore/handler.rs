use futures::future::Future;
use futures::{Async, Poll, task};

pub struct ReactorHandler<F>
where
	F: Fn(),
{
	pub handler: F,
}

impl<F> Future for ReactorHandler<F>
where
	F: Fn(),
{
	type Item = ();
	type Error = ();

	fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
		(self.handler)();

		Ok(Async::NotReady)
	}
}
