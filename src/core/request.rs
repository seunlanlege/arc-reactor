use hyper::{Body, Headers, HttpVersion, Method, Uri};
use std::{fmt, net};
use recognizer::Params;
use anymap::AnyMap;
use serde_json::{from_value, Value, self};
use serde::de::DeserializeOwned;
use queryst_prime::parse;

pub struct Request {
	uri: Uri,
	pub body: Option<Body>,
	version: HttpVersion,
	pub headers: Headers,
	pub remote: Option<net::SocketAddr>,
	method: Method,
	pub json: Option<Value>,
	pub(crate) anyMap: AnyMap,
}

#[derive(Debug)]
pub enum JsonError {
	None,
	Err(serde_json::Error)
}

impl Request {
	pub fn new(
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
			json: None,
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
	pub fn remote_ip(&self) -> net::SocketAddr  {
		self.remote.unwrap()
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

	#[inline]
	pub fn body(self) -> Body { self.body.unwrap_or_default() }

	pub fn json<T>(&self) -> Result<T, JsonError>
		where
			T: DeserializeOwned + fmt::Debug
	{
		let json = self.json.clone();
		if json.is_none() {
			return Err(JsonError::None)
		}

		from_value::<T>(json.unwrap())
			.map_err(JsonError::Err)
	}

	pub fn set_json(&mut self, value: Value) {
		self.json = Some(value);
	}

	pub fn with_json(mut self, value: Value) -> Self {
		self.json = Some(value);

		self
	}

	#[inline]
	pub fn body_ref(&self) -> Option<&Body> { self.body.as_ref() }

	pub fn set_body(&mut self, body: Body) {
		self.body = Some(body)
	}

	pub fn deconstruct(self) -> (Method, Uri, HttpVersion, Headers, Body) {
		(self.method, self.uri, self.version, self.headers, self.body.unwrap_or_default())
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
