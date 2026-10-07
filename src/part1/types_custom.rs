use crate::part1::schema::{User, UserProfile, UserRole};
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use sqlx::types::Json;
use sqlx::PgPool;
use uuid::Uuid;

/// Demonstrates UUIDv7 generation.
///
/// Unlike random UUIDv4, UUIDv7 embeds a Unix timestamp in its most significant bits,
/// ensuring monotonically increasing values that avoid B-Tree index fragmentation and
/// random I/O write amplification at scale.
pub fn generate_uuid_v7() -> Uuid {
    Uuid::now_v7()
}

/// Demonstrates inserting and querying exact financial values with `rust_decimal::Decimal`
/// to eliminate floating point rounding inaccuracies (e.g. 0.1 + 0.2 != 0.3 in f64).
pub async fn calculate_exact_order_total(
    pool: &PgPool,
    user_id: i64,
) -> Result<Decimal, sqlx::Error> {
    let total: Option<Decimal> = sqlx::query_scalar(
        "SELECT SUM(amount) FROM orders WHERE user_id = $1"
    )
    .bind(user_id)
    .fetch_one(pool)
    .await?;

    Ok(total.unwrap_or(Decimal::ZERO))
}

/// Demonstrates working with native Postgres `TIMESTAMPTZ` mapped to `DateTime<Utc>`
pub async fn list_orders_in_range(
    pool: &PgPool,
    start: DateTime<Utc>,
    end: DateTime<Utc>,
) -> Result<i64, sqlx::Error> {
    sqlx::query_scalar(
        "SELECT count(*) FROM orders WHERE created_at >= $1 AND created_at < $2"
    )
    .bind(start)
    .bind(end)
    .fetch_one(pool)
    .await
}

/// Demonstrates inserting and querying PostgreSQL custom ENUMs (`user_role`)
pub async fn find_users_by_role(
    pool: &PgPool,
    role: UserRole,
) -> Result<Vec<User>, sqlx::Error> {
    sqlx::query_as::<_, User>(
        "SELECT * FROM users WHERE role = $1 ORDER BY id ASC"
    )
    .bind(role)
    .fetch_all(pool)
    .await
}

/// Demonstrates inserting JSONB documents with `sqlx::types::Json`
pub async fn update_user_profile(
    pool: &PgPool,
    user_id: i64,
    profile: UserProfile,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "UPDATE users SET profile = $1, updated_at = now() WHERE id = $2"
    )
    .bind(Json(profile))
    .bind(user_id)
    .execute(pool)
    .await?;

    Ok(())
}

/// Demonstrates querying native PostgreSQL arrays (`TEXT[]`) and containment (`@>`)
pub async fn find_users_with_tag(
    pool: &PgPool,
    tag: &str,
) -> Result<Vec<User>, sqlx::Error> {
    // Array containment operator `@>` in Postgres
    sqlx::query_as::<_, User>(
        "SELECT * FROM users WHERE tags @> ARRAY[$1]::text[] ORDER BY id ASC"
    )
    .bind(tag)
    .fetch_all(pool)
    .await
}
