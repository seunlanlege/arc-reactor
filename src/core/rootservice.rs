use hyper;
use hyper::server::Service;
use futures::Future;
use futures::prelude::{async_block, await};
use tokio_core::reactor::Handle;
use super::{Request, Response};
use proto::{ArcHandler, ArcService};
use std::sync::Arc;
use std::net::SocketAddr;

// The only reason this exists is so i can pass the
// clientIp to the ArcService.
pub(crate) struct RootService {
	pub(crate) remote_ip: SocketAddr,
	pub(crate) service: Arc<ArcHandler>,
	pub(crate) handle: Handle
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
		let responseFuture = ArcService::call(&*self.service, request, Response::new());

		let future = async_block! {
			let response = await!(responseFuture);
			match response {
				Ok(res) => Ok(res.into()),
				Err(res) => Ok(res.into())
			}
		};

		return box future;
	}
}
