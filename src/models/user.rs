use serde::{Deserialize, Serialize};
use tokio_postgres::row::Row;
use uuid::Uuid;



#[derive(Serialize, Deserialize)]
pub struct SessionUser {
    pub session_id: Uuid,

    pub email: String,
    pub domain_name: String,
    pub first_name: String,
    pub last_name: Option<String>,

    pub primary_phone: String,
    pub secondary_email: Option<String>,

    pub device_details: serde_json::Value,
    pub fcm_token: String,

    pub organization_id: Uuid,
    pub organization_name: String,
}


#[derive(Serialize)]
pub struct RestrictionPolicy {
    pub policy_id: Uuid,
    pub policy_name: String,
    pub policy_description: String,
    pub ip_restriction: Option<Vec<String>>,
    pub geo_restriction: Option<Vec<String>>,
}


#[derive(Serialize)]
pub struct UserInfo {
    pub email: String,
    pub domain_name: String,
    pub first_name: String,
    pub last_name: Option<String>,

    pub primary_phone: String,
    pub secondary_email: Option<String>,

    pub is_app_2fa_enabled: bool,
    pub is_sms_2fa_enabled: bool,
    pub is_email_2fa_enabled: bool,

    pub organization_id: Uuid,
    pub organization_name: String,

    pub restriction_policy: Option<RestrictionPolicy>,
}


// ------- Implementations ------- //


impl From<Row> for SessionUser {
    fn from(row: Row) -> Self {
        SessionUser {
            session_id: row.get("session_id"),

            email: row.get("email"),
            domain_name: row.get("domain_name"),
            first_name: row.get("first_name"),
            last_name: row.get("last_name"),

            primary_phone: row.get("primary_phone"),
            secondary_email: row.get("secondary_email"),
            device_details: row.get("device_details"),
            fcm_token: row.get("fcm_token"),

            organization_id: row.get("organization_id"),
            organization_name: row.get("organization_name"),
        }
    }
}
            

impl SessionUser {
    pub fn create(session_id: Uuid, user_info: &UserInfo, device_details: &serde_json::Value, fcm_token: &str) -> Self {
        SessionUser {
            session_id,

            email: user_info.email.clone(),
            domain_name: user_info.domain_name.clone(),
            first_name: user_info.first_name.clone(),
            last_name: user_info.last_name.clone(),
            primary_phone: user_info.primary_phone.clone(),
            secondary_email: user_info.secondary_email.clone(),
            device_details: device_details.clone(),
            fcm_token: fcm_token.to_string(),

            organization_id: user_info.organization_id,
            organization_name: user_info.organization_name.clone(),
        }
    }
}


impl From<Row> for UserInfo {
    fn from(row: Row) -> Self {
        let restriction_policy = if let Some(policy_id) = row.get::<&str, Option<Uuid>>("policy_id") {
            Some(RestrictionPolicy {
                policy_id,
                policy_name: row.get("policy_name"),
                policy_description: row.get("policy_description"),
                ip_restriction: row.get("ip_restriction"),
                geo_restriction: row.get("geo_restriction"),
            })
        } else {
            None
        };

        UserInfo {
            email: row.get("email"),
            domain_name: row.get("domain_name"),
            first_name: row.get("first_name"),
            last_name: row.get("last_name"),

            primary_phone: row.get("primary_phone"),
            secondary_email: row.get("secondary_email"),

            is_app_2fa_enabled: row.get("is_app_2fa_enabled"),
            is_sms_2fa_enabled: row.get("is_sms_2fa_enabled"),
            is_email_2fa_enabled: row.get("is_email_2fa_enabled"),

            organization_id: row.get("organization_id"),
            organization_name: row.get("organization_name"),

            restriction_policy,
        }
    }
}
