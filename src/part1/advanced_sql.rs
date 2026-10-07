use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use sqlx::PgPool;

/// Result row for recursive CTE employee hierarchy
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct HierarchyNode {
    pub id: i64,
    pub name: String,
    pub manager_id: Option<i64>,
    pub path: String,
    pub depth: i32,
}

/// Result row for window function with running total and lag delta
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct OrderWindowMetric {
    pub order_id: i64,
    pub user_id: i64,
    pub amount: Decimal,
    pub row_num: i64,
    pub running_total: Decimal,
    pub previous_amount: Option<Decimal>,
}

/// Result row for latest order per user
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct LatestUserOrder {
    pub user_id: i64,
    pub order_id: i64,
    pub amount: Decimal,
    pub created_at: DateTime<Utc>,
}

/// Demonstrates CTE (Common Table Expression) to pre-filter and aggregate
pub async fn get_high_value_active_users(
    pool: &PgPool,
    min_total_spent: Decimal,
) -> Result<Vec<(i64, String, Decimal)>, sqlx::Error> {
    let rows = sqlx::query_as::<_, (i64, String, Decimal)>(
        r#"
        WITH active_users AS (
            SELECT id, username FROM users WHERE status = 'active'
        ),
        user_totals AS (
            SELECT user_id, SUM(amount) AS total_spent
            FROM orders
            GROUP BY user_id
        )
        SELECT u.id, u.username, t.total_spent
        FROM active_users u
        JOIN user_totals t ON u.id = t.user_id
        WHERE t.total_spent >= $1
        ORDER BY t.total_spent DESC
        "#
    )
    .bind(min_total_spent)
    .fetch_all(pool)
    .await?;

    Ok(rows)
}

/// Demonstrates Recursive CTE to traverse an organizational tree
pub async fn get_organization_tree(
    pool: &PgPool,
) -> Result<Vec<HierarchyNode>, sqlx::Error> {
    sqlx::query_as::<_, HierarchyNode>(
        r#"
        WITH RECURSIVE org_tree AS (
            -- Base case: Top-level executives with no manager
            SELECT 
                id, 
                name, 
                manager_id, 
                name::text AS path, 
                1 AS depth
            FROM employees
            WHERE manager_id IS NULL

            UNION ALL

            -- Recursive step: Employees reporting to managers in org_tree
            SELECT 
                e.id, 
                e.name, 
                e.manager_id, 
                (ot.path || ' -> ' || e.name)::text AS path, 
                ot.depth + 1 AS depth
            FROM employees e
            JOIN org_tree ot ON e.manager_id = ot.id
        )
        SELECT id, name, manager_id, path, depth
        FROM org_tree
        ORDER BY path ASC
        "#
    )
    .fetch_all(pool)
    .await
}

/// Demonstrates Window Functions: `ROW_NUMBER`, `SUM() OVER`, and `LAG`
pub async fn get_user_order_analytics(
    pool: &PgPool,
    user_id: i64,
) -> Result<Vec<OrderWindowMetric>, sqlx::Error> {
    sqlx::query_as::<_, OrderWindowMetric>(
        r#"
        SELECT 
            id AS order_id,
            user_id,
            amount,
            ROW_NUMBER() OVER (PARTITION BY user_id ORDER BY created_at ASC) AS row_num,
            SUM(amount) OVER (PARTITION BY user_id ORDER BY created_at ASC) AS running_total,
            LAG(amount) OVER (PARTITION BY user_id ORDER BY created_at ASC) AS previous_amount
        FROM orders
        WHERE user_id = $1
        ORDER BY created_at ASC
        "#
    )
    .bind(user_id)
    .fetch_all(pool)
    .await
}

/// Demonstrates `DISTINCT ON` to efficiently extract exactly one latest record per user
pub async fn get_latest_order_per_user_distinct_on(
    pool: &PgPool,
) -> Result<Vec<LatestUserOrder>, sqlx::Error> {
    sqlx::query_as::<_, LatestUserOrder>(
        r#"
        SELECT DISTINCT ON (user_id)
            user_id,
            id AS order_id,
            amount,
            created_at
        FROM orders
        ORDER BY user_id, created_at DESC
        "#
    )
    .fetch_all(pool)
    .await
}

/// Demonstrates `LATERAL` join to fetch top-N recent orders per user
pub async fn get_top_orders_per_user_lateral(
    pool: &PgPool,
    limit_per_user: i64,
) -> Result<Vec<LatestUserOrder>, sqlx::Error> {
    sqlx::query_as::<_, LatestUserOrder>(
        r#"
        SELECT 
            u.id AS user_id,
            latest.id AS order_id,
            latest.amount,
            latest.created_at
        FROM users u
        CROSS JOIN LATERAL (
            SELECT id, amount, created_at
            FROM orders o
            WHERE o.user_id = u.id
            ORDER BY o.created_at DESC
            LIMIT $1
        ) latest
        ORDER BY u.id ASC, latest.created_at DESC
        "#
    )
    .bind(limit_per_user)
    .fetch_all(pool)
    .await
}
