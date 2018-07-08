use arc_reactor::futures::{Future, IntoFuture, future};
use futures_cpupool::CpuPool;
use diesel;
use diesel::{result, PgConnection};
use r2d2::{self, Pool};
use r2d2_diesel::ConnectionManager;
use dotenv::var;

lazy_static! {
	static ref FUTURES_POOL: CpuPool = {
		CpuPool::new_num_cpus()
	};
}

pub type Conn = diesel::pg::PgConnection;

pub fn query<T, F, R>(f: F) -> impl Future<Item = T, Error = result::Error>
	where
		T: Send + 'static,
		F: FnOnce(&Conn) -> R + Send + 'static,
		R: Future<Item = T, Error = result::Error> + Send + 'static,
{
	lazy_static! {
		static ref R2D2: Pool<ConnectionManager<PgConnection>> = {
			let database_url = var("DATABASE_URL").expect("DATABASE_URL must be set");
			let manager = ConnectionManager::<PgConnection>::new(database_url.as_str());
			r2d2::Pool::builder().build(manager).expect("Failed to create pool.")
		};
	}

	let pool = R2D2.clone();
	FUTURES_POOL.spawn_fn(move || {
		pool
			.get()
			.map_err(|_err| result::Error::NotFound)
			.map(|conn| future::ok(f(&*conn)))
	})
}

table! {
	people (id) {
		id -> Int8,
		name -> Varchar,
	}
}
