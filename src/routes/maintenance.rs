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


use crate::database::maintenance::list_maintenance_status;
use actix_web::{get, web, HttpResponse};
use crate::models::errors::ApiResponse;
use crate::state::AppState;


#[get("/list")]
pub async fn get_maintenance_list(state: web::Data<AppState>) -> ApiResponse {
    // Get the maintenance status from the database
    let maintenance_status = list_maintenance_status(&state.pg_pool).await?;

    Ok(HttpResponse::Ok().json(maintenance_status))
}
