use dotenvy::dotenv;
use fastbloom::BloomFilter;
use sqlx_mastery_course::part1::{
    advanced_sql, database_logic, joins_aggregates, powerful_dml, queries_basics,
    queries_dynamic, queries_macros, repository_pattern::UserRepository, security_rls,
    transactions_locking, types_custom,
};
use sqlx_mastery_course::part2::{
    capstone_service::{self, AppState},
    challenges::{
        c01_lookup, c02_availability, c03_autocomplete, c04_counts, c05_pagination,
        c06_export, c07_timeseries, c08_top_n, c09_duplicates, c10_migrations, c11_deletion,
        c12_counters, c13_queues,
    },
    ingestion_race, lab_config, production_scale,
};
use sqlx_mastery_course::pool::{check_health, init_pool};
use std::env;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};
use tracing::info;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    dotenv().ok();
    tracing_subscriber::fmt::init();

    let database_url = env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://postgres:Darshan7069@localhost:5432/mastery_db".to_string());

    info!("==================================================================");
    info!("🚀 Aarambh Dev Hub: SQLx + PostgreSQL Master Course (Part 1 & 2)");
    info!("==================================================================");

    let pool = init_pool(&database_url).await?;
    check_health(&pool).await?;

    let args: Vec<String> = env::args().collect();
    let command = args.get(1).map(|s| s.as_str()).unwrap_or("all");

    match command {
        "server" => {
            info!("Starting Capstone Axum Service on 127.0.0.1:3000...");
            let bloom = Arc::new(RwLock::new(BloomFilter::with_num_bits(65536).expected_items(1000)));
            let (tx, rx) = mpsc::channel(10000);

            capstone_service::spawn_counter_flush_worker(pool.clone(), rx);
            capstone_service::spawn_job_queue_worker(pool.clone());

            let app_state = AppState {
                pool: pool.clone(),
                bloom_filter: bloom,
                view_counter_tx: tx,
            };

            let app = capstone_service::create_app(app_state);
            let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
            let listener = tokio::net::TcpListener::bind(addr).await?;
            info!("Capstone API listening on http://{}", addr);
            axum::serve(listener, app).await?;
        }
        "part1" => {
            run_part1_demos(&pool).await?;
        }
        "part2" => {
            run_part2_demos(&pool).await?;
        }
        _ => {
            info!("Running complete suite: Part 1 and Part 2 demonstrations...");
            run_part1_demos(&pool).await?;
            run_part2_demos(&pool).await?;
            info!("All demonstrations completed successfully!");
        }
    }

    Ok(())
}

async fn run_part1_demos(pool: &sqlx::PgPool) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    info!("--- Running Part 1: Zero to Advanced Demos ---");

    // 1. DML Upsert & Queries
    let u1 = powerful_dml::upsert_user(pool, "alice", "alice@example.com", "IN").await?;
    let _u2 = powerful_dml::upsert_user(pool, "bob", "bob@example.com", "US").await?;
    let _u3 = powerful_dml::upsert_user(pool, "charlie", "charlie@example.com", "UK").await?;
    info!("Seeded users, Alice ID: {u1}");

    // 2. Basics
    let raw_name = queries_basics::get_username_raw(pool, u1).await?;
    info!("Basics - Raw username: {raw_name}");
    let user_obj = queries_basics::get_user_by_id(pool, u1).await?;
    info!("Basics - Struct mapped: {}", user_obj.username);

    // 3. Compile-time macro
    if let Some(summary) = queries_macros::get_user_summary_macro(pool, u1).await? {
        info!("Macro query! verified user: {} ({:?})", summary.username, summary.role);
    }

    // 4. Dynamic SQL
    let filter = queries_dynamic::UserFilter {
        country: Some("IN".to_string()),
        sort_by: Some("username".to_string()),
        sort_desc: false,
        ..Default::default()
    };
    let dynamic_users = queries_dynamic::search_users_dynamic(pool, &filter).await?;
    info!("Dynamic SQL with AssertSqlSafe returned {} users", dynamic_users.len());

    // 5. Types (UUIDv7, Custom Enums)
    let v7 = types_custom::generate_uuid_v7();
    info!("Generated UUIDv7: {v7}");

    // 6. Joins & Aggregates
    let joins = joins_aggregates::get_users_with_optional_orders(pool).await?;
    info!("Joins & Aggregates: {} user-order join records retrieved", joins.len());

    // 7. Advanced SQL (CTEs, Organization Tree)
    let org_tree = advanced_sql::get_organization_tree(pool).await?;
    info!("Advanced SQL: Recursive CTE returned {} org tree nodes", org_tree.len());

    // 8. Database Logic (Views & Refresh)
    let views = database_logic::query_user_order_view(pool).await?;
    info!("Database Logic: Queried view returning {} rows", views.len());

    // 9. Repository Pattern with PgExecutor
    let repo_user = UserRepository::find_by_id(pool, u1).await?;
    info!("Repository pattern fetched: {:?}", repo_user.map(|u| u.username));

    // 10. Transactions & Optimistic Locking
    let mut tx = pool.begin().await?;
    let acc_id: i64 = sqlx::query_scalar(
        "INSERT INTO accounts (user_id, balance, version) VALUES ($1, 500.00, 1) RETURNING id"
    )
    .bind(u1)
    .fetch_one(&mut *tx)
    .await?;
    tx.commit().await?;

    let cas_ok = transactions_locking::update_balance_optimistic(pool, acc_id, rust_decimal::Decimal::new(60000, 2), 1).await?;
    info!("Optimistic CAS update success: {cas_ok}");

    // 11. Row-Level Security
    let tenant_a = uuid::Uuid::new_v4();
    let doc = security_rls::insert_document_for_tenant(pool, tenant_a, "Contract A", "Confidential text").await?;
    info!("RLS - Created document {} for Tenant {}", doc.id, tenant_a);
    let tenant_docs = security_rls::fetch_documents_for_tenant(pool, tenant_a).await?;
    info!("RLS - Isolated tenant queries returned {} documents", tenant_docs.len());

    info!("Part 1 demonstrations completed successfully.");
    Ok(())
}

async fn run_part2_demos(pool: &sqlx::PgPool) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    info!("--- Running Part 2: 100 Crore Rows Stress-Test Challenges ---");

    // Runtime lab settings
    let lab = lab_config::fetch_postgres_runtime_settings(pool).await?;
    info!("Lab settings: shared_buffers={}, work_mem={}", lab.shared_buffers, lab.work_mem);

    // Ingestion Race test
    let single_res = ingestion_race::benchmark_single_inserts(pool, 10).await?;
    let qb_res = ingestion_race::benchmark_query_builder_batch(pool, 50, 25).await?;
    let unnest_res = ingestion_race::benchmark_unnest_batch(pool, 100).await?;
    let copy_res = ingestion_race::benchmark_parallel_copy(pool, 200, 4).await?;
    info!("Ingestion results: Single={} rows/s, QB={} rows/s, UNNEST={} rows/s, COPY={} rows/s",
        single_res.rows_per_sec, qb_res.rows_per_sec, unnest_res.rows_per_sec, copy_res.rows_per_sec);

    // Challenges 1 to 13
    let c01 = c01_lookup::run_challenge_01(pool, "user_copy_0").await?;
    info!("C01 Lookup: B-Tree={:.2}ms, Covering={:.2}ms, BRIN={:.2}ms", c01.btree_index_ms, c01.covering_index_ms, c01.brin_index_ms);

    let c02 = c02_availability::run_challenge_02(pool, "new_username", &["user_copy_0".into()]).await?;
    info!("C02 Availability: count(*)={:.2}ms, EXISTS={:.2}ms, Bloom={}ns", c02.count_star_ms, c02.exists_query_ms, c02.bloom_filter_nanos);

    let c03 = c03_autocomplete::run_challenge_03(pool, "user").await?;
    info!("C03 Autocomplete: text_pattern_ops={:.2}ms, GIN trigram={:.2}ms", c03.prefix_text_pattern_ms, c03.trgm_gin_fuzzy_ms);

    let c04 = c04_counts::run_challenge_04(pool).await?;
    info!("C04 Counts: count(*)={:.2}ms, reltuples={:.2}ms, trigger={:.2}ms", c04.count_star_ms, c04.reltuples_estimate_ms, c04.trigger_counter_ms);

    let c05 = c05_pagination::run_challenge_05(pool, 100, 20).await?;
    info!("C05 Pagination: OFFSET={:.2}ms, Keyset={:.2}ms", c05.offset_latency_ms, c05.keyset_latency_ms);

    let mut conn = pool.acquire().await?;
    let c06 = c06_export::run_challenge_06(pool, &mut conn, 100).await?;
    info!("C06 Export: Streaming={:.2}ms, COPY STDOUT={:.2}ms", c06.streaming_latency_ms, c06.copy_stdout_latency_ms);

    let c07 = c07_timeseries::run_challenge_07(pool, chrono::Utc::now() - chrono::Duration::days(1), chrono::Utc::now()).await?;
    info!("C07 Timeseries: BTree={:.2}ms, BRIN={:.2}ms, Pruned={:.2}ms", c07.btree_query_ms, c07.brin_query_ms, c07.partition_pruned_ms);

    let c08 = c08_top_n::run_challenge_08(pool).await?;
    info!("C08 Top-N: Window={:.2}ms, DISTINCT ON={:.2}ms, LATERAL={:.2}ms", c08.window_function_ms, c08.distinct_on_ms, c08.lateral_join_ms);

    let c09 = c09_duplicates::run_challenge_09(pool).await?;
    info!("C09 Duplicates: Low work_mem={:.2}ms, Tuned={:.2}ms", c09.low_work_mem_ms, c09.tuned_work_mem_ms);

    let c10 = c10_migrations::run_challenge_10(pool).await?;
    info!("C10 Migrations: Concurrent index={:.2}ms, Backfilled={} rows", c10.concurrent_index_ms, c10.backfilled_rows);

    let c11 = c11_deletion::run_challenge_11(pool).await?;
    info!("C11 Deletion: Partition drop={:.2}ms", c11.partition_drop_ms);

    let c12 = c12_counters::run_challenge_12(pool, 1, 100).await?;
    info!("C12 Hot Counters: Sharded RPS={}, In-Memory RPS={}", c12.sharded_counter_rps, c12.in_memory_channel_rps);

    let c13 = c13_queues::run_challenge_13(pool, 50, 4).await?;
    info!("C13 Queues: SKIP LOCKED throughput={} jobs/s", c13.skip_locked_throughput_jobs_sec);

    let pool_advice = production_scale::compute_pool_sizing(16, 1);
    info!("Production Scale pool sizing: {} connections recommended", pool_advice.recommended_pool_size);

    info!("Part 2 demonstrations completed successfully.");
    Ok(())
}
