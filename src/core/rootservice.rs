use super::{Request, Response};
use futures::Future;
use hyper::{self, server::Service, header::Server};
use proto::{ArcHandler, ArcService};
use std::{net::SocketAddr, panic::AssertUnwindSafe};
use tokio_core::reactor::Handle;
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
		let res = Response::new();
		let responseFuture = AssertUnwindSafe(self.service.call(request, res)).catch_unwind();

		let responseFuture =
			responseFuture.then(|result| {
				match result {
					Ok(response) => {
						match response {
							Ok(mut res) | Err(mut res) => {
								res.headers_mut().set(Server::new("Arc-Reactor/0.1.5"));
								Ok(res.into())
							}
						}
					}
					Err(_) => Ok(hyper::Response::new().with_status(hyper::StatusCode::InternalServerError)),
				}
			});

		return Box::new(responseFuture);
	}
}
