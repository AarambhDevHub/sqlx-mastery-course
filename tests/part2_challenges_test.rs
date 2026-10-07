use axum::body::Body;
use axum::http::{Request, StatusCode};
use dotenvy::dotenv;
use fastbloom::BloomFilter;
use sqlx_mastery_course::part2::{
    capstone_service::{self, AppState},
    challenges::{
        c01_lookup, c02_availability, c03_autocomplete, c04_counts, c05_pagination,
        c06_export, c07_timeseries, c08_top_n, c09_duplicates, c10_migrations, c11_deletion,
        c12_counters, c13_queues,
    },
    ingestion_race,
};
use sqlx_mastery_course::pool::init_pool;
use std::env;
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};
use tower::ServiceExt; // for oneshot
use uuid::Uuid;

async fn get_test_pool() -> sqlx::PgPool {
    dotenv().ok();
    let database_url = env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://postgres:Darshan7069@localhost:5432/mastery_db".to_string());
    init_pool(&database_url)
        .await
        .expect("Failed to connect to PostgreSQL test instance")
}

#[tokio::test]
async fn test_part2_ingestion_race_suite() {
    let pool = get_test_pool().await;

    // Test 1: Single Inserts
    let single = ingestion_race::benchmark_single_inserts(&pool, 5)
        .await
        .expect("single inserts failed");
    assert_eq!(single.total_rows, 5);

    // Test 2: QueryBuilder Batch
    let qb = ingestion_race::benchmark_query_builder_batch(&pool, 20, 10)
        .await
        .expect("querybuilder batch failed");
    assert_eq!(qb.total_rows, 20);

    // Test 3: Array UNNEST Batch
    let unnest = ingestion_race::benchmark_unnest_batch(&pool, 30)
        .await
        .expect("unnest batch failed");
    assert_eq!(unnest.total_rows, 30);

    // Test 4: Parallel Streaming COPY
    let copy = ingestion_race::benchmark_parallel_copy(&pool, 40, 2)
        .await
        .expect("parallel copy failed");
    assert_eq!(copy.total_rows, 40);
}

#[tokio::test]
async fn test_part2_challenges_01_to_05() {
    let pool = get_test_pool().await;

    let target_uname = format!("target_{}", Uuid::now_v7());
    sqlx::query("INSERT INTO lab_users (username, email, status) VALUES ($1, $2, 'active')")
        .bind(&target_uname)
        .bind(format!("{target_uname}@test.com"))
        .execute(&pool)
        .await
        .expect("Seed lab user failed");

    // Challenge 1: Find One Username
    let c01 = c01_lookup::run_challenge_01(&pool, &target_uname)
        .await
        .expect("Challenge 01 failed");
    assert!(c01.btree_index_ms >= 0.0);
    assert!(c01.covering_index_ms >= 0.0);

    // Challenge 2: Signup Availability Check
    let c02 = c02_availability::run_challenge_02(&pool, &target_uname, std::slice::from_ref(&target_uname))
        .await
        .expect("Challenge 02 failed");
    assert!(c02.count_star_ms >= 0.0);
    assert!(c02.exists_query_ms >= 0.0);

    // Challenge 3: Autocomplete & Trigram Fuzzy Search
    let c03 = c03_autocomplete::run_challenge_03(&pool, "target")
        .await
        .expect("Challenge 03 failed");
    assert!(c03.prefix_text_pattern_ms >= 0.0);
    assert!(c03.trgm_gin_fuzzy_ms >= 0.0);

    // Challenge 4: Dashboard Counts
    let c04 = c04_counts::run_challenge_04(&pool)
        .await
        .expect("Challenge 04 failed");
    assert!(c04.actual_count >= 1);
    assert!(c04.trigger_counter_ms >= 0.0);

    // Challenge 5: Deep Pagination
    let c05 = c05_pagination::run_challenge_05(&pool, 0, 10)
        .await
        .expect("Challenge 05 failed");
    assert!(c05.keyset_latency_ms >= 0.0);
    assert!(c05.offset_latency_ms >= 0.0);
}

#[tokio::test]
async fn test_part2_challenges_06_to_10() {
    let pool = get_test_pool().await;

    // Pre-seed lab_users so C06 export benchmark has rows to stream
    for i in 0..10 {
        let uname = format!("export_seed_{}_{}", Uuid::now_v7(), i);
        let _ = sqlx::query("INSERT INTO lab_users (username, email, status) VALUES ($1, $2, 'active') ON CONFLICT DO NOTHING")
            .bind(&uname)
            .bind(format!("{uname}@test.com"))
            .execute(&pool)
            .await;
    }

    // Challenge 6: Massive Dataset Export
    let mut conn = pool.acquire().await.expect("Acquire conn failed");
    let c06 = c06_export::run_challenge_06(&pool, &mut conn, 20)
        .await
        .expect("Challenge 06 failed");
    assert!(c06.streamed_rows > 0);
    assert!(c06.exported_bytes > 0);

    // Challenge 7: Time-Series Events
    let c07 = c07_timeseries::run_challenge_07(
        &pool,
        chrono::Utc::now() - chrono::Duration::days(2),
        chrono::Utc::now() + chrono::Duration::days(1),
    )
    .await
    .expect("Challenge 07 failed");
    assert!(c07.brin_index_size_mb < c07.btree_index_size_mb);

    // Challenge 8: Top-N Per User
    let c08 = c08_top_n::run_challenge_08(&pool)
        .await
        .expect("Challenge 08 failed");
    assert!(c08.lateral_join_ms >= 0.0);

    // Challenge 9: Duplicates Detection & Purge
    let c09 = c09_duplicates::run_challenge_09(&pool)
        .await
        .expect("Challenge 09 failed");
    assert!(c09.tuned_work_mem_ms >= 0.0);

    // Challenge 10: Zero-Downtime Live Migrations
    let c10 = c10_migrations::run_challenge_10(&pool)
        .await
        .expect("Challenge 10 failed");
    assert!(c10.concurrent_index_ms >= 0.0);
}

#[tokio::test]
async fn test_part2_challenges_11_to_13() {
    let pool = get_test_pool().await;

    // Challenge 11: Large Deletion (Partition Drop)
    let c11 = c11_deletion::run_challenge_11(&pool)
        .await
        .expect("Challenge 11 failed");
    assert!(c11.partition_drop_ms >= 0.0);

    // Challenge 12: Hot Counters (Sharded Counters)
    let c12 = c12_counters::run_challenge_12(&pool, 10, 20)
        .await
        .expect("Challenge 12 failed");
    assert!(c12.sharded_counter_rps > 0);

    // Challenge 13: Transactional Job Queues with SKIP LOCKED
    let c13 = c13_queues::run_challenge_13(&pool, 10, 2)
        .await
        .expect("Challenge 13 failed");
    assert_eq!(c13.total_jobs_processed, 10);
}

#[tokio::test]
async fn test_part2_capstone_axum_service() {
    let pool = get_test_pool().await;

    let test_user = format!("axum_{}", Uuid::now_v7());
    sqlx::query("INSERT INTO lab_users (username, email, status) VALUES ($1, $2, 'active')")
        .bind(&test_user)
        .bind(format!("{test_user}@test.com"))
        .execute(&pool)
        .await
        .expect("Insert user for axum test failed");

    // Initialize AppState
    let mut bloom = BloomFilter::with_num_bits(65536).expected_items(100);
    bloom.insert(test_user.as_str());
    let bloom_arc = Arc::new(RwLock::new(bloom));

    let (tx, rx) = mpsc::channel(1000);
    capstone_service::spawn_counter_flush_worker(pool.clone(), rx);

    let state = AppState {
        pool: pool.clone(),
        bloom_filter: bloom_arc,
        view_counter_tx: tx,
    };

    let app = capstone_service::create_app(state);

    // 1. Test GET /users/check for existing user
    let req = Request::builder()
        .uri(format!("/users/check?username={test_user}"))
        .body(Body::empty())
        .unwrap();

    let response = app.clone().oneshot(req).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    // 2. Test GET /users/check for available username (filtered by Bloom filter)
    let req = Request::builder()
        .uri("/users/check?username=totally_unique_and_free")
        .body(Body::empty())
        .unwrap();

    let response = app.clone().oneshot(req).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    // 3. Test GET /users keyset pagination
    let req = Request::builder()
        .uri("/users?cursor=0&limit=5")
        .body(Body::empty())
        .unwrap();

    let response = app.clone().oneshot(req).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    // 4. Test POST /posts/{id}/view
    let req = Request::builder()
        .method("POST")
        .uri("/posts/1/view")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(req).await.unwrap();
    assert_eq!(response.status(), StatusCode::ACCEPTED);
}
