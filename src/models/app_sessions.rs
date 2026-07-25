use tokio_postgres::row::Row;
use serde::Serialize;


#[derive(Serialize)]
pub struct AppSessionInfo {
    pub session_id: uuid::Uuid,
    pub email: String,
    pub domain_name: String,
    pub device_details: serde_json::Value,
    pub last_active_at: chrono::DateTime<chrono::Utc>,
}


// ------- Implementations ------- //


impl From<Row> for AppSessionInfo {
    fn from(row: Row) -> Self {
        AppSessionInfo {
            session_id: row.get("session_id"),
            email: row.get("email"),
            domain_name: row.get("domain_name"),
            device_details: row.get("device_details"),
            last_active_at: row.get("last_active_at"),

        }
    }
}


impl AppSessionInfo {
    pub fn from_rows(rows: Vec<Row>) -> Vec<Self> {
        rows.into_iter().map(|row| AppSessionInfo::from(row)).collect()
    }
}
