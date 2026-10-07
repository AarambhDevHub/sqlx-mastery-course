use crate::part1::schema::Document;
use sqlx::PgPool;
use uuid::Uuid;

/// Inserts a multi-tenant document scoped to a specific tenant ID.
///
/// Sets role to `app_user` and `app.tenant_id` locally in the transaction so that PostgreSQL's
/// Row-Level Security policy permits the write and enforces tenant isolation.
pub async fn insert_document_for_tenant(
    pool: &PgPool,
    tenant_id: Uuid,
    title: &str,
    content: &str,
) -> Result<Document, sqlx::Error> {
    let mut tx = pool.begin().await?;

    // Switch to non-superuser app_user role so RLS is strictly enforced
    sqlx::query("SET LOCAL ROLE app_user").execute(&mut *tx).await?;

    // Set transaction-local session parameter for RLS policy
    sqlx::query("SELECT set_config('app.tenant_id', $1, true)")
        .bind(tenant_id.to_string())
        .execute(&mut *tx)
        .await?;

    let doc = sqlx::query_as::<_, Document>(
        r#"
        INSERT INTO documents (tenant_id, title, content)
        VALUES ($1, $2, $3)
        RETURNING *
        "#
    )
    .bind(tenant_id)
    .bind(title)
    .bind(content)
    .fetch_one(&mut *tx)
    .await?;

    tx.commit().await?;
    Ok(doc)
}

/// Fetches documents while enforcing Row-Level Security via `SET LOCAL app.tenant_id = $1`.
///
/// Even though the SQL query is a blanket `SELECT * FROM documents` without a WHERE clause,
/// PostgreSQL's RLS engine enforces isolation and physically returns only the tenant's data.
pub async fn fetch_documents_for_tenant(
    pool: &PgPool,
    tenant_id: Uuid,
) -> Result<Vec<Document>, sqlx::Error> {
    let mut tx = pool.begin().await?;

    // Switch to non-superuser app_user role so RLS is strictly enforced
    sqlx::query("SET LOCAL ROLE app_user").execute(&mut *tx).await?;

    // Set transaction-local parameter: 'true' marks it as is_local (expires on commit/rollback)
    sqlx::query("SELECT set_config('app.tenant_id', $1, true)")
        .bind(tenant_id.to_string())
        .execute(&mut *tx)
        .await?;

    let docs = sqlx::query_as::<_, Document>("SELECT * FROM documents ORDER BY id ASC")
        .fetch_all(&mut *tx)
        .await?;

    tx.commit().await?;
    Ok(docs)
}
