use std::fmt::{Debug, Formatter};

use anyhow::Result;
use clap::Parser;
use diesel::pg::PgConnection;
use diesel::prelude::*;
use diesel::r2d2::{ConnectionManager, Pool};
use url::Url;

use self::models::{Flag, NewFlag};

pub mod models;
pub mod schema;

pub type PgConnectionPool = Pool<ConnectionManager<PgConnection>>;

#[derive(Parser, Clone)]
pub struct DbConfig {
    #[arg(long, env, default_value = "user")]
    db_user: String,
    #[arg(long, env, default_value = "password")]
    db_pass: String,
    #[arg(long, env, default_value = "localhost")]
    db_host: String,
    #[arg(long, env, default_value = "5552")]
    db_port: u16,
    #[arg(long, env, default_value = "db")]
    db_name: String,
    #[arg(long, env)]
    db_pool_size: Option<u32>,
}

#[derive(Clone)]
pub struct Db {
    pub connection: PgConnectionPool,
}

impl DbConfig {
    pub fn url(&self) -> String {
        format!(
            "postgres://{}:{}@{}:{}/{}",
            &self.db_user, &self.db_pass, &self.db_host, &self.db_port, &self.db_name
        )
    }

    pub fn connect(self) -> Result<Db> {
        let url: Url = self.url().parse().expect(&format!("Invalid DB URL from {self:?}"));
        let connection = create_pool(url.as_str(), self.db_pool_size);
        Ok(Db { connection })
    }
}

pub fn create_pool(database_url: &str, pool_size: Option<u32>) -> PgConnectionPool {
    let manager = ConnectionManager::<PgConnection>::new(database_url);
    PgConnectionPool::builder()
        .max_size(pool_size.unwrap_or(1))
        .build(manager)
        .expect(&format!("error creating pool"))
}

impl Debug for DbConfig {
    /// Safe implementation that redacts the password, so you can't log it by mistake!
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut url = self.url();
        url = url.replace(&self.db_pass, "*****");
        f.write_str(&url)
    }
}

impl Db {
    // Put your DB methods here. These are just examples
    pub fn create_flag(&self, flag: &bool) -> Result<Flag> {
        use schema::flags;
        let new_flag = NewFlag { flag: flag.clone() };

        Ok(diesel::insert_into(flags::table)
            .values(&new_flag)
            .get_result(&mut self.connection.get()?)?)
    }

    pub fn get_flag(&self, flag_id: i32) -> Result<Flag> {
        use schema::flags::dsl::*;
        Ok(flags
            .filter(id.eq_all(flag_id))
            .first::<Flag>(&mut self.connection.get()?)?)
    }
}
