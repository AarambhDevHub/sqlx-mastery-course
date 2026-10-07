use crate::part1::schema::UserRole;
use chrono::{DateTime, Utc};
use sqlx::PgPool;

/// Record structure returned by compile-time checked macro `query!`
#[derive(Debug, PartialEq, Eq)]
pub struct UserMacroSummary {
    pub id: i64,
    pub username: String,
    pub email: String,
    pub role: UserRole,
    pub created_at: DateTime<Utc>,
}

/// Demonstrates compile-time SQL verification using the `query!` macro.
///
/// Under the hood, during `cargo build`, the SQLx procedural macro connects to
/// the database specified by `DATABASE_URL`, sends the SQL to PostgreSQL via the
/// `PREPARE` protocol, and generates Rust types matching the exact column definitions.
///
/// To build without a live database (e.g. in CI/CD), run:
/// ```bash
/// cargo sqlx prepare
/// ```
/// This generates a `.sqlx/query-*.json` cache directory.
pub async fn get_user_summary_macro(
    pool: &PgPool,
    user_id: i64,
) -> Result<Option<UserMacroSummary>, sqlx::Error> {
    let row = sqlx::query!(
        r#"
        SELECT id, username, email, role as "role: UserRole", created_at
        FROM users
        WHERE id = $1
        "#,
        user_id
    )
    .fetch_optional(pool)
    .await?;

    Ok(row.map(|r| UserMacroSummary {
        id: r.id,
        username: r.username,
        email: r.email,
        role: r.role,
        created_at: r.created_at,
    }))
}

/// Demonstrates compile-time checked macro mapping to an anonymous or custom type
pub async fn update_user_status_macro(
    pool: &PgPool,
    user_id: i64,
    new_status: &str,
) -> Result<u64, sqlx::Error> {
    let result = sqlx::query!(
        r#"
        UPDATE users
        SET status = $1, updated_at = now()
        WHERE id = $2
        "#,
        new_status,
        user_id
    )
    .execute(pool)
    .await?;

    Ok(result.rows_affected())
}
