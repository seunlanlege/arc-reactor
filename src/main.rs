#![feature(
proc_macro,
box_syntax,
generators,
)]

#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(dead_code)]
#![allow(unused_imports)]

extern crate anymap;
extern crate impl_service;
extern crate num_cpus;
extern crate tokio_core;
extern crate route_recognizer as recognizer;
extern crate futures_await as futures;
extern crate hyper;

mod ArcRouting;
mod ArcCore;
#[macro_use]
mod ArcProto;

use impl_service::{service, middleware};
use hyper::{Error, StatusCode};
use futures::future::Future;
use futures::prelude::{async_block};

use ArcCore::*;
use ArcRouting::*;
use ArcProto::{MiddleWare, ArcService};

fn getMainRoutes() -> ArcRouter {
	let router: ArcRouter = ArcRouter::new()
		.get("/:username", RequestHandler);
//		.post("/", (mw![middleware, middleware2], RequestHandler))
//		.post("/:username", (mw![middleware, middleware2], RequestHandler))

	return router
}

fn main() {
	ArcReactor::new()
		.port(3000)
		.routes(getMainRoutes())
		.initiate()
		.unwrap()
}

struct RequestHandler;
impl ArcService for RequestHandler {
	fn call (&self , request: Request , res: Response ) -> Box<Future<Item=Response , Error=Error>> {
		Box::new(
			async_block!{
        let params = request.params().unwrap();
        let body = format!("Hello {}" , params["username"]);
        let res = res
          .with_status(StatusCode::Ok)
          .with_body(body);
        Ok(res)
      }
		)
	}
}
//#[service]
//fn RequestHandler(request: Request, res: Response) {
//	let url = request.params();
//	let body = format!("Hello {}", url["username"]);
//	let res =	res
//		.with_status(StatusCode::Ok)
//		.with_body(body);
//
//	Ok(res)
//}


//#[middleware]
//fn middleware(req: Request){
//	arc::Ok(req)
//}

//#[middleware]
//fn middleware2(req: Request) {
//	arc::Ok(req)
//}
