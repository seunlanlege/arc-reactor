#[macro_use]
extern crate serde_json;

use arc_reactor::prelude::*;
use models::User;

#[service]
pub fn get_user(_req: Request, mut res: Response) {
    let new_user = User {
        name: "Rusty",
        password: "Hello Rust!",
        password_confirmation: "Hello Rust!"
    };
      
    // Serialize user into JSON string.
    res.set_body(new_user.into());
          
    Ok(res)
}
