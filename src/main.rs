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


use routes::{health, user, maintenance, auth_app, sso, mailbox};
use actix_web::web::scope as actix_scope;
use actix_web::middleware::from_fn;
use actix_web::{App, HttpServer};
use std::env::var as env_var;
use actix_cors::Cors;

mod middleware;
mod database;
mod handlers;
mod models;
mod routes;
mod cache;
mod state;


#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let app_state = state::initialize().await;

    // Start the Actix web server
    HttpServer::new(move || {
        App::new()
            .app_data(app_state.clone())
            .wrap(Cors::default()
                .allowed_origin_fn(state::allow_origins)
                .allowed_methods(vec!["GET", "POST", "PATCH", "DELETE"])
                .supports_credentials()
                .allow_any_header()
                .max_age(420)
            )
            .service(
                actix_scope("/health")
                .service(health::api_health_check)
            )
            .service(
                actix_scope("/auth")
                .service(user::login)
                .service(user::validate_session_otp)
                .service(
                    actix_scope("")
                    .wrap(from_fn(middleware::auth::auth_check))
                    .service(user::update_fcm_token)
                    .service(user::validate_session)
                    .service(user::logout)
                )
            )
            .service(
                actix_scope("/user")
                .wrap(from_fn(middleware::auth::auth_check))
                .service(user::get_identity)
                // .service(user::update_identity)
                // .service(user::update_password)
            )
            .service(
                actix_scope("/mailbox/session")
                .wrap(from_fn(middleware::auth::auth_check))
                .service(mailbox::get_all_mailbox_client_sessions)
                .service(mailbox::update_mailbox_client_session)
                .service(mailbox::delete_mailbox_client_session)
            )
            .service(
                actix_scope("/sso/session")
                .wrap(from_fn(middleware::auth::auth_check))
                .service(sso::update_sso_session_status)
                .service(sso::get_all_sso_sessions)
                .service(sso::delete_sso_session)
            )
            .service(
                actix_scope("/mail25-app/session")
                .wrap(from_fn(middleware::auth::auth_check))
                .service(auth_app::get_all_app_sessions)
                .service(auth_app::delete_app_session)
            )
            .service(
                actix_scope("/maintenance")
                .wrap(from_fn(middleware::auth::auth_check))
                .service(maintenance::get_maintenance_list)
            )
            // .service(
            //     actix_scope("/totp")
            //     .wrap(from_fn(middleware::auth::auth_check))
            //     .service(totp::list_all_totp)
            //     .service(totp::delete_totp)
            //     .service(totp::create_totp)
            //     .service(totp::edit_totp)
            // )
    })
    .bind(("0.0.0.0", 8686))?
    .workers(env_var("API_WORKERS_COUNT").unwrap_or("4".to_string()).parse().unwrap())
    .run().await
}
