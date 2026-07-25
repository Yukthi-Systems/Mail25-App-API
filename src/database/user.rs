use crate::models::user::{UserInfo, SessionUser};
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
