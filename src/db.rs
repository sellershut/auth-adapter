use anyhow::Result;
use sea_orm::{Database, DatabaseConnection};

async fn hello() -> Result<DatabaseConnection> {
    let conn = Database::connect("hello").await?;
    Ok(conn)
}
