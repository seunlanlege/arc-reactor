extern crate arc_reactor;
extern crate serde;
extern crate serde_derive;
extern crate serde_json;

use arc_reactor::{RouteGroup, Router};

// Project imports
mod api;
use self::api as ApiRoutes;
use arc_reactor::prelude::*;
use models::user::*;

/// Setup and mounts route path to methods
pub fn get_route() -> Router {
  // Create an 'api' route-group i.e. localhost:PORT/api/
  let api_group = ApiRoutes::new(RouteGroup::new("api"));

  // Mount application's routes and route-groups.
  Router::new()
    .get("/", get_user)
    .group(api_group)
}

#[service]
fn get_user(_req: Request, mut res: Response) {
  let user = User { 
    name: "Rusty", 
    password: "Hello Rust!", 
    password_confirmation: "Hello Rust!" 
  };
  
  // Serialize it to a JSON string.
  let json_string = serde_json::to_string(&user).unwrap();
  res.set_body(json_string);
  
  Ok(res)
}
