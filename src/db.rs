use anyhow::Result;
use sea_orm::{Database, DatabaseConnection};

#[derive(Debug)]
pub struct AuthAdapter {
    conn: DatabaseConnection,
}

impl AuthAdapter {
    pub async fn new(connection: &str) -> Result<Self> {
        let conn = Database::connect(connection).await?;
        Ok(Self { conn })
    }
}
