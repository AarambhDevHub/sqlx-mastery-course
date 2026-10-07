use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::{get, post};
use axum::{Json, Router};
use fastbloom::BloomFilter;
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, Row};
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};
use tracing::info;

/// Shared application state for the Capstone Axum Service
#[derive(Clone)]
pub struct AppState {
    pub pool: PgPool,
    pub bloom_filter: Arc<RwLock<BloomFilter>>,
    pub view_counter_tx: mpsc::Sender<ViewIncrementMsg>,
}

/// Message sent across the channel to buffer view increments
pub struct ViewIncrementMsg {
    pub post_id: i64,
}

/// Query parameters for user search
#[derive(Debug, Deserialize)]
pub struct SearchQuery {
    pub q: String,
}

/// Query parameters for username availability
#[derive(Debug, Deserialize)]
pub struct CheckQuery {
    pub username: String,
}

/// Query parameters for keyset pagination
#[derive(Debug, Deserialize)]
pub struct KeysetQuery {
    pub cursor: Option<i64>,
    pub limit: Option<i64>,
}

/// User item returned by API
#[derive(Debug, Serialize, Deserialize)]
pub struct UserResponseItem {
    pub id: i64,
    pub username: String,
    pub email: String,
}

/// Search users by prefix using the `text_pattern_ops` index
pub async fn search_users(
    State(state): State<AppState>,
    Query(query): Query<SearchQuery>,
) -> Result<Json<Vec<UserResponseItem>>, (StatusCode, String)> {
    let rows = sqlx::query_as::<_, (i64, String, String)>(
        "SELECT id, username, email FROM lab_users WHERE username LIKE $1 ORDER BY username ASC LIMIT 20"
    )
    .bind(format!("{}%", query.q))
    .fetch_all(&state.pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let items = rows
        .into_iter()
        .map(|(id, username, email)| UserResponseItem { id, username, email })
        .collect();

    Ok(Json(items))
}

/// Check username availability combining in-memory Bloom filter + PostgreSQL `EXISTS`
pub async fn check_username(
    State(state): State<AppState>,
    Query(query): Query<CheckQuery>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    // Stage 1: In-Memory Bloom Filter Gate
    {
        let bloom = state.bloom_filter.read().await;
        if !bloom.contains(&query.username.as_str()) {
            // 100% Guaranteed NOT in database -> zero DB round trips!
            return Ok(Json(serde_json::json!({
                "username": query.username,
                "available": true,
                "source": "in_memory_bloom_filter"
            })));
        }
    }

    // Stage 2: Database fallback using fast SELECT EXISTS(SELECT 1)
    let exists: bool = sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM lab_users WHERE username = $1)"
    )
    .bind(&query.username)
    .fetch_one(&state.pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(serde_json::json!({
        "username": query.username,
        "available": !exists,
        "source": "postgres_exists_query"
    })))
}

/// Keyset pagination endpoint: O(1) constant sub-millisecond retrieval
pub async fn list_users_keyset(
    State(state): State<AppState>,
    Query(params): Query<KeysetQuery>,
) -> Result<Json<Vec<UserResponseItem>>, (StatusCode, String)> {
    let cursor = params.cursor.unwrap_or(0);
    let limit = params.limit.unwrap_or(20).clamp(1, 100);

    let rows = sqlx::query_as::<_, (i64, String, String)>(
        "SELECT id, username, email FROM lab_users WHERE id > $1 ORDER BY id ASC LIMIT $2"
    )
    .bind(cursor)
    .bind(limit)
    .fetch_all(&state.pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let items = rows
        .into_iter()
        .map(|(id, username, email)| UserResponseItem { id, username, email })
        .collect();

    Ok(Json(items))
}

/// High-throughput view count buffering via Tokio channel
pub async fn record_post_view(
    State(state): State<AppState>,
    Path(post_id): Path<i64>,
) -> impl IntoResponse {
    let _ = state.view_counter_tx.send(ViewIncrementMsg { post_id }).await;
    StatusCode::ACCEPTED
}

/// Creates the complete Axum Router integrating all 13 challenge winning solutions
pub fn create_app(state: AppState) -> Router {
    Router::new()
        .route("/users/search", get(search_users))
        .route("/users/check", get(check_username))
        .route("/users", get(list_users_keyset))
        .route("/posts/{id}/view", post(record_post_view))
        .with_state(state)
}

/// Background worker flushing buffered post view counts to PostgreSQL every 100ms
pub fn spawn_counter_flush_worker(
    pool: PgPool,
    mut rx: mpsc::Receiver<ViewIncrementMsg>,
) -> tokio::task::JoinHandle<()> {
    tokio::spawn(async move {
        use std::collections::HashMap;
        let mut interval = tokio::time::interval(std::time::Duration::from_millis(100));

        loop {
            interval.tick().await;

            let mut batch: HashMap<i64, i64> = HashMap::new();
            while let Ok(msg) = rx.try_recv() {
                *batch.entry(msg.post_id).or_insert(0) += 1;
            }

            for (post_id, count) in batch {
                let _ = sqlx::query("UPDATE posts SET view_count = view_count + $1 WHERE id = $2")
                    .bind(count)
                    .bind(post_id)
                    .execute(&pool)
                    .await;
            }
        }
    })
}

/// Background worker consuming transactional job queues with `SKIP LOCKED`
pub fn spawn_job_queue_worker(pool: PgPool) -> tokio::task::JoinHandle<()> {
    tokio::spawn(async move {
        loop {
            let res = async {
                let mut tx = pool.begin().await?;
                let job_opt = sqlx::query(
                    r#"
                    SELECT id 
                    FROM jobs 
                    WHERE status = 'pending' 
                    ORDER BY id ASC 
                    LIMIT 1 
                    FOR UPDATE SKIP LOCKED
                    "#
                )
                .fetch_optional(&mut *tx)
                .await?;

                if let Some(row) = job_opt {
                    let job_id: i64 = row.try_get("id")?;
                    sqlx::query("UPDATE jobs SET status = 'completed', updated_at = now() WHERE id = $1")
                        .bind(job_id)
                        .execute(&mut *tx)
                        .await?;
                    tx.commit().await?;
                    Ok::<bool, sqlx::Error>(true)
                } else {
                    tx.rollback().await?;
                    Ok(false)
                }
            }
            .await;

            match res {
                Ok(true) => {} // instantly check next job
                Ok(false) => {
                    tokio::time::sleep(std::time::Duration::from_millis(200)).await;
                }
                Err(e) => {
                    info!("Worker error: {e}");
                    tokio::time::sleep(std::time::Duration::from_millis(500)).await;
                }
            }
        }
    })
}
