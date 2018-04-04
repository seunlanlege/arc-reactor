#![macro_use]
use core::{Request, Response};
use std::sync::Arc;
use futures::future::{Future, IntoFuture};

type MiddleWareFuture<I> = Box<Future<Item = I, Error = Response>>;

/// The middleware Trait.
/// In arc-reactor the middleware system is designed to be as intuitive and as simple as possible.
///
/// # How it works.
///
/// Think of a `MiddleWare<T>` as a function that returns `Result<T, Response>`
///
/// E.g A middleware is given a `Request` to do some processing, if that MiddleWare, returns
/// Ok(request). Then the returned request is passed to the next middleware or route handler.
///
/// ```rust,ignore
/// use arc_reactor::prelude::*;
///
/// #[middleware(Request)]
/// fn hasAccessToken(req: Request) {
/// 	if let Some(user) = req.query::AccessToken().and_then(|token|
/// 			await!(db::fetchUserFromToken(token)).ok() // psuedo-code this won't compile,
///       // the await macro is exported in the prelude. it is re-exported from futures_await
/// 		)
/// 		req.set::<User>(user);
/// 		return Ok(req);
/// 	}
/// 	let res = Response::new().with_status(StatusCode::Unauthorized);
/// 	return Err(res)
/// }
///
///
/// #[service]
/// fn UserService(req: Request, res: Response) {
/// 	let user = req.get::<User>().unwrap(); // It's safe to unwrap here, because the service is only called when `hasAccessToken` returns Ok(request).
///   ....
/// }
///
///
/// fn main() {
/// 	let router = Router::new()
/// 		.get("/user", arc!(mw![hasAccessToken], UserService));
///   .....
///   // start the server mount the routes.
///
/// }
/// ```
///
/// ## Working With a `Vec<MiddleWare<T>>`
///
/// The same rules as above applies, each middleware pass the request among themselves to do
/// processing. If any of them returns `Err(Response)`, the rest of the middlewares are skipped as
/// well as the route handler.
///
///
/// ```rust,ignore
/// use arc_reactor::prelude::*;
///
/// #[middleware(Request)]
/// fn middleware1(req: Request) {
/// 	println!("will be executed");
/// 	return Ok(req)
/// }
///
/// #[middleware(Request)]
/// fn middleware2(req: Request) {
/// 	println!("will always be called, because middleware1 always returns Ok(request)");
/// 	return Err(Response::new().with_status(StatusCode::Unauthorized))
/// }
///
/// #[middleware(Request)]
/// fn middleware3(req: Request) {
/// 	println!("will never be called");
/// 	return Ok(req)
/// }
///
/// #[service]
/// fn TestService(req: Request, res: Response) {
/// 	println!("Will never be called, because middleware 2 returns an Err(response)")
/// 	......
/// }
///
///
/// fn main() {
/// 	let router = Router::new()
/// 		.get("/user", arc!(mw![middleware1, middleware2, middleware3], TestService)); // note that the order of middlewares matter!
///   .....
///   // start the server mount the routes.
///
/// }
///
/// ```
///
/// # Note
/// Please note that you would probably never have a reason to implement this trait on your type
/// directly.
///
/// Instead you'll use the middleware proc_macro
/// [`#[middleware]`](../impl_service/fn.middleware.html) to decorate your functions. The proc_macro
/// makes `MiddleWare`'s aasynchronous by default. so you can `await!()` on futures.
pub trait MiddleWare<T>: Sync + Send {
	fn call(&self, param: T) -> MiddleWareFuture<T>;
}

/// This enables a vector of `MiddleWare<Request>` to behave like a single `MiddleWare<Request>`
/// returning `Err(Response)` in any of the `MiddleWare<Request>` will cause the rest of the
/// middlewares to be skipped. Note that there's a conveinience macro `mw` that allows you not
/// write boxes everywhere.
///
impl MiddleWare<Request> for Vec<Arc<Box<MiddleWare<Request>>>> {
	fn call(&self, request: Request) -> MiddleWareFuture<Request> {
		self
			.iter()
			.fold(box Ok(request).into_future(), |request, middleware| {
				let clone = middleware.clone();
				box request.and_then(move |req| clone.call(req))
			})
	}
}

/// This enables a vector of `MiddleWare<Request>` to behave like a single `MiddleWare<Request>`
/// returning `Err(Response)` in any of the `MiddleWare<Request>` will cause the rest of the
/// middlewares to be skipped. Note that there's a conveinient macro `mw` that allows you not write
/// boxes everywhere.
///
impl MiddleWare<Response> for Vec<Arc<Box<MiddleWare<Response>>>> {
	fn call(&self, response: Response) -> MiddleWareFuture<Response> {
		self
			.iter()
			.fold(box Ok(response).into_future(), |response, middleware| {
				let clone = middleware.clone();
				box response.and_then(move |res| clone.call(res))
			})
	}
}

/// Set middlewares that should be executed on a request.
///
/// # Example
///
/// ```
/// fn getMainRoutes() -> Router {
/// 	let app_middlewares = mw![checkIfAuth];
///
/// 	// Feel free to use pre-configured app middleware
/// }
///
/// #[middleware(Request)]
/// fn checkIfAuth(req: Request) {
/// 	if req.body().is_empty() {
/// 		return Err((401, "You need to include some data to access this route!").into());
/// 	}
///
/// 	Ok(req)
/// }
/// ```
#[macro_export]
macro_rules! mw {
	($($middlewares:expr), +) => {{
	use std::sync::Arc;
		let middleWares: Vec<Arc<Box<MiddleWare<_>>>> = vec![$(Arc::new(box $middlewares)), +];
     box middleWares as Box<MiddleWare<_>>
	}};
}
