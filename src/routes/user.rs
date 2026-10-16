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


use crate::database::user::{get_user_by_email, create_app_session, delete_app_session, identity_checks, replace_fcm_token, validate_and_update_session, get_full_identity, update_identity_info, update_identity_password, get_full_app_info, replace_app_info};
use actix_web::{HttpMessage, HttpRequest, HttpResponse, delete, get, patch, post, web};
use crate::cache::handler::{set_redis_cache, delete_redis_cache, get_redis_cache};
use crate::handlers::rmq::send_notification_to_rmq;
use crate::models::errors::{ApiResponse, AppError};
use crate::handlers::auth::PasswordHasher;
use crate::models::user::SessionUser;
use serde::{Deserialize, Serialize};
use crate::state::AppState;
use uuid::Uuid;


// ------- Request and Response Structs ------- //

// Structs required only for request and response handling,
// so they are defined here in the route file instead of models/user.rs

// Struct for login request
#[derive(Deserialize)]
struct LoginRequest {
    phone_number: String,
    email_prefix: String,
    domain_name: String,
    device_details: serde_json::Value,
}


// Struct for OTP validation request
#[derive(Deserialize)]
struct OtpValidateRequest {
    session_id: Uuid,
    otp_code: u32,
    fcm_token: String,
}


// Internal struct for OTP validation data stored in Redis
#[derive(Serialize, Deserialize)]
struct ValidateSessionRequest {
    otp_code: u32,
    email: String,
    domain_name: String,
    phone_number: String,
    device_details: serde_json::Value,
}


// Struct for updating identity information request
#[derive(Deserialize)]
struct UpdateIdentityInfoRequest {
    first_name: String,
    last_name: String,
    secondary_email: String,
    enable_app_2fa: bool,
    enable_sms_2fa: bool,
    enable_email_2fa: bool,
}


#[post("/login")]
pub async fn login(login_request: web::Json<LoginRequest>, state: web::Data<AppState>) -> ApiResponse {
    let login_request = login_request.into_inner();
    let email = format!("{}@{}", login_request.email_prefix, login_request.domain_name).to_lowercase();
    let session_id = Uuid::new_v4();
    let otp_code = rand::random_range(100000..999999); // Generate a random 6-digit OTP code (Always 6 digits)

    // Check if the user is associated with the provided phone number and email along with active orgs and stuff
    if !identity_checks(&state.pg_pool, &login_request.phone_number, &email).await? {
        return Err(AppError::Unauthorized(
            "User does not exist or is inactive or is not associated with the provided phone number, please check your email or contact support".to_string()
        ));
    }

    // Send OTP to provided phone number
    send_notification_to_rmq(
        &state.config.rmq_settings,
        "sms",
        &serde_json::json!({
            "to": login_request.phone_number,
            "template": "aio_otp",
            "variables": {
                "to": login_request.phone_number,
                "otp": otp_code
            }
        })
    ).await?;

    // Save the OTP and session ID in Redis with a TTL of 3 minutes (in seconds)
    let otp_data = ValidateSessionRequest {
        otp_code,
        email,
        domain_name: login_request.domain_name,
        phone_number: login_request.phone_number,
        device_details: login_request.device_details
    };

    let otp_key = format!("otp:{}", session_id);
    set_redis_cache(state.redis_cache.clone(), &otp_key, &otp_data, 3 * 60).await?;

    // Return response indicating that OTP has been sent
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "message": "OTP has been sent to your phone number",
        "session_id": session_id
    })))
}


#[post("/validate/otp")]
pub async fn validate_session_otp(otp_validate_request: web::Json<OtpValidateRequest>, state: web::Data<AppState>) -> ApiResponse {
    let otp_validate_request = otp_validate_request.into_inner();
    let otp_key = format!("otp:{}", otp_validate_request.session_id);

    // Retrieve OTP data from Redis
    let otp_data: Option<ValidateSessionRequest> = get_redis_cache(state.redis_cache.clone(), &otp_key).await?;
    if otp_data.is_none() {
        return Err(AppError::Unauthorized("Invalid or expired OTP session".to_string()));
    }
    let otp_data = otp_data.unwrap();

    // Validate the provided OTP code
    if otp_data.otp_code != otp_validate_request.otp_code {
        // Delete the OTP session from Redis to prevent brute-force attempts
        delete_redis_cache(state.redis_cache.clone(), &otp_key).await?;

        return Err(AppError::Unauthorized("Invalid OTP code".to_string()));
    }

    // Fetch user details from the database using the email and domain name from OTP data
    let user_info = get_user_by_email(&state.pg_pool, &otp_data.email).await?;
    if user_info.is_none() {
        return Err(AppError::Unauthorized("User not found".to_string()));
    }
    let user_info = user_info.unwrap();

    // Construct SessionUser from OTP data
    let session_user = SessionUser::create(
        otp_validate_request.session_id,
        &user_info,
        &otp_data.device_details,
        &otp_validate_request.fcm_token
    );

    // Since OTP is valid and the session is valid, create a new app session in the database and save it in Redis with a TTL of 3 Days (in seconds)
    create_app_session(&state.pg_pool, &session_user).await?;

    // Save session in Redis with TTL of 3 days (in seconds)
    let session_key = format!("app:session:{}", session_user.session_id);
    set_redis_cache(state.redis_cache.clone(), &session_key, &session_user, 3 * 24 * 60 * 60).await?;

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "message": "OTP validated successfully and session is created",
        "details": session_user
    })))
}


#[get("/session")]
pub async fn validate_session(request: HttpRequest, state: web::Data<AppState>) -> ApiResponse {
    // Get SessionUser from request extensions
    let ext = request.extensions();
    let session_user = ext.get::<SessionUser>().unwrap();

    // Validate the actual DB session
    if !validate_and_update_session(&state.pg_pool, &session_user.session_id).await? {
        // Remove the session from Redis if it's invalid or expired
        delete_redis_cache(state.redis_cache.clone(), &format!("app:session:{}", session_user.session_id)).await?;

        return Err(AppError::Unauthorized("Session is invalid or expired".to_string()));
    }

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "message": "Session is valid",
        "email": session_user.email,
        "first_name": session_user.first_name,
        "last_name": session_user.last_name,
        "organization_id": session_user.organization_id,
        "organization_name": session_user.organization_name
    })))
}


#[delete("/logout")]
pub async fn logout(request: HttpRequest, state: web::Data<AppState>) -> ApiResponse {
    // Get SessionUser from request extensions
    let ext = request.extensions();
    let session_user = ext.get::<SessionUser>().unwrap();

    // Delete session from Redis
    let session_key = format!("app:session:{}", session_user.session_id);
    delete_redis_cache(state.redis_cache.clone(), &session_key).await?;

    // Delete session from DB
    delete_app_session(&state.pg_pool, &session_user.session_id).await?;

    log::debug!("User {} logged out successfully and session {} deleted", session_user.email, session_user.session_id);

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "message": "Logged out successfully"
    })))
}


#[patch("/fcm-token")]
pub async fn update_fcm_token(request: HttpRequest, state: web::Data<AppState>, fcm_token: web::Json<String>) -> ApiResponse {
    // Get SessionUser from request extensions
    let ext = request.extensions();
    let session_user = ext.get::<SessionUser>().unwrap();

    // Update FCM token in the database
    replace_fcm_token(&state.pg_pool, &session_user.session_id, &fcm_token).await?;

    // Update FCM token in Redis (But not required as we don't use FCM token from Redis)

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "message": "FCM token updated successfully"
    })))
}


#[get("/identity")]
pub async fn get_identity(request: HttpRequest, state: web::Data<AppState>) -> ApiResponse {
    // Get SessionUser from request extensions
    let ext = request.extensions();
    let session_user = ext.get::<SessionUser>().unwrap();

    let identity_info = get_full_identity(&state.pg_pool, &session_user.email).await?;

    Ok(HttpResponse::Ok().json(identity_info))
}


#[post("/identity/update")]
pub async fn update_identity(request: HttpRequest, body: web::Json<UpdateIdentityInfoRequest>, state: web::Data<AppState>) -> ApiResponse {
    // Get SessionUser from request extensions
    let ext = request.extensions();
    let session_user = ext.get::<SessionUser>().unwrap();

    // Note: Can not update or disable the 2FA methods for their own account, only enable them
    // and also the user can not update the primary phone for their own account (Only Admins can)

    // Update identity in the database
    update_identity_info(
        &state.pg_pool,
        &session_user.email,
        &body.first_name,
        &body.last_name,
        &body.secondary_email,
        body.enable_app_2fa,
        body.enable_sms_2fa,
        body.enable_email_2fa,
    )
    .await?;

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "message": "Identity updated successfully"
    })))
}


#[patch("/password")]
pub async fn update_password(request: HttpRequest, body: web::Json<PasswordHasher>, state: web::Data<AppState>) -> ApiResponse {
    // Get SessionUser from request extensions
    let ext = request.extensions();
    let session_user = ext.get::<SessionUser>().unwrap();
    let password_hasher = body.into_inner();

    // Validate the new password before updating it in the database
    password_hasher.validate()?;

    let bcrypt_hash = password_hasher.generate_bcrypt_hash();
    let ssha1_hash = password_hasher.generate_ssha1_hash();

    // Update password in the database
    update_identity_password(
        &state.pg_pool,
        &session_user.email,
        &bcrypt_hash,
        &ssha1_hash,
    )
    .await?;

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "message": "Password updated successfully"
    })))
}


#[get("/app-info")]
pub async fn get_app_info(request: HttpRequest, state: web::Data<AppState>) -> ApiResponse {
    // Get SessionUser from request extensions
    let ext = request.extensions();
    let session_user = ext.get::<SessionUser>().unwrap();

    let app_info = get_full_app_info(&state.pg_pool, &session_user.email).await?;

    Ok(HttpResponse::Ok().json(app_info))
}


#[post("/app-info")]
pub async fn update_app_info(request: HttpRequest, body: web::Json<serde_json::Value>, state: web::Data<AppState>) -> ApiResponse {
    // Get SessionUser from request extensions
    let ext = request.extensions();
    let session_user = ext.get::<SessionUser>().unwrap();
    let new_app_info = body.into_inner(); 

    // Update app info in the database
    replace_app_info(
        &state.pg_pool,
        &session_user.email,
        &new_app_info
    )
    .await?;

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "message": "App info updated successfully"
    })))
}
