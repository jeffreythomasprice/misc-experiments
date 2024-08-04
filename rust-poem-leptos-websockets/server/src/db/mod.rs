pub mod models;
pub mod schema;

use std::env;

use anyhow::Result;
use diesel::pg::PgConnection;
use diesel::prelude::*;
use diesel::r2d2::{ConnectionManager, Pool};

pub fn connection() -> Result<Pool<ConnectionManager<PgConnection>>> {
    let manager = ConnectionManager::<PgConnection>::new(&env::var("DATABASE_URL")?);
    Ok(Pool::builder().test_on_check_out(true).build(manager)?)
}
