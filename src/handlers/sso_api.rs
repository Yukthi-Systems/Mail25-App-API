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


use crate::models::initial::SSOApiSettings;
use crate::models::errors::AppError;
use reqwest::Client;


/// Function to clear mailbox session cache in SSO API
pub async fn clear_mailbox_session_cache(sso_api_settings: &SSOApiSettings, origin_ip: &str, email: &str) -> Result<(), AppError> {
    let client = Client::new();

    let sso_api_url = format!(
        "{}/internal/mail-service/clear",
        sso_api_settings.base_url
    );

    let response = client.delete(&sso_api_url)
        .header("X-API-Key", &sso_api_settings.api_key)
        .header("content-type", "application/json")
        .json(&serde_json::json!({
            "ip_addr": origin_ip,
            "email": email,
            "domain": email.split('@').nth(1).unwrap_or_default()
        }))
        .send()
        .await?;

    if response.status().is_success() {
        log::info!("Successfully cleared mailbox session cache for IP: {}, Email: {}", origin_ip, email);
        Ok(())
    } else {
        let error_text = response.text().await?;
        log::error!("Failed to clear mailbox session cache for IP: {}, Email: {}. Error: {}", origin_ip, email, error_text);
        Err(AppError::Unprocessable("Failed to clear mailbox session cache".to_string()))
    }
}


/// Function to update the SSO Password
pub async fn update_sso_password(sso_api_settings: &SSOApiSettings, email: &str, new_raw_password: &str) -> Result<(), AppError> {
    let client = Client::new();

    let sso_api_url = format!(
        "{}/internal/password/reset",
        sso_api_settings.base_url
    );

    let response = client.post(&sso_api_url)
        .header("X-API-Key", &sso_api_settings.api_key)
        .header("content-type", "application/json")
        .json(&serde_json::json!({
            "email": email,
            "new_raw_password": new_raw_password
        }))
        .send()
        .await?;

    if response.status().is_success() {
        log::info!("Successfully updated SSO password for Email: {}", email);
        Ok(())
    } else {
        let error_text = response.text().await?;
        log::error!("Failed to update SSO password for Email: {}. Error: {}", email, error_text);
        Err(AppError::Unprocessable("Failed to update SSO password".to_string()))
    }
}
