use tokio_postgres::row::Row;
use serde::Serialize;


#[derive(Serialize)]
pub struct SsoSessionInfo {
    pub session_id: uuid::Uuid,
    pub device_details: serde_json::Value,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub last_auth_at: chrono::DateTime<chrono::Utc>,
    pub is_active: bool,
}


// ------- Implementations ------- //


impl From<Row> for SsoSessionInfo {
    fn from(row: Row) -> Self {
        SsoSessionInfo {
            session_id: row.get("session_id"),
            device_details: row.get("device_details"),
            created_at: row.get("created_at"),
            last_auth_at: row.get("last_auth_at"),
            is_active: row.get("is_active"),

        }
    }
}


impl SsoSessionInfo {
    pub fn from_rows(rows: Vec<Row>) -> Vec<Self> {
        rows.into_iter().map(|row| SsoSessionInfo::from(row)).collect()
    }
}
