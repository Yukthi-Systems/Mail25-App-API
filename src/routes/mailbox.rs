use crate::database::mailbox_sessions::{list_mailbox_sessions, toggle_mailbox_session, mailbox_session_delete};
use actix_web::{HttpMessage, HttpRequest, HttpResponse, delete, get, patch, web};
use crate::handlers::sso_api::clear_mailbox_session_cache;
use crate::models::errors::ApiResponse;
use crate::models::user::SessionUser;
use crate::state::AppState;


#[get("/list")]
pub async fn get_all_mailbox_client_sessions(request: HttpRequest, state: web::Data<AppState>) -> ApiResponse {
    // Get SessionUser from request extensions
    let ext = request.extensions();
    let session_user = ext.get::<SessionUser>().unwrap();

    // Get the maintenance status from the database
    let mailbox_sessions = list_mailbox_sessions(&state.pg_pool, &session_user.email).await?;

    Ok(HttpResponse::Ok().json(mailbox_sessions))
}


#[patch("/update/{origin_ip}/{enable}")]
pub async fn update_mailbox_client_session(request: HttpRequest, path: web::Path<(String, bool)>, state: web::Data<AppState>) -> ApiResponse {
    // Get SessionUser from request extensions
    let ext = request.extensions();
    let session_user = ext.get::<SessionUser>().unwrap();
    let (origin_ip, enable) = path.into_inner();

    // Toggle the mailbox session in the database
    toggle_mailbox_session(&state.pg_pool, &origin_ip, &session_user.email, enable).await?;

    // Clear the cache in SSO API
    clear_mailbox_session_cache(&state.config.sso_api_settings, &origin_ip, &session_user.email).await?;

    Ok(HttpResponse::Ok().json("Mailbox session toggled successfully"))
}


#[delete("/delete/{origin_ip}")]
pub async fn delete_mailbox_client_session(request: HttpRequest, origin_ip: web::Path<String>, state: web::Data<AppState>) -> ApiResponse {
    // Get SessionUser from request extensions
    let ext = request.extensions();
    let session_user = ext.get::<SessionUser>().unwrap();
    let origin_ip = origin_ip.into_inner();

    // Delete the mailbox session from the database
    mailbox_session_delete(&state.pg_pool, &origin_ip, &session_user.email).await?;

    // Clear the cache in SSO API
    clear_mailbox_session_cache(&state.config.sso_api_settings, &origin_ip, &session_user.email).await?;

    Ok(HttpResponse::Ok().json("Mailbox session deleted successfully"))
}
