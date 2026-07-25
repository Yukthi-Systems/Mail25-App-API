use crate::models::maintenance::MaintenanceInfo;
use deadpool_postgres::{
    PoolError as PgError,
    Pool as PgPool
};


// -- Status Maintenance Alerts
// CREATE TABLE maintenance_alerts (
//     maintenance_id SERIAL PRIMARY KEY,
//     is_active BOOLEAN DEFAULT TRUE NOT NULL,

//     title TEXT NOT NULL,
//     description TEXT NOT NULL,
//     affected TEXT[] NOT NULL,

//     severity maintenance_severity_enum NOT NULL,
//     type TEXT NOT NULL,

//     start_time TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL,
//     end_time TIMESTAMPTZ NOT NULL
// );


/// Fetch Maintenance status from the database (All Active)
pub async fn list_maintenance_status(db_pool: &PgPool) -> Result<Vec<MaintenanceInfo>, PgError> {
    let client = db_pool.get().await?;

    let rows = client.query(
        "SELECT maintenance_id, title, description, affected, severity::TEXT, type, start_time, end_time
         FROM maintenance_alerts
         WHERE is_active = TRUE AND end_time > CURRENT_TIMESTAMP
         ORDER BY start_time DESC",
        &[]
    ).await?;

    Ok(MaintenanceInfo::from_rows(rows))
}
