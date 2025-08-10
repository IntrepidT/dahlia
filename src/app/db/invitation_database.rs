use leptos::prelude::*;
cfg_if::cfg_if! {
    if #[cfg(feature = "ssr")] {
        use crate::app::models::invitation::{
            generate_invitation_code, generate_verification_code, CreateInvitationRequest, Invitation,
            VerificationCode, VerificationType,
        };
        use chrono::{DateTime, Duration, Utc};
        use log::{debug, error, info};
        use sqlx::{PgPool, Row};

        pub async fn create_invitation(
            pool: &PgPool,
            request: CreateInvitationRequest,
            invited_by_user_id= Option<i64>,
        ) -> Result<Invitation, sqlx::Error> {
            let code = generate_invitation_code();
            let expires_at = if request.expires_in_days > 0 {
                Some(Utc::now() + Duration::days(request.expires_in_days as i64))
            } else {
                None
            };

            let row = sqlx::query(
                r#"
                INSERT INTO invitations (code, school_name, invited_by_user_id, role, max_uses, expires_at)
                VALUES ($1, $2, $3, $4, $5, $6)
                RETURNING id, code, school_name, invited_by_user_id, role, max_uses, current_uses, expires_at, created_at
                "#,
            )
            .bind(&code)
            .bind(&request.school_name)
            .bind(invited_by_user_id)
            .bind(&request.role)
            .bind(request.max_uses)
            .bind(expires_at)
            .fetch_one(pool)
            .await?;

            Ok(Invitation {
                id= row.get("id"),
                code: row.get("code"),
                school_name: row.get("school_name"),
                invited_by_user_id= row.get("invited_by_user_id"),
                role: row.get("role"),
                max_uses: row.get("max_uses"),
                current_uses: row.get("current_uses"),
                expires_at: row.get("expires_at"),
                created_at: row.get("created_at"),
            })
        }

        pub async fn get_invitation_by_code(
            pool: &PgPool,
            code: &str,
        ) -> Result<Option<Invitation>, sqlx::Error> {
            let row = sqlx::query(
                "SELECT id, code, school_name, invited_by_user_id, role, max_uses, current_uses, expires_at, created_at FROM invitations WHERE code = $1"
            )
            .bind(code)
            .fetch_optional(pool)
            .await?;

            Ok(row.map(|r| Invitation {
                id= r.get("id"),
                code: r.get("code"),
                school_name: r.get("school_name"),
                invited_by_user_id= r.get("invited_by_user_id"),
                role: r.get("role"),
                max_uses: r.get("max_uses"),
                current_uses: r.get("current_uses"),
                expires_at: r.get("expires_at"),
                created_at: r.get("created_at"),
            }))
        }

        pub async fn use_invitation(pool: &PgPool, code: &str) -> Result<bool, sqlx::Error> {
            let result = sqlx::query(
                r#"
                UPDATE invitations 
                SET current_uses = current_uses + 1 
                WHERE code = $1 
                  AND current_uses < max_uses 
                  AND (expires_at IS NULL OR expires_at > NOW())
                "#,
            )
            .bind(code)
            .execute(pool)
            .await?;

            Ok(result.rows_affected() > 0)
        }

        pub async fn get_invitations_by_user(
            pool: &PgPool,
            user_id= i64,
        ) -> Result<Vec<Invitation>, sqlx::Error> {
            let rows = sqlx::query(
                r#"
                SELECT id, code, school_name, invited_by_user_id, role, max_uses, current_uses, expires_at, created_at 
                FROM invitations 
                WHERE invited_by_user_id = $1 
                ORDER BY created_at DESC
                "#,
            )
            .bind(user_id)
            .fetch_all(pool)
            .await?;

            Ok(rows
                .into_iter()
                .map(|r| Invitation {
                    id= r.get("id"),
                    code: r.get("code"),
                    school_name: r.get("school_name"),
                    invited_by_user_id= r.get("invited_by_user_id"),
                    role: r.get("role"),
                    max_uses: r.get("max_uses"),
                    current_uses: r.get("current_uses"),
                    expires_at: r.get("expires_at"),
                    created_at: r.get("created_at"),
                })
                .collect())
        }

        pub async fn create_verification_code(
            pool: &PgPool,
            user_id= i64,
            verification_type: VerificationType,
        ) -> Result<VerificationCode, sqlx::Error> {
            let code = generate_verification_code();
            let expires_at = Utc::now() + Duration::minutes(10); // 10-minute expiration

            // Invalidate any existing unused codes for this user and type
            sqlx::query(
                "UPDATE verification_codes SET used_at = NOW() WHERE user_id = $1 AND type = $2 AND used_at IS NULL"
            )
            .bind(user_id)
            .bind(verification_type.as_str())
            .execute(pool)
            .await?;

            let row = sqlx::query(
                r#"
                INSERT INTO verification_codes (user_id, code, type, expires_at)
                VALUES ($1, $2, $3, $4)
                RETURNING id, user_id, code, type, expires_at, used_at, created_at
                "#,
            )
            .bind(user_id)
            .bind(&code)
            .bind(verification_type.as_str())
            .bind(expires_at)
            .fetch_one(pool)
            .await?;

            Ok(VerificationCode {
                id= row.get("id"),
                user_id= row.get("user_id"),
                code: row.get("code"),
                verification_type: VerificationType::from_str(row.get("type")).unwrap(),
                expires_at: row.get("expires_at"),
                used_at: row.get("used_at"),
                created_at: row.get("created_at"),
            })
        }

        pub async fn validate_verification_code(
            pool: &PgPool,
            user_id= i64,
            code: &str,
            verification_type: VerificationType,
        ) -> Result<bool, sqlx::Error> {
            // Find the code and mark it as used if valid
            let result = sqlx::query(
                r#"
                UPDATE verification_codes 
                SET used_at = NOW() 
                WHERE user_id = $1 
                  AND code = $2 
                  AND type = $3 
                  AND used_at IS NULL 
                  AND expires_at > NOW()
                "#,
            )
            .bind(user_id)
            .bind(code)
            .bind(verification_type.as_str())
            .execute(pool)
            .await?;

            let is_valid = result.rows_affected() > 0;

            if is_valid {
                match verification_type {
                    VerificationType::Email => {
                        sqlx::query("UPDATE users SET email_verified = true WHERE id = $1")
                            .bind(user_id)
                            .execute(pool)
                            .await?;
                        info!("Email verified for user_id= {}", user_id);
                    }
                    VerificationType::Phone => {
                        sqlx::query("UPDATE users SET phone_verified = true WHERE id = $1")
                            .bind(user_id)
                            .execute(pool)
                            .await?;
                        info!("Phone verified for user_id= {}", user_id);
                    }
                }
            }

            Ok(is_valid)
        }

        pub async fn cleanup_expired_codes(pool: &PgPool) -> Result<u64, sqlx::Error> {
            let result = sqlx::query("DELETE FROM verification_codes WHERE expires_at < NOW()")
                .execute(pool)
                .await?;

            debug!(
                "Cleaned up {} expired verification codes",
                result.rows_affected()
            );
            Ok(result.rows_affected())
        }

        pub async fn update_user_phone_number(
            pool: &PgPool,
            user_id= i64,
            phone_number: &str,
        ) -> Result<(), sqlx::Error> {
            sqlx::query("UPDATE users SET phone_number = $1 WHERE id = $2")
                .bind(phone_number)
                .bind(user_id)
                .execute(pool)
                .await?;

            Ok(())
        }

        pub async fn get_user_verification_status(
            pool: &PgPool,
            user_id= i64,
        ) -> Result<Option<(bool, bool)>, sqlx::Error> {
            let row = sqlx::query("SELECT email_verified, phone_verified FROM users WHERE id = $1")
                .bind(user_id)
                .fetch_optional(pool)
                .await?;

            Ok(row.map(|r| {
                (
                    r.get::<Option<bool>, _>("email_verified").unwrap_or(false),
                    r.get::<Option<bool>, _>("phone_verified").unwrap_or(false),
                )
            }))
        }

        pub async fn is_user_fully_verified(pool: &PgPool, user_id= i64) -> Result<bool, sqlx::Error> {
            let row = sqlx::query("SELECT email_verified, phone_verified FROM users WHERE id = $1")
                .bind(user_id)
                .fetch_optional(pool)
                .await?;

            Ok(row
                .map(|r| {
                    r.get::<Option<bool>, _>("email_verified").unwrap_or(false)
                        && r.get::<Option<bool>, _>("phone_verified").unwrap_or(false)
                })
                .unwrap_or(false))
        }

        // ===== ADMIN QUERIES =====

        pub async fn get_all_invitations_for_admin(
            pool: &PgPool,
            limit: i64,
            offset: i64,
        ) -> Result<Vec<Invitation>, sqlx::Error> {
            let rows = sqlx::query(
                r#"
                SELECT i.id, i.code, i.school_name, i.invited_by_user_id, i.role, 
                       i.max_uses, i.current_uses, i.expires_at, i.created_at
                FROM invitations i
                ORDER BY i.created_at DESC
                LIMIT $1 OFFSET $2
                "#,
            )
            .bind(limit)
            .bind(offset)
            .fetch_all(pool)
            .await?;

            Ok(rows
                .into_iter()
                .map(|r| Invitation {
                    id= r.get("id"),
                    code: r.get("code"),
                    school_name: r.get("school_name"),
                    invited_by_user_id= r.get("invited_by_user_id"),
                    role: r.get("role"),
                    max_uses: r.get("max_uses"),
                    current_uses: r.get("current_uses"),
                    expires_at: r.get("expires_at"),
                    created_at: r.get("created_at"),
                })
                .collect())
        }

        pub async fn delete_invitation(pool: &PgPool, invitation_id= i64) -> Result<bool, sqlx::Error> {
            let result = sqlx::query("DELETE FROM invitations WHERE id = $1")
                .bind(invitation_id)
                .execute(pool)
                .await?;

            Ok(result.rows_affected() > 0)
        }

        pub async fn count_recent_verification_codes(
            pool: &PgPool,
            user_id= i64,
            verification_type: VerificationType,
            minutes: i32,
        ) -> Result<i64, sqlx::Error> {
            let since = Utc::now() - Duration::minutes(minutes as i64);

            let count = sqlx::query(
                "SELECT COUNT(*) as count FROM verification_codes WHERE user_id = $1 AND type = $2 AND created_at > $3"
            )
            .bind(user_id)
            .bind(verification_type.as_str())
            .bind(since)
            .fetch_one(pool)
            .await?;

            Ok(count.get::<Option<i64>, _>("count").unwrap_or(0))
        }
    }
}
