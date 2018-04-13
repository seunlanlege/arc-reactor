#[recursion_limit = "128"]
#[macro_use] extern crate diesel;
#[macro_use] extern crate diesel_infer_schema;

use diesel::prelude::*;
use diesel;
use diesel::mysql::MysqlConnection;
use diesel::{result, MyqlConnection};
use r2d2_diesel::ConnectionManager;
use arc_reactor::prelude::*;
use helpers::database;

/// Model is a trait every model in arc-reactor could implement
/// to automatically enable model structs become queryable.
pub trait Model<T>
where
    T: Queryable + Insertable + Identifiable +  Serialize + Deserialize
{
    type Schema;

    /// Get all rows of table implementing this struct.
    pub fn all<T>(id: i32) -> Option<T> {
        let futures_db_response = database::query(|conn| Schema.load::<T>(conn));

        match await!(futures_db_response) { // the await macro is exported in the arc_reactor::prelude;
            Ok(result) => return Ok(result),
            _ => None
        }
    }

    // Feel free to add more generalized table operations.
}
