use hyper::{Uri, Body, HttpVersion, Headers, Method};
use std::{fmt, net};
use recognizer::Params;
use anymap::AnyMap;

pub struct Request {
	uri: Uri,
	pub body: Body,
	version: HttpVersion,
	headers: Headers,
	remote: Option<net::SocketAddr>,
	method: Method,
	pub(crate) paramsMap: AnyMap
}

impl Request {
	pub fn new(
		method: Method,
		uri: Uri,
		version: HttpVersion,
		headers: Headers,
		body: Body,
		remote: Option<net::SocketAddr>
	) -> Self {
		Self {
			method,
			uri,
			version,
			headers,
			body,
			remote,
			paramsMap: AnyMap::new()
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
	pub fn query(&self) -> Option<&str> {
		self.uri.query()
	}

	pub fn params(&self) -> Option<&Params> {
		self.paramsMap.get::<Params>()
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
