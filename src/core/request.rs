use hyper::{Body, Headers, HttpVersion, Method, Uri};
use std::{fmt, net};
use recognizer::Params;
use anymap::AnyMap;
use serde_json::from_value;
use serde::de::DeserializeOwned;
use queryst_prime::parse;

pub struct Request {
	uri: Uri,
	pub body: Body,
	version: HttpVersion,
	headers: Headers,
	remote: Option<net::SocketAddr>,
	method: Method,
	pub(crate) anyMap: AnyMap,
}

impl Request {
	pub fn new(
		method: Method,
		uri: Uri,
		version: HttpVersion,
		headers: Headers,
		body: Body,
		remote: Option<net::SocketAddr>,
	) -> Self {
		Self {
			method,
			uri,
			version,
			headers,
			body,
			remote,
			anyMap: AnyMap::new(),
		}
	}

	#[inline]
	pub fn version(&self) -> &HttpVersion {
		&self.version
	}

	#[inline]
	pub fn headers(&self) -> &Headers {
		&self.headers
	}

	#[inline]
	pub fn method(&self) -> &Method {
		&self.method
	}

	#[inline]
	pub fn uri(&self) -> &Uri {
		&self.uri
	}

	#[inline]
	pub fn path(&self) -> &str {
		self.uri.path()
	}

	#[inline]
	pub fn query<T>(&self) -> Option<T>
	where
		T: DeserializeOwned,
	{
		self
			.uri
			.query()
			.and_then(|query| parse(query).ok())
			.and_then(|value| from_value::<T>(value).ok())
	}

	pub fn params(&self) -> Option<&Params> {
		self.anyMap.get::<Params>()
	}

	pub fn get<T: 'static>(&self) -> Option<&T> {
		self.anyMap.get::<T>()
	}

	pub fn set<T: 'static>(&mut self, value: T) -> Option<T> {
		self.anyMap.insert::<T>(value)
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
