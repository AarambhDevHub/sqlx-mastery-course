use crate::part1::schema::{User, UserDecorated};
use futures::Stream;
use sqlx::{PgPool, Row};

/// Demonstrates manual column extraction from a raw `PgRow` using `sqlx::query`
pub async fn get_username_raw(pool: &PgPool, user_id: i64) -> Result<String, sqlx::Error> {
    let row = sqlx::query("SELECT username FROM users WHERE id = $1")
        .bind(user_id)
        .fetch_one(pool)
        .await?;

    let username: String = row.try_get("username")?;
    Ok(username)
}

/// Demonstrates mapping rows directly to a struct via `sqlx::query_as` and `fetch_one`
pub async fn get_user_by_id(pool: &PgPool, user_id: i64) -> Result<User, sqlx::Error> {
    sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = $1")
        .bind(user_id)
        .fetch_one(pool)
        .await
}

/// Demonstrates `fetch_optional` returning `Ok(None)` when no matching record exists
pub async fn find_user_by_email(pool: &PgPool, email: &str) -> Result<Option<User>, sqlx::Error> {
    sqlx::query_as::<_, User>("SELECT * FROM users WHERE email = $1")
        .bind(email)
        .fetch_optional(pool)
        .await
}

/// Demonstrates `fetch_all` collecting all rows into a `Vec<User>`
pub async fn list_users_by_status(pool: &PgPool, status: &str) -> Result<Vec<User>, sqlx::Error> {
    sqlx::query_as::<_, User>("SELECT * FROM users WHERE status = $1 ORDER BY id ASC")
        .bind(status)
        .fetch_all(pool)
        .await
}

/// Demonstrates `query_scalar` for extracting a single scalar primitive column
pub async fn count_users(pool: &PgPool) -> Result<i64, sqlx::Error> {
    sqlx::query_scalar::<_, i64>("SELECT count(*) FROM users")
        .fetch_one(pool)
        .await
}

/// Demonstrates `fetch` returning a streaming cursor over the database socket
pub fn stream_users<'a>(
    pool: &'a PgPool,
) -> impl Stream<Item = Result<User, sqlx::Error>> + 'a {
    sqlx::query_as::<_, User>("SELECT * FROM users ORDER BY id ASC")
        .fetch(pool)
}

/// Demonstrates advanced `FromRow` attributes:
/// - aliasing `username AS user_name` for `#[sqlx(rename = ...)]`
/// - child decomposition via `#[sqlx(flatten)]`
/// - in-memory computed `#[sqlx(skip)]`
pub async fn get_user_decorated(pool: &PgPool, id: i64) -> Result<UserDecorated, sqlx::Error> {
    let mut decorated = sqlx::query_as::<_, UserDecorated>(
        r#"
        SELECT 
            id, 
            username AS user_name, 
            email, 
            country, 
            status, 
            role
        FROM users 
        WHERE id = $1
        "#
    )
    .bind(id)
    .fetch_one(pool)
    .await?;

    // Populate the skipped field in Rust memory
    decorated.display_badge = Some(format!("[{}:{}]", decorated.metadata.country, decorated.metadata.status));
    Ok(decorated)
}
