use crate::part1::schema::User;
use chrono::{DateTime, Utc};
use sqlx::postgres::Postgres;
use sqlx::{AssertSqlSafe, PgPool, QueryBuilder};

/// Filter parameters for dynamic search
#[derive(Debug, Default)]
pub struct UserFilter {
    pub country: Option<String>,
    pub status: Option<String>,
    pub created_after: Option<DateTime<Utc>>,
    pub sort_by: Option<String>,
    pub sort_desc: bool,
    pub limit: Option<i64>,
}

/// Demonstrates dynamic SQL construction using SQLx 0.9's `QueryBuilder`
/// and safe dynamic identifier handling.
///
/// `QueryBuilder` maintains an internal buffer and automatically handles `$1, $2, ...`
/// parameter index bindings. Dynamic structural fragments (like validated identifiers)
/// are validated against strict whitelists before concatenation.
pub async fn search_users_dynamic(
    pool: &PgPool,
    filter: &UserFilter,
) -> Result<Vec<User>, sqlx::Error> {
    let mut builder: QueryBuilder<Postgres> = QueryBuilder::new("SELECT * FROM users WHERE 1=1");

    if let Some(ref country) = filter.country {
        builder.push(" AND country = ");
        builder.push_bind(country);
    }

    if let Some(ref status) = filter.status {
        builder.push(" AND status = ");
        builder.push_bind(status);
    }

    if let Some(ref created_after) = filter.created_after {
        builder.push(" AND created_at >= ");
        builder.push_bind(*created_after);
    }

    // Whitelist dynamic order-by column to prevent SQL injection
    let sort_column = match filter.sort_by.as_deref() {
        Some("username") => "username",
        Some("created_at") => "created_at",
        Some("email") => "email",
        _ => "id",
    };

    let direction = if filter.sort_desc { "DESC" } else { "ASC" };
    let order_clause = format!(" ORDER BY {} {}", sort_column, direction);

    // Append validated ordering clause
    builder.push(order_clause);

    if let Some(limit) = filter.limit {
        builder.push(" LIMIT ");
        builder.push_bind(limit);
    }

    let query = builder.build_query_as::<User>();
    query.fetch_all(pool).await
}

/// Demonstrates SQLx 0.9's `AssertSqlSafe` marker when executing dynamic SQL strings directly
pub async fn execute_dynamic_raw_query(
    pool: &PgPool,
    table_name: &str,
) -> Result<i64, sqlx::Error> {
    // Whitelist allowed table names
    let safe_table = match table_name {
        "users" => "users",
        "orders" => "orders",
        _ => "users",
    };

    let raw_sql = format!("SELECT count(*) FROM {safe_table}");
    // In SQLx 0.9, dynamic SQL strings must be explicitly wrapped in AssertSqlSafe
    sqlx::query_scalar(AssertSqlSafe(raw_sql))
        .fetch_one(pool)
        .await
}
