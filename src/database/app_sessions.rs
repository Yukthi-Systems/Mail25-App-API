/*
 * Copyright (C) 2026 Yukthi Systems Private Limited
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License version 3
 * as published by the Free Software Foundation.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * version 3 along with this program. If not, see
 * <https://www.gnu.org/licenses/>.
 */


use crate::models::app_sessions::AppSessionInfo;
use deadpool_postgres::{
    PoolError as PgError,
    Pool as PgPool
};


// -- Auth App Sessions
// CREATE TABLE mail25_app_sessions (
//     session_id UUID PRIMARY KEY,
//     email VARCHAR(254) NOT NULL REFERENCES email_identities(email) ON DELETE CASCADE,
//     domain_name VARCHAR(254) NOT NULL REFERENCES domains(domain_name) ON DELETE CASCADE,

//     phone VARCHAR(20) NOT NULL,  -- Phone number used for authentication
//     fcm_token TEXT NOT NULL,  -- FCM token for push notifications

//     device_details JSONB NOT NULL,

//     last_active_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL
//     -- TODO: Add session expiry on App level (e.g., after 10 days of inactivity) [Maintain Last Active At timestamp]
//     -- Add Last Active At timestamp, then run a cron job to delete expired sessions
// );


/// List all users app sessions
pub async fn list_app_sessions(db_pool: &PgPool, phone: &str) -> Result<Vec<AppSessionInfo>, PgError> {
    let client = db_pool.get().await?;

    let rows = client.query(
        "SELECT session_id, email, domain_name, device_details, last_active_at FROM mail25_app_sessions WHERE phone = $1",
        &[&phone]
    ).await?;

    Ok(AppSessionInfo::from_rows(rows))
}


/// Delete an app session by session_id
pub async fn app_session_delete(db_pool: &PgPool, session_id: uuid::Uuid, phone: &str) -> Result<(), PgError> {
    let client = db_pool.get().await?;

    client.execute(
        "DELETE FROM mail25_app_sessions WHERE session_id = $1 AND phone = $2",
        &[&session_id, &phone]
    ).await?;

    Ok(())
}
