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
