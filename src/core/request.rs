use anymap::AnyMap;
use contrib::Json;
#[cfg(feature = "unstable")]
use contrib::MultiPartMap;
use hyper::{Body, Headers, HttpVersion, Method, Uri};
use percent_encoding::percent_decode;
use routing::recognizer::Params;
use serde::de::DeserializeOwned;
use serde_json::{self, from_slice};
use serde_qs::{self, from_str};
#[cfg(feature = "unstable")]
use std::collections::HashMap;
use std::{fmt, net};
use tokio_core::reactor::Handle;
/// The Request Struct, This is passed to Middlewares and route handlers.
///
pub struct Request {
	pub(crate) uri: Uri,
	pub(crate) handle: Option<Handle>,
	pub(crate) body: Option<Body>,
	pub(crate) version: HttpVersion,
	pub(crate) headers: Headers,
	pub(crate) remote: Option<net::SocketAddr>,
	pub(crate) method: Method,
	pub(crate) anyMap: AnyMap,
}

/// The error returned by `Request::json()`.
///
/// `From<JsonError>` is implemented for Response
/// so you can use the `?` to unwrap or return an early response
///
/// ```rust
/// extern crate arc_reactor;
/// use arc_reactor::prelude::*;
///
/// #[service]
/// fn UserService(req: Request, res: Response) {
/// 	let User { name } = req.json()?;
/// 	// will return an error response with the
/// 	// json '{ "error": "Json was empty" }' if JsonError::None
/// 	// or '{ "error": "{serde error}" }' if it failed to deserialize.
/// 	}
/// ```
///
#[derive(Debug)]
pub enum JsonError {
	/// This error can occur if
	/// 1. The client, didn't include "content-type: application/json" in the
	/// request headers 2. The body parser wasn't mounted as a middleware on
	/// this route, directly or indirectly.
	None,
	/// The error reported by serde, when deserialization of the
	/// request body fails.
	///
	Err(serde_json::Error),
}

/// The error returned by `Request::query()`.
///
/// `From<QueryParseError>` is implemented for Response
/// so you can use the `?` to unwrap or return an early response
///
/// ```rust, ignore
/// #[service]
/// fn UserService(req: Request, res: Response) {
/// 	let AccessToken { token } = req.query()?;
/// 	// will return an error response with the
/// 	// json '{ "error": "query data was empty" }' if QueryParseError::None
/// 	// or '{ "error": "{parse error}" }' if it failed to deserialize.
/// 	}
/// ```
///
#[derive(Debug)]
pub enum QueryParseError {
	/// This error occurs when deserialization fails for the query
	Err(serde_qs::Error),
	/// this error occurs when there is no query data in the uri
	None,
}

impl Request {
	pub(crate) fn new(
		method: Method,
		uri: Uri,
		version: HttpVersion,
		headers: Headers,
		body: Body,
	) -> Self {
		Self {
			method,
			uri,
			version,
			headers,
			body: Some(body),
			remote: None,
			anyMap: AnyMap::new(),
			handle: None,
		}
	}

	/// get a handle to the underlying event loop executing this request.
	/// useful for spawning additional work/futures on the event loop.
	///
	/// # Examples
	/// ```rust
	/// extern crate arc_reactor;
	/// use arc_reactor::prelude::*;
	///
	/// #[service]
	/// pub fn UsefulService(req: Request, res: Response) {
	/// 	let handle = req.reactor_handle();
	///
	/// 	let future = async_block! {
	/// 	println!("I'm doing some asynchronous work!");
	/// 	Ok(()) // async_block must return a type-def of Result
	/// 	// and the event loop requires a future of type `Future<Item = (), Error = ()>`
	/// 		};
	///
	/// 	handle.spawn(future);
	/// 	return Ok(res);
	/// 	}
	/// ```
	///
	pub fn reactor_handle(&self) -> Handle {
		self.handle.clone().unwrap()
	}

	/// Returns a reference to the request's HttpVersion
	#[inline]
	pub fn version(&self) -> &HttpVersion {
		&self.version
	}

	/// Returns a reference to the request's headers
	#[inline]
	pub fn headers(&self) -> &Headers {
		&self.headers
	}

	pub fn headers_mut(&mut self) -> &mut Headers {
		&mut self.headers
	}

	/// Returns a reference to the request's method
	///
	/// The methods can be any of the following:
	/// GET, HEAD, POST, PUT DELETE, CONNECT, OPTIONS, TRACE, PATCH.
	#[inline]
	pub fn method(&self) -> &Method {
		&self.method
	}

	/// Returns a request to the request's Uri
	#[inline]
	pub fn uri(&self) -> &Uri {
		&self.uri
	}

	/// Returns the query path of the request.
	#[inline]
	pub fn path(&self) -> &str {
		self.uri.path()
	}

	/// Returns the IP of the connected client.
	/// This should always be set, except in testing environments with
	/// `FakeReactor`.
	#[inline]
	pub fn remote_ip(&self) -> Option<net::SocketAddr> {
		self.remote
	}

	/// Serializes the query string into a struct via serde.
	///
	///  # Examples
	///
	/// ```rust
	/// extern crate arc_reactor;
	/// use arc_reactor::prelude::*;
	///
	/// #[derive(Serialize, Deserialize)]
	/// struct AccessToken {
	/// 	token: String,
	/// 	}
	///
	/// #[service]
	/// pub fn login(req: Request, _res: Response) {
	/// 	if let Ok(AccessToken { token }) = req.query() {
	/// 		// do something with the token here.
	/// 		}
	/// 	}
	/// ```
	///
	#[inline]
	pub fn query<T>(&self) -> Result<T, QueryParseError>
	where
		T: DeserializeOwned,
	{
		self.uri
			.query()
			.ok_or(QueryParseError::None)
			.and_then(|encoded| Ok(percent_decode(encoded.as_bytes()).decode_utf8_lossy()))
			.and_then(|query| from_str::<T>(&query).map_err(QueryParseError::Err))
	}

	/// Get the url params for the request.
	///
	/// e.g `/profile/:id`
	///
	/// ```rust
	/// extern crate arc_reactor;
	/// use arc_reactor::prelude::*;
	///
	/// [service]
	/// pub fn ProfileService(req: Request, res: Response) {
	/// 	let profileId = req.params().unwrap()["id"];
	/// 	// Its safe to unwrap here as this woute would never be matched without the `id`
	/// }
	/// ```
	pub fn params(&self) -> Option<&Params> {
		self.anyMap.get::<Params>()
	}

	/// The request struct constains an `AnyMap` so that middlewares can append
	/// additional information.
	///
	/// You can get values out of the `AnyMap` by using this method.
	///
	/// # Examples
	///
	/// ```rust
	/// extern crate arc_reactor;
	/// use arc_reactor::prelude::*;
	/// #[derive(Serialize, Deserialize)]
	/// struct AccessToken {
	/// 	token: String,
	/// 	}
	///
	/// struct User {
	/// 	name: String,
	/// 	}
	///
	/// #[middleware(Request)]
	/// pub fn AssertAuth(req: Request) {
	/// 	if let AccessToken { token } = req.query::<AccessToken>() {
	/// 		let user = User { name: "Seun" };
	/// 		req.set::<User>(user); // Set the user
	/// 	} else {
	/// 		return Err((401, "Unauthorized!").into());
	/// 		}
	/// 	}
	///
	/// #[service]
	/// pub fn ProfileService(req: Request, res: Response) {
	/// 	let user = req.get::<User>().unwrap();
	/// 	// Its safe to unwrap here, because if user isn't set this service will never
	/// 	// be called.
	/// 	}
	/// ```
	pub fn get<T: 'static>(&self) -> Option<&T> {
		self.anyMap.get::<T>()
	}

	/// Set a type on the request.
	pub fn set<T: 'static>(&mut self, value: T) -> Option<T> {
		self.anyMap.insert::<T>(value)
	}

	/// Removes the type previously set on the request.
	pub fn remove<T: 'static>(&mut self) -> Option<T> {
		self.anyMap.remove::<T>()
	}

	/// Move the request body
	#[inline]
	pub fn body(&mut self) -> Body {
		match self.body.take() {
			Some(body) => body,
			None => Default::default(),
		}
	}

	/// Serialize the request's json value into a struct.
	///
	/// Note that the json value needs to have been previously set on the
	/// request by a middleware; otherwise this would return
	/// `Err(JsonError::None)`.
	pub fn json<T>(&self) -> Result<T, JsonError>
	where
		T: DeserializeOwned,
	{
		match self.get::<Json>() {
			Some(ref slice) => from_slice::<T>(slice).map_err(JsonError::Err),
			_ => Err(JsonError::None),
		}
	}

	#[cfg(feature = "unstable")]
	pub fn form(&self) -> Result<HashMap<String, String>, JsonError> {
		match self.get::<MultiPartMap>() {
			Some(ref map) => Ok(map.0.clone()),
			_ => Err(JsonError::None),
		}
	}

	/// Get a reference to the request body.
	#[inline]
	pub fn body_ref(&self) -> Option<&Body> {
		self.body.as_ref()
	}

	/// Set the request body.
	pub fn set_body(&mut self, body: Body) {
		self.body = Some(body)
	}

	/// Decontruct this request.
	pub fn deconstruct(self) -> (Method, Uri, HttpVersion, Headers, Body) {
		(
			self.method,
			self.uri,
			self.version,
			self.headers,
			self.body.unwrap_or_default(),
		)
	}
}

impl fmt::Debug for Request {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		f.debug_struct("Request")
			.field("method", &self.method)
			.field("uri", &self.uri)
			.field("version", &self.version)
			.field("remote", &self.remote)
			.field("headers", &self.headers)
			.finish()
	}
}
