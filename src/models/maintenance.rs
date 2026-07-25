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


use tokio_postgres::row::Row;
use serde::Serialize;


#[derive(Serialize)]
pub struct MaintenanceInfo {
    pub maintenance_id: i32,
    pub title: String,
    pub description: String,
    pub affected: Vec<String>,
    pub severity: String,
    pub maintenance_type: String,
    pub start_time: chrono::DateTime<chrono::Utc>,
    pub end_time: chrono::DateTime<chrono::Utc>,
}


// ------- Implementations ------- //


impl From<Row> for MaintenanceInfo {
    fn from(row: Row) -> Self {
        MaintenanceInfo {
            maintenance_id: row.get("maintenance_id"),
            title: row.get("title"),
            description: row.get("description"),
            affected: row.get("affected"),
            severity: row.get("severity"),
            maintenance_type: row.get("type"),
            start_time: row.get("start_time"),
            end_time: row.get("end_time"),
        }
    }
}


impl MaintenanceInfo {
    pub fn from_rows(rows: Vec<Row>) -> Vec<Self> {
        rows.into_iter().map(|row| MaintenanceInfo::from(row)).collect()
    }
}
