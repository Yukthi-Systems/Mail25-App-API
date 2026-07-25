use crate::models::mailbox_sessions::MailBoxSessionInfo;
use deadpool_postgres::{
    PoolError as PgError,
    Pool as PgPool
};


// -- Email Client Sessions
// CREATE TABLE mailbox_sessions (
//     origin_ip VARCHAR(45) NOT NULL,  -- IPv4 or IPv6
//     attempted_by VARCHAR(254) NOT NULL REFERENCES mailboxes(email) ON DELETE CASCADE,
//     domain_name VARCHAR(254) NOT NULL REFERENCES domains(domain_name) ON DELETE CASCADE,

//     geo_ip_location JSONB NOT NULL,
//     is_active BOOLEAN DEFAULT TRUE NOT NULL,    -- Is the session active

//     attempted_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL,
//     session_expires_at TIMESTAMPTZ NOT NULL,

//     PRIMARY KEY (origin_ip, attempted_by)
// );  -- TODO: Every 1 Minute, we run the clean up cron job, to delete the expired sessions


/// List all users mailbox sessions
pub async fn list_mailbox_sessions(db_pool: &PgPool, email: &str) -> Result<Vec<MailBoxSessionInfo>, PgError> {
    let client = db_pool.get().await?;

    let rows = client.query(
        "SELECT origin_ip, geo_ip_location, is_active, attempted_at, session_expires_at FROM mailbox_sessions WHERE attempted_by = $1",
        &[&email]
    ).await?;

    Ok(MailBoxSessionInfo::from_rows(rows))
}


/// Set the active status of a mailbox session by origin_ip
pub async fn toggle_mailbox_session(db_pool: &PgPool, origin_ip: &str, email: &str, enable: bool) -> Result<(), PgError> {
    let client = db_pool.get().await?;

    // Set the is_active status of the session
    client.execute(
        "UPDATE mailbox_sessions SET is_active = $1 WHERE origin_ip = $2 AND attempted_by = $3",
        &[&enable, &origin_ip, &email]
    ).await?;

    Ok(())
}


/// Delete a mailbox session by origin_ip
pub async fn mailbox_session_delete(db_pool: &PgPool, origin_ip: &str, email: &str) -> Result<(), PgError> {
    let client = db_pool.get().await?;

    client.execute(
        "DELETE FROM mailbox_sessions WHERE origin_ip = $1 AND attempted_by = $2",
        &[&origin_ip, &email]
    ).await?;

    Ok(())
}
