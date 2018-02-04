mod router;
mod routegroup;

pub use self::routegroup::*;
pub use self::router::*;

#[cfg(test)]
mod tests {
	use impl_service::*;
	use hyper::*;
	use futures::Future;
	use ArcProto::ArcService;
	use futures::future;
	
	use super::*;
	
	#[service]
	fn AsyncService()  {
		Response::new()
			.with_status(StatusCode::Ok)
			.with_body("Hello World".as_bytes())
	}
}
