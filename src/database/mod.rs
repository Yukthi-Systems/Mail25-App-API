use deadpool_postgres::{
    PoolError as PgError,
    Pool as PgPool
};

pub mod user;
pub mod maintenance;
pub mod app_sessions;
pub mod sso_sessions;
pub mod mailbox_sessions;


// DB working state Check
pub async fn health_check(db_pool: &PgPool) -> Result<(), PgError> {
    // Simple query to check if the database is responsive
    let client = db_pool.get().await?;
    let _ = client.query("SELECT 1", &[]).await?;
    Ok(())
}
