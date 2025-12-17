mod application;
mod domain;
mod infrastructure;
mod presentation;

use crate::infrastructure::config::Config;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = Config::from_env()?;

    println!("Starting server on {}", config.server_address());
    println!("Database: {}", config.database_url());
    println!("Max connections: {}", config.database.max_connections);

    Ok(())
}