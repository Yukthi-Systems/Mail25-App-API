use crate::models::sso_sessions::SsoSessionInfo;
use deadpool_postgres::{
    PoolError as PgError,
    Pool as PgPool
};


/// List all users SSO sessions
pub async fn list_sso_sessions(db_pool: &PgPool, email: &str) -> Result<Vec<SsoSessionInfo>, PgError> {
    let client = db_pool.get().await?;

    let rows = client.query(
        "SELECT session_id, device_details, created_at, last_auth_at, is_active FROM sso_sessions WHERE email = $1",
        &[&email]
    ).await?;

    Ok(SsoSessionInfo::from_rows(rows))
}


/// Delete an SSO session by session_id
pub async fn sso_session_delete(db_pool: &PgPool, session_id: uuid::Uuid, email: &str) -> Result<(), PgError> {
    let client = db_pool.get().await?;

    client.execute(
        "DELETE FROM sso_sessions WHERE session_id = $1 AND email = $2",
        &[&session_id, &email]
    ).await?;

    Ok(())
}


/// Update Active Status of an SSO session by session_id
pub async fn sso_session_update_active_status(db_pool: &PgPool, session_id: uuid::Uuid, email: &str, is_active: bool) -> Result<(), PgError> {
    let client = db_pool.get().await?;

    client.execute(
        "UPDATE sso_sessions SET is_active = $1 WHERE session_id = $2 AND email = $3",
        &[&is_active, &session_id, &email]
    ).await?;

    Ok(())
}
