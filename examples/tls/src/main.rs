#![feature(proc_macro, generators, box_syntax)]
#![allow(non_camel_case_types, non_snake_case)]

extern crate arc_reactor;
use arc_reactor::native_tls::{Pkcs12, TlsAcceptor};
use arc_reactor::{prelude::*, ArcReactor, Router};

fn main() {
	// build the tls acceptor.
	let der = include_bytes!("identity.p12");
	let cert = Pkcs12::from_der(der, "mypass").unwrap();
	let tls_cx = TlsAcceptor::builder(cert).unwrap().build().unwrap();

	ArcReactor::new()
		.port(3000) // port to listen on
		.routes(getMainRoutes())
		.tls(tls_cx) // set the tls acceptor on the arc reactor
		.threads(3)
		.initiate()
		.unwrap()
	// now visit https://localhost:3000 in your browser.
}

fn getMainRoutes() -> Router {
	// Setup and maps routes to Services.
	Router::new()
		.get("/", RequestHandler)
}

#[service]
fn RequestHandler(_request: Request, mut res: Response) {
	res.set_body("hello world");

	Ok(res)
}
