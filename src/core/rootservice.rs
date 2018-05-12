use super::{Request, Response};
use futures::Future;
use hyper;
use hyper::server::Service;
use proto::ArcService;
use std::net::SocketAddr;
use std::{panic::AssertUnwindSafe};
use tokio_core::reactor::Handle;
use proto::ArcHandler;
// The only reason this exists is so I can pass the
// clientIp to the ArcService.
pub(crate) struct RootService {
	pub(crate) remote_ip: SocketAddr,
	pub(crate) service: ArcHandler,
	pub(crate) handle: Handle,
}

impl Service for RootService {
	type Request = hyper::Request;
	type Response = hyper::Response;
	type Error = hyper::Error;
	type Future = Box<Future<Item = Self::Response, Error = Self::Error>>;

	fn call(&self, req: Self::Request) -> Self::Future {
		let mut request: Request = req.into();
		request.handle = Some(self.handle.clone());
		request.remote = Some(self.remote_ip);
		let responseFuture =
			AssertUnwindSafe(self.service.call(request, Response::new())).catch_unwind();

		let responseFuture = responseFuture.then(|result| match result {
			Ok(response) => match response {
				Ok(res) => Ok(res.into()),
				Err(res) => Ok(res.into()),
			},
			Err(_) => {
				Ok(hyper::Response::new().with_status(hyper::StatusCode::InternalServerError))
			}
		});

		return Box::new(responseFuture);
	}
}
