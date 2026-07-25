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


use actix_web::{HttpMessage, HttpRequest, HttpResponse, delete, get, web};
use crate::database::app_sessions::{list_app_sessions, app_session_delete};
use crate::models::errors::{ApiResponse, AppError};
use crate::models::user::SessionUser;
use crate::state::AppState;


#[get("/list")]
pub async fn get_all_app_sessions(request: HttpRequest, state: web::Data<AppState>) -> ApiResponse {
    // Get SessionUser from request extensions
    let ext = request.extensions();
    let session_user = ext.get::<SessionUser>().unwrap();

    // Get the maintenance status from the database
    let app_sessions = list_app_sessions(&state.pg_pool, &session_user.primary_phone).await?;

    Ok(HttpResponse::Ok().json(app_sessions))
}


#[delete("/delete/{session_id}")]
pub async fn delete_app_session(request: HttpRequest, session_id: web::Path<uuid::Uuid>, state: web::Data<AppState>) -> ApiResponse {
    // Get SessionUser from request extensions
    let ext = request.extensions();
    let session_user = ext.get::<SessionUser>().unwrap();
    let session_id = session_id.into_inner();

    // It can not be the same user deleting their own session
    if session_user.session_id == session_id {
        return Err(AppError::Unprocessable("You cannot delete your own session".into()));
    }

    // Delete the app session from the database
    app_session_delete(&state.pg_pool, session_id, &session_user.primary_phone).await?;

    Ok(HttpResponse::Ok().json("App session deleted successfully"))
}
