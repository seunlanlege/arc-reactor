use contrib::Json;
#[cfg(feature = "unstable")]
use contrib::MultiPartMap;
use http::request::Parts;
use hyper::{
	header::{HeaderMap, HeaderValue},
	Body,
	Method,
	Uri,
	Version,
};
use percent_encoding::percent_decode;
use routing::recognizer::Params;
use serde::de::DeserializeOwned;
use serde_json::{self, from_slice};
use serde_qs::{self, from_str};
#[cfg(feature = "unstable")]
use std::{collections::HashMap};
use std::net::SocketAddr;

/// The Request Struct, This is passed to Middlewares and route handlers.
///
/// #
#[derive(Debug)]
pub struct Request {
	pub(crate) parts: Parts,
	pub(crate) body: Body,
	pub(crate) remote: Option<SocketAddr>,
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
	pub(crate) fn new(parts: Parts, body: Body) -> Self {
		Self { parts, body, remote: None }
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
	// pub fn reactor_handle(&self) -> Handle {
	// 	self.handle.clone().unwrap()
	// }

	/// Returns a reference to the request's Version
	#[inline]
	pub fn version(&self) -> &Version {
		&self.parts.version
	}

	/// Returns a reference to the request's headers
	#[inline]
	pub fn headers(&self) -> &HeaderMap<HeaderValue> {
		&self.parts.headers
	}

	pub fn headers_mut(&mut self) -> &mut HeaderMap<HeaderValue> {
		&mut self.parts.headers
	}

	/// Returns a reference to the request's method
	///
	/// The methods can be any of the following:
	/// GET, HEAD, POST, PUT DELETE, CONNECT, OPTIONS, TRACE, PATCH.
	#[inline]
	pub fn method(&self) -> &Method {
		&self.parts.method
	}

	/// Returns a request to the request's Uri
	#[inline]
	pub fn uri(&self) -> &Uri {
		&self.parts.uri
	}

	/// Returns the query path of the request.
	#[inline]
	pub fn path(&self) -> &str {
		self.parts.uri.path()
	}

	/// Returns the IP of the connected client.
	/// This should always be set, except in testing environments with
	/// `FakeReactor`.
	#[inline]
	pub fn remote_ip(&self) -> Option<SocketAddr> {
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
		self.uri()
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
		self.parts.extensions.get::<Params>()
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
	pub fn get<T: Send + Sync + 'static>(&self) -> Option<&T> {
		self.parts.extensions.get::<T>()
	}

	/// Set a type on the request.
	pub fn set<T: Send + Sync + 'static>(&mut self, value: T) -> Option<T> {
		self.parts.extensions.insert::<T>(value)
	}

	/// Removes the type previously set on the request.
	pub fn remove<T: Send + Sync + 'static>(&mut self) -> Option<T> {
		self.parts.extensions.remove::<T>()
	}

	/// Move the request body
	#[inline]
	pub fn body(&mut self) -> Body {
		::std::mem::replace(&mut self.body, Body::empty())
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
	pub fn body_ref(&self) -> &Body {
		&self.body
	}

	/// Get a reference to the request body.
	#[inline]
	pub fn body_mut(&mut self) -> &mut Body {
		&mut self.body
	}

	/// Set the request body.
	pub fn set_body(&mut self, body: Body) {
		self.body = body
	}
}
