extern crate diesel;
extern crate serde;
extern crate serde_derive;
extern crate serde_json;

// import diesel table schema and database helper
use helpers::{database as db, schema};

#[derive(Queryable, Insertable, Identifiable, Serialize, Deserialize)]
#[table_name = "user"]
#[primary_key(id)]
pub struct User<'a> {
  pub id: u8,
  pub name: &'a str,
  pub password: &'a str,
  pub password_confirm: &'a str,
}


/// Auto-imports all default functions for models (feel free 
/// to override the default implementation).
impl db::Model<User> for User {
  type Schema = schema::user;
}

/// Implementation of struct User
impl <'a> User<'a> {
  
  // Add more model operation.

  pub fn indexByName(&self) -> bool {
    // Implement logic.
    //
    // Use db:query(|conn| ... ) to run queries against diesel, 
    // where `conn` contains your db connection.
    //

    true
  }
}
