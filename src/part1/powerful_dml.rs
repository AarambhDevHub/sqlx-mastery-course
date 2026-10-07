use crate::part1::schema::User;
use sqlx::{PgConnection, PgPool};

/// Demonstrates High-Performance Upsert (`INSERT ... ON CONFLICT DO UPDATE`)
/// utilizing the special PostgreSQL pseudo-table `EXCLUDED`.
///
/// Returns the generated or existing primary key `id` using `RETURNING id`.
pub async fn upsert_user(
    pool: &PgPool,
    username: &str,
    email: &str,
    country: &str,
) -> Result<i64, sqlx::Error> {
    let id: i64 = sqlx::query_scalar(
        r#"
        INSERT INTO users (username, email, country)
        VALUES ($1, $2, $3)
        ON CONFLICT (username) DO UPDATE
        SET 
            email = EXCLUDED.email,
            country = EXCLUDED.country,
            updated_at = now()
        RETURNING id
        "#
    )
    .bind(username)
    .bind(email)
    .bind(country)
    .fetch_one(pool)
    .await?;

    Ok(id)
}

/// Demonstrates `RETURNING` returning full modified record in a single round-trip
pub async fn update_user_status_returning(
    pool: &PgPool,
    user_id: i64,
    status: &str,
) -> Result<User, sqlx::Error> {
    sqlx::query_as::<_, User>(
        r#"
        UPDATE users
        SET status = $1, updated_at = now()
        WHERE id = $2
        RETURNING *
        "#
    )
    .bind(status)
    .bind(user_id)
    .fetch_one(pool)
    .await
}

/// Demonstrates Bulk `UNNEST` Ingestion.
///
/// Bypasses PostgreSQL's 65,535 query parameter limit by passing array vectors.
/// Even with 10,000+ rows, this uses only 3 bind parameters (`$1, $2, $3`).
pub async fn bulk_insert_users_unnest(
    pool: &PgPool,
    usernames: &[String],
    emails: &[String],
    countries: &[String],
) -> Result<u64, sqlx::Error> {
    let result = sqlx::query(
        r#"
        INSERT INTO users (username, email, country)
        SELECT * FROM UNNEST($1::text[], $2::text[], $3::text[])
        ON CONFLICT (username) DO NOTHING
        "#
    )
    .bind(usernames)
    .bind(emails)
    .bind(countries)
    .execute(pool)
    .await?;

    Ok(result.rows_affected())
}

/// Demonstrates raw PostgreSQL COPY streaming via `copy_in_raw`.
///
/// Streams CSV formatted byte chunks directly into the database engine,
/// bypassing SQL parsing and parameter binding overhead completely.
pub async fn stream_copy_users_csv(
    conn: &mut PgConnection,
    csv_payload: &[u8],
) -> Result<u64, sqlx::Error> {
    let mut copy = conn
        .copy_in_raw("COPY users (username, email, country) FROM STDIN WITH (FORMAT csv, HEADER true)")
        .await?;

    copy.send(csv_payload).await?;
    let rows_imported = copy.finish().await?;
    Ok(rows_imported)
}
