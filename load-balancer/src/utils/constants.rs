use std::env;

use dotenvy::dotenv;
use lazy_static::lazy_static;

lazy_static! {
    pub static ref DATABSE_URL: String = set_database_url();
}


fn set_database_url() -> String {
    dotenv().ok();
    env::var(env_variables::DATABASE_URL_ENV_VAR).expect("Database connection url must be set")
}

pub mod env_variables {
    pub const DATABASE_URL_ENV_VAR: &str = "DATABASE_URL";
}
