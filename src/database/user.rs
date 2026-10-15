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


use crate::models::user::{UserInfo, SessionUser, FullIdentity};
use deadpool_postgres::{
    PoolError as PgError,
    Pool as PgPool
};


/// Fetch user by email, only if the user and their organization are active
pub async fn get_user_by_email(db_pool: &PgPool, email: &str) -> Result<Option<UserInfo>, PgError> {
    let client = db_pool.get().await?;

    let row = client
        .query_opt(
            r#"
            SELECT
                eid.email,
                eid.domain_name,
                eid.first_name,
                eid.last_name,

                eid.primary_phone,
                eid.secondary_email,
                eid.is_app_2fa_enabled,
                eid.is_sms_2fa_enabled,
                eid.is_email_2fa_enabled,

                eid.restriction_policy_id,

                o.organization_id,
                o.organization_name,

                rp.policy_id,
                rp.policy_name,
                rp.policy_description,
                rp.ip_restriction,
                rp.geo_restriction,
                rp.is_active AS restriction_policy_active

            FROM email_identities AS eid

            INNER JOIN domains AS d
                ON d.domain_name = eid.domain_name

            INNER JOIN organizations AS o
                ON o.organization_id = d.managed_by

            LEFT JOIN restriction_policies AS rp
                ON rp.policy_id = eid.restriction_policy_id
                AND rp.is_active = TRUE

            WHERE
                eid.email = $1

                -- Email Identity checks
                AND eid.is_enabled = TRUE
                AND eid.is_password_expired = FALSE

                -- domain checks
                AND d.is_active = TRUE

                -- owning org check
                AND o.is_active = TRUE

                -- parent org checks
                AND NOT EXISTS (
                    SELECT 1
                    FROM organizations parent_org
                    WHERE parent_org.organization_id::text = ANY(o.hierarchy_path)
                    AND parent_org.is_active = FALSE
                )
            "#,
            &[&email],
        )
        .await?;

    Ok(row.map(UserInfo::from))
}


pub async fn delete_app_session(db_pool: &PgPool, session_id: &uuid::Uuid) -> Result<(), PgError> {
    let client = db_pool.get().await?;

    client
        .execute(
            r#"
            DELETE FROM mail25_app_sessions WHERE session_id = $1
            "#,
            &[&session_id],
        )
        .await?;

    Ok(())
}


pub async fn identity_checks(db_pool: &PgPool, phone_number: &str, email: &str) -> Result<bool, PgError> {
    let client = db_pool.get().await?;

    let row = client
        .query_opt(
            r#"
            SELECT 1
            FROM email_identities AS eid
            INNER JOIN domains AS d ON d.domain_name = eid.domain_name
            INNER JOIN organizations AS o ON o.organization_id = d.managed_by
            WHERE eid.primary_phone = $1
              AND eid.email = $2
              AND eid.is_enabled = TRUE
              AND eid.is_password_expired = FALSE
              AND d.is_active = TRUE
              AND o.is_active = TRUE
              AND NOT EXISTS (
                  SELECT 1
                  FROM organizations parent_org
                  WHERE parent_org.organization_id::text = ANY(o.hierarchy_path)
                    AND parent_org.is_active = FALSE
              )
            "#,
            &[&phone_number, &email],
        )
        .await?;

    Ok(row.is_some())
}


pub async fn create_app_session(db_pool: &PgPool, user_session: &SessionUser) -> Result<(), PgError> {
    let client = db_pool.get().await?;

    client
        .execute(
            r#"
            INSERT INTO 
                mail25_app_sessions (
                    session_id,
                    email,
                    domain_name,
                    phone,
                    fcm_token,
                    device_details
                )
            VALUES ($1, $2, $3, $4, $5, $6)
            "#,
            &[
                &user_session.session_id,
                &user_session.email,
                &user_session.domain_name,
                &user_session.primary_phone,
                &user_session.fcm_token,
                &user_session.device_details,
            ])
            .await?;

    Ok(())
}


pub async fn get_app_session(db_pool: &PgPool, session_id: &uuid::Uuid) -> Result<Option<SessionUser>, PgError> {
    let client = db_pool.get().await?;

    let row = client
        .query_opt(
            r#"
            SELECT
                s.session_id,

                s.email,
                s.domain_name,

                eid.first_name,
                eid.last_name,
                eid.primary_phone,
                eid.secondary_email,

                s.device_details,
                s.fcm_token,

                o.organization_id,
                o.organization_name

            FROM mail25_app_sessions AS s

            INNER JOIN email_identities AS eid
                ON eid.email = s.email
            AND eid.domain_name = s.domain_name

            INNER JOIN domains AS d
                ON d.domain_name = s.domain_name

            INNER JOIN organizations AS o
                ON o.organization_id = d.managed_by

            WHERE
                s.session_id = $1

                AND eid.is_enabled = TRUE
                AND eid.is_password_expired = FALSE

                AND d.is_active = TRUE

                AND o.is_active = TRUE

                AND NOT EXISTS (
                    SELECT 1
                    FROM organizations AS parent_org
                    WHERE parent_org.organization_id::text = ANY(o.hierarchy_path)
                    AND parent_org.is_active = FALSE
                )
            "#,
            &[session_id],
        )
        .await?;

    Ok(row.map(SessionUser::from))
}


pub async fn replace_fcm_token(db_pool: &PgPool, session_id: &uuid::Uuid, new_fcm_token: &str) -> Result<(), PgError> {
    let client = db_pool.get().await?;

    client
        .execute(
            r#"
            UPDATE mail25_app_sessions
            SET fcm_token = $1
            WHERE session_id = $2
            "#,
            &[&new_fcm_token, &session_id],
        )
        .await?;

    Ok(())
}


pub async fn validate_and_update_session(db_pool: &PgPool, session_id: &uuid::Uuid) -> Result<bool, PgError> {
    let client = db_pool.get().await?;

    // Update the last active timestamp for the session and check if the session exists or not
    let row = client
        .query_opt(
            r#"
            UPDATE mail25_app_sessions
            SET last_active_at = CURRENT_TIMESTAMP
            WHERE session_id = $1
            RETURNING session_id
            "#,
            &[session_id],
        )
        .await?;

    Ok(row.is_some())
}


pub async fn get_full_identity(db_pool: &PgPool, email: &str) -> Result<FullIdentity, PgError> {
    let client = db_pool.get().await?;

    let row = client
        .query_one(
            r#"
            SELECT
                eid.email,
                eid.first_name,
                eid.last_name,
                eid.primary_phone,
                eid.secondary_email,
                eid.is_app_2fa_enabled,
                eid.is_sms_2fa_enabled,
                eid.is_email_2fa_enabled,
                eid.is_password_expired,
                eid.password_updated_at,
                eid.updated_at,
                dep.department_name,
                dep.details AS department_details
            FROM email_identities AS eid
            LEFT JOIN departments AS dep
                ON dep.department_id = eid.department_id
            WHERE eid.email = $1
              AND eid.is_enabled = TRUE
              AND eid.is_password_expired = FALSE
            "#,
            &[&email],
        )
        .await?;

    Ok(FullIdentity::from(row))
}


pub async fn update_identity_info(
    db_pool: &PgPool,
    email: &str,
    first_name: &str,
    last_name: &str,
    secondary_email: &str,
    enable_app_2fa: bool,
    enable_sms_2fa: bool,
    enable_email_2fa: bool,
) -> Result<(), PgError> {
    let client = db_pool.get().await?;

    // Note: The users can only enable the 2FA methods for their own account, but not disable them

    client
        .execute(
            r#"
            UPDATE email_identities
            SET first_name = $1,
                last_name = $2,
                secondary_email = $3,
                is_app_2fa_enabled = CASE WHEN is_app_2fa_enabled = FALSE THEN $4 ELSE is_app_2fa_enabled END,
                is_sms_2fa_enabled = CASE WHEN is_sms_2fa_enabled = FALSE THEN $5 ELSE is_sms_2fa_enabled END,
                is_email_2fa_enabled = CASE WHEN is_email_2fa_enabled = FALSE THEN $6 ELSE is_email_2fa_enabled END,
                updated_at = CURRENT_TIMESTAMP
            WHERE email = $7
            "#,
            &[
                &first_name,
                &last_name,
                &secondary_email,
                &enable_app_2fa,
                &enable_sms_2fa,
                &enable_email_2fa,
                &email,
            ],
        )
        .await?;

    Ok(())
}
