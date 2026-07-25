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


use actix_web::{HttpMessage, HttpRequest, HttpResponse, delete, get, post, web};
use crate::database::sso_sessions::{list_sso_sessions, sso_session_delete, sso_session_update_active_status};
use crate::models::errors::ApiResponse;
use crate::models::user::SessionUser;
use crate::state::AppState;


#[get("/list")]
pub async fn get_all_sso_sessions(request: HttpRequest, state: web::Data<AppState>) -> ApiResponse {
    // Get SessionUser from request extensions
    let ext = request.extensions();
    let session_user = ext.get::<SessionUser>().unwrap();

    // Get the maintenance status from the database
    let sso_sessions = list_sso_sessions(&state.pg_pool, &session_user.email).await?;

    Ok(HttpResponse::Ok().json(sso_sessions))
}


#[delete("/delete/{session_id}")]
pub async fn delete_sso_session(request: HttpRequest, session_id: web::Path<uuid::Uuid>, state: web::Data<AppState>) -> ApiResponse {
    // Get SessionUser from request extensions
    let ext = request.extensions();
    let session_user = ext.get::<SessionUser>().unwrap();
    let session_id = session_id.into_inner();

    // Delete the SSO session from the database
    sso_session_delete(&state.pg_pool, session_id, &session_user.email).await?;

    // TODO: Add SSO API call to delete the session cache from the SSO server

    Ok(HttpResponse::Ok().json("SSO session deleted successfully"))
}


#[post("/status/{session_id}/{is_active}")]
pub async fn update_sso_session_status(request: HttpRequest, path: web::Path<(uuid::Uuid, bool)>, state: web::Data<AppState>) -> ApiResponse {
    // Get SessionUser from request extensions
    let ext = request.extensions();
    let session_user = ext.get::<SessionUser>().unwrap();
    let (session_id, is_active) = path.into_inner();

    sso_session_update_active_status(&state.pg_pool, session_id, &session_user.email, is_active).await?;

    Ok(HttpResponse::Ok().json("SSO session status updated successfully"))
}
