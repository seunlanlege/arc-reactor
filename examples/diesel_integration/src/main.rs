#![feature(conservative_impl_trait, proc_macro, box_syntax, generators)]

extern crate arc_reactor;
#[macro_use]
extern crate diesel;
extern crate dotenv;
extern crate futures_cpupool;
#[macro_use]
extern crate lazy_static;
extern crate r2d2;
extern crate r2d2_diesel;
extern crate serde;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate serde_json;

use arc_reactor::prelude::*;
use diesel::prelude::*;
use arc_reactor::{ArcReactor, Router, StatusCode};

mod db;

fn main() {
	let routes = Router::new().get("/", IndexService);
	ArcReactor::new()
		.routes(routes)
		.initiate()
		.unwrap()
}

// DB Person Schema.
#[derive(Queryable, Serialize, Deserialize)]
struct Person {
	pub id: i64,
	pub name: String
}

/// Create a table called `people` and add data to it
/// set the database url as an env.
/// 
/// This guide assumes you already know how to use diesel.
#[service]
fn IndexService(_req: Request, res: Response) {
	use db::people::dsl::*;
	
	// Returns a future, that resolves to the rows in the query.
	let future = db::query(|conn| people.load::<Person>(conn));

	match await!(future) { // the await macro is exported in the arc_reactor::prelude;
		Ok(result) => return Ok(result.into()), // From<T: Serialize> is implemented for Response.
		_ => {}
	};

	// diesel error occured ? then return NotFound.
	return Err(res.with_status(StatusCode::NotFound))
}

