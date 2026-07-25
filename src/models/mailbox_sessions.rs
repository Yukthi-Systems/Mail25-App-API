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
pub struct MailBoxSessionInfo {
    pub origin_ip: String,
    pub geo_ip_location: serde_json::Value,
    pub is_active: bool,
    pub attempted_at: chrono::DateTime<chrono::Utc>,
    pub session_expires_at: chrono::DateTime<chrono::Utc>,
}


// ------- Implementations ------- //


impl From<Row> for MailBoxSessionInfo {
    fn from(row: Row) -> Self {
        MailBoxSessionInfo {
            origin_ip: row.get("origin_ip"),
            geo_ip_location: row.get("geo_ip_location"),
            is_active: row.get("is_active"),
            attempted_at: row.get("attempted_at"),
            session_expires_at: row.get("session_expires_at"),
        }
    }
}


impl MailBoxSessionInfo {
    pub fn from_rows(rows: Vec<Row>) -> Vec<Self> {
        rows.into_iter().map(|row| MailBoxSessionInfo::from(row)).collect()
    }
}
