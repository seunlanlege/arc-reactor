#[macro_use]
extern crate serde_json;

use arc_reactor::prelude::*;
use models::User;

#[service]
pub fn index_users(_req: Request, mut res: Response) {
  // Query all rows in 'user' table, and convert result into json.
  res.set_body(User::all().unwrap().into());

  Ok(res)
}

#[service]
pub fn verify_username(_req: Request, mut res: Response) {
  
  // Generate custom json string
  let payload = json!({
    "status" : true, 
    "message": "Username is valid."
  });

  res.set_body(json_str);

  Ok(res)
}

#[service]
pub fn auth(_req: Request, mut res: Response) {

  // Returns a 401 error.
  res.with_status(StatusCode::UnAuthorized)
}
