# 🎬 YouTube Script — 100 Crore Rows: 13 Real Challenges with SQLx (Part 2 of 2)

**Channel:** Aarambh Dev Hub  
**Estimated Duration:** about 3 hours (target 2h55m – 3h15m)  
**Voice:** Google Gemini TTS (gemini-3.8-flash-tts) — natural conversational tone  
**Note:** No bullet reads — flows like a real person. No code syntax reading. All code is already written and displayed in VS Code on camera, so this is a senior engineer walking through benchmark results and battle-tested code. Project directory for this video: `crore_lab`. Built for SQLx 0.9 and PostgreSQL 16+.

---

## HOOK AND THE HEADLINE DEMO

### Part 1
You can find one username in one hundred crore rows in zero point four two milliseconds. But the wrong index or a missing query constraint will bring your entire database to a dead halt.

### Part 2
Let us put that scale into perspective. One crore is ten million. One hundred crore is one thousand million. That is one billion rows in a single PostgreSQL table. At that scale, data stops being an abstract idea that fits neatly in RAM. A billion rows is hundreds of gigabytes on disk. If your database engine has to scan that table row by row just to answer a simple query, your server will grind its disks for minutes, pin your CPU cores at one hundred percent, and starve every other connection in your pool.

### Part 3
Look at the terminal on screen right now. We run our lookup benchmark against this one-billion-row dataset using Rust and SQLx. In the first run, without an index, the query takes nearly three minutes. In the second run, using the right indexing strategy, the execution time drops to zero point four two milliseconds. That is a speedup of several hundred thousand times.

### Part 4
This video is Part 2 of our masterclass series, and it is designed to be completely watchable on its own. If you already understand basic SQLx, you can follow every challenge without watching Part 1. Across the next three hours, we tackle thirteen real-world high-scale challenges that every engineering team faces as their product grows. Each challenge follows four beats: the real-world problem, the naive implementation and why it fails, the battle-tested solutions compared side by side with measured numbers, and a single rule to remember. Let us get started.

---

## TEST LAB SETUP, HARDWARE, AND POSTGRES SETTINGS

### Part 1
Before we measure anything, let us establish the rules and inspect our benchmark laboratory. Guessing at performance without documenting your hardware and configuration is pointless. All the benchmark metrics you will see on screen come from saved JSON results produced by our Rust benchmark harness running on dedicated hardware.

### Part 2
Here are the machine specifications: a dedicated bare-metal server with sixteen physical CPU cores, sixty-four gigabytes of high-speed memory, and enterprise NVMe solid-state storage delivering over five thousand megabytes per second of sequential read throughput. If you run these benchmarks on a smaller laptop, your absolute numbers will be slower, but the relative ratios between naive and optimized approaches will remain identical.

### Part 3
Now let us inspect our PostgreSQL configuration in `docker-compose.yml`. Running default Postgres settings against a billion rows is guaranteed failure because Postgres defaults are tuned for tiny machines. We configure `shared_buffers` to sixteen gigabytes, which is twenty-five percent of system RAM. We set `work_mem` to sixty-four megabytes per operation, allowing complex sorting and hashing to happen in memory. We configure `maintenance_work_mem` to four gigabytes so index creation runs at full speed. We set `effective_cache_size` to forty-eight gigabytes to inform the query planner of available operating system page cache, and set `max_parallel_workers_per_gather` to four cores.

### Part 4
Next, look at our benchmark harness in `src/benchmarks/harness.rs`. We do not rely on a single execution. We run multiple warm-up queries to stabilize disk and buffer caches, followed by dozens of timed iterations. We record minimum latency, fiftieth percentile, ninety-fifth percentile, and ninety-ninth percentile response times. Most importantly, we inspect queries using PostgreSQL's `EXPLAIN (ANALYZE, BUFFERS, FORMAT JSON)`. This tells us not just the elapsed time, but exactly how many shared buffer pages were hit in RAM and how many read requests touched physical storage.

---

## GENERATING AND LOADING 100 CRORE ROWS: THE INGESTION RACE

### Part 1
How do you get one hundred crore rows into PostgreSQL without waiting for weeks? Open `src/ingestion/race.rs`. We benchmarked four different ingestion strategies in Rust with SQLx to see how they perform when loading massive datasets.

### Part 2
The first method is the naive loop: executing a single `INSERT` statement per row inside a standard loop. Look at the numbers on screen. Single inserts yield roughly six hundred rows per second. At that rate, loading one billion rows would take nearly nineteen days of continuous writing. The overhead of individual network round trips, statement preparation, and transaction logging per row destroys throughput.

### Part 3
The second method is batching with SQLx's `QueryBuilder`. Instead of one row per insert, we construct multi-row inserts: `INSERT INTO users (username, email, country) VALUES ($1, $2, $3), ($4, $5, $6)...`. But here you hit a hard PostgreSQL wire protocol limit: PostgreSQL allows at most sixty-five thousand five hundred and thirty-five bind parameters in a single statement. Since each row has three columns, our maximum batch size is twenty-one thousand eight hundred and forty-five rows. We batch twenty thousand rows per statement inside a transaction. Throughput jumps to eighty-five thousand rows per second.

### Part 4
The third method is using PostgreSQL's `UNNEST` function. Instead of passing twenty thousand individual values, we pass three arrays: an array of usernames, an array of emails, and an array of countries. The SQL statement unrolls the arrays in a single step. The beauty of `UNNEST` is that the entire batch uses exactly three bind parameters, completely bypassing the sixty-five thousand parameter ceiling. Throughput reaches one hundred and ten thousand rows per second.

### Part 5
The fourth and winning method is the native PostgreSQL `COPY` protocol using SQLx's `copy_in_raw`. Open `src/ingestion/copy_loader.rs`. `COPY` bypasses the entire query parsing and planning pipeline, streaming raw CSV or binary bytes directly into table heap storage. Using `copy_in_raw`, a single thread streams over two hundred and fifty thousand rows per second.

### Part 6
To achieve maximum ingestion speed, we launch eight parallel worker tasks in Tokio, each acquiring an independent connection from our pool and streaming partitioned chunks of data simultaneously. In parallel, our ingestion throughput exceeds one million two hundred thousand rows per second.

### Part 7
Here is the golden rule of bulk ingestion: always drop indexes before loading data, and build indexes after the data is loaded. Updating B-tree indexes row-by-row during ingestion forces random disk I/O and page splits that slow ingestion down by an order of magnitude. Once the table is fully populated, we inspect its footprint using `pg_total_relation_size`: the raw heap occupies approximately seventy-two gigabytes on disk. Finally, we run `ANALYZE users` to populate PostgreSQL's query optimizer statistics. Now we are ready for Challenge One.

---

## CHALLENGE 1: FIND ONE USERNAME (THE INDEXING MASTERCLASS)

### Part 1
Our first challenge is the most common query in web engineering: finding a single user record by their username. In our table of one hundred crore rows, we query `SELECT id, email, created_at FROM users WHERE username = $1`. Open `src/challenges/c01_lookup.rs`.

### Part 2
Let us look at the naive implementation: querying the table without any index on the username column. The execution plan on screen shows a `Seq Scan` on the `users` table. PostgreSQL launches parallel workers and begins reading every eight-kilobyte page from the first block to the last. It inspects one billion rows sequentially. The query takes two minutes and forty-four seconds, reading over nine million pages from disk. In production, a single query like this exhausts your connection pool and starves your entire database.

### Part 3
Now look at Solution One: a standard unique B-tree index, created with `CREATE UNIQUE INDEX idx_users_username ON users (username)`. Look at the new execution plan. The sequential scan is replaced with an `Index Scan`. A B-tree on one billion rows has a depth of only four levels. PostgreSQL traverses four tree nodes, finds the exact row pointer in the heap, and returns the result. Execution time drops from one hundred and sixty-four thousand milliseconds down to zero point four five milliseconds. The query touches only four buffer pages.

### Part 4
Now consider a real-world production bug: case-insensitive login. A user registers as Alice in lowercase, but logs in typing Alice with a capital A. Naive application code updates the query to `WHERE lower(username) = lower($1)`. Look at what happens to our query plan: PostgreSQL reverts to a full sequential scan, taking nearly three minutes again. Why? Because the B-tree index contains raw username strings, not the evaluated output of the `lower()` function. The database cannot use a standard index when a column is wrapped in an arbitrary function call.

### Part 5
There are two ways to solve this. First, an expression index: `CREATE UNIQUE INDEX idx_users_lower_username ON users (lower(username))`. Now PostgreSQL matches the functional expression against the index directly, restoring sub-millisecond lookups. The second approach is PostgreSQL's `citext` extension, which provides a case-insensitive text type that compares case-insensitively by default while utilizing standard indexes cleanly.

### Part 6
Now let us inspect Solution Three: the covering index. Notice that in our query, we selected `id`, `email`, and `created_at`. Even with a B-tree index scan, PostgreSQL must visit the table heap to fetch the email and creation date columns. If we define our index with an `INCLUDE` clause: `CREATE UNIQUE INDEX idx_users_covering ON users (username) INCLUDE (email, created_at)`. PostgreSQL now performs an `Index Only Scan`. All requested data is stored directly within the index leaf pages. PostgreSQL never touches the table heap at all, cutting memory buffer reads in half and shielding query performance from table bloat.

### Part 7
What about hash indexes and partial indexes? PostgreSQL supports `CREATE INDEX USING HASH`. Hash indexes provide O(1) lookups and are slightly faster for exact equality, but they cannot support range scans, cannot enforce uniqueness, and have larger index sizes. A partial index, by contrast, includes a `WHERE` clause: `CREATE INDEX idx_users_active ON users (username) WHERE status = 'active'`. If ninety percent of your queries target active users and inactive users make up the bulk of historical data, a partial index cuts index storage by eighty percent and fits entirely in memory.

### Part 8
What about a BRIN index? A Block Range Index records only the minimum and maximum values for ranges of physical pages on disk. For a billion rows, a B-tree index occupies roughly twenty-four gigabytes, whereas a BRIN index occupies only twenty-eight megabytes. However, BRIN indexes only work if the data is physically sorted on disk. Since usernames are randomly distributed, a BRIN index on username fails and forces large scans. Here is the rule to remember: Use a unique B-tree index with covering columns for point lookups, and use expression indexes whenever queries filter on computed transformations.

---

## CHALLENGE 2: SIGNUP USERNAME AVAILABILITY CHECK

### Part 1
Our second challenge occurs during user registration: millions of prospective users typing usernames into a registration form, triggering real-time availability checks. The query must answer a simple binary question: does this username already exist? Open `src/challenges/c02_availability.rs`.

### Part 2
Look at the naive approach: `SELECT count(*) FROM users WHERE username = $1`. Many developers write this query and check whether the returned count is zero. While an index on username makes this relatively fast, counting rows forces the database to evaluate all matches and construct an aggregate record. Worse, if developers omit the index, counting against one billion rows takes minutes.

### Part 3
The proper SQL query is `SELECT EXISTS(SELECT 1 FROM users WHERE username = $1)`. Look at the execution plan: `EXISTS` issues an index scan and stops execution the microsecond the first matching row is encountered. The latency drops to zero point two eight milliseconds.

### Part 4
When registration traffic surges to tens of thousands of requests per second, even sub-millisecond database queries can saturate connection pools. We benchmark two application-layer solutions. First, an in-memory Redis cache with an affirmative cache-stampede lock. Second, a Bloom filter in Rust. A Bloom filter is a probabilistic data structure that tests set membership with zero false negatives. If the Bloom filter says a username is available, it is guaranteed available, and our application never touches PostgreSQL at all. Here is the rule to remember: Always check existence with SELECT EXISTS, and filter high-frequency availability checks with in-memory Bloom filters before hitting the database.

---

## CHALLENGE 3: AUTOCOMPLETE AND FUZZY SEARCH

### Part 1
Our third challenge is autocomplete: as a user types characters into a search box, the backend must return matching usernames instantly. Open `src/challenges/c03_autocomplete.rs`.

### Part 2
Look at the naive implementation: `SELECT username FROM users WHERE username LIKE '%alice%'`. Because the search pattern contains a leading wildcard, standard B-tree indexes cannot be used. PostgreSQL has no way of jumping into the middle of sorted B-tree strings, so it executes a full sequential scan across one billion rows. Latency climbs to nearly three minutes per keystroke.

### Part 3
If your search requirement is prefix matching—matching names that start with the input, like `alice%`—a standard B-tree index still fails in PostgreSQL if your database collation is not C locale. To enable fast prefix searches, we create an index using the `text_pattern_ops` operator class: `CREATE INDEX idx_users_prefix ON users (username text_pattern_ops)`. With this index, `LIKE 'ali%'` performs an index range scan in zero point six milliseconds.

### Part 4
What if you need full fuzzy substring matching and typo tolerance? We enable PostgreSQL's `pg_trgm` extension and build a Generalized Inverted Index, or GIN index: `CREATE INDEX idx_users_trgm ON users USING GIN (username gin_trgm_ops)`. Trigrams split strings into three-character overlapping substrings. Queries using `%alice%` or similarity distance execute in twelve milliseconds across one billion rows. Here is the rule to remember: Use text_pattern_ops for lightning-fast prefix searches, and use trigram GIN indexes when you require arbitrary substring or fuzzy matching.

---

## CHALLENGE 4: THE DASHBOARD COUNT NIGHTMARE

### Part 1
Our fourth challenge is rendering an administrative dashboard that displays the total number of users in the system. The query seems innocent: `SELECT count(*) FROM users`. Open `src/challenges/c04_dashboard_counts.rs`.

### Part 2
In databases like MySQL with MyISAM, counting rows was an instantaneous metadata lookup. In PostgreSQL, however, Multi-Version Concurrency Control, or MVCC, means that every single row has visibility flags indicating which transaction created or deleted it. There is no global row counter. To compute an exact `count(*)`, PostgreSQL must visit every page in the table or index to verify row visibility. On one hundred crore rows, `SELECT count(*)` takes over two minutes and consumes gigabytes of buffer cache.

### Part 3
If your dashboard only needs an approximate count, we can query PostgreSQL's system catalog: `SELECT reltuples::bigint AS estimate FROM pg_class WHERE relname = 'users'`. Look at the execution time: zero point one two milliseconds. The query reads a single row from the catalog table. Because our table was analyzed, the estimate is accurate to within zero point one percent of the true row count, satisfying ninety-nine percent of business reporting needs.

### Part 4
What if business rules demand an exact, transactionally accurate count? We implement a counter table maintained via database triggers. Whenever a user is inserted or deleted, a trigger increments or decrements a row in `counter_aggregates`. Querying this single-row table takes zero point three milliseconds. Here is the rule to remember: Never run SELECT count(*) on large tables; use pg_class catalog estimates for dashboards, or trigger-maintained summary tables when exact numbers are strictly required.

---

## CHALLENGE 5: DEEP PAGINATION

### Part 1
Our fifth challenge is pagination: allowing users or automated consumers to page through millions of records. Open `src/challenges/c05_deep_pagination.rs`.

### Part 2
Look at naive pagination using `LIMIT` and `OFFSET`: `SELECT id, username FROM users ORDER BY id LIMIT 20 OFFSET 10000000`. To deliver page five hundred thousand, PostgreSQL must scan the index, read ten million rows, discard all ten million rows, and return the final twenty. On our billion-row dataset, deep offset queries take several seconds to execute, and disk I/O scales linearly with page depth.

### Part 3
The solution is keyset pagination, also known as cursor-based pagination. Instead of an offset, the client passes the identifier of the last seen row from the previous page: `SELECT id, username FROM users WHERE id > $last_seen_id ORDER BY id LIMIT 20`. Look at the execution plan: PostgreSQL performs a direct B-tree index seek to the exact target identifier, reads twenty consecutive index entries, and returns immediately.

### Part 4
Look at the benchmark comparison on screen. While `OFFSET` latency explodes from one millisecond on page one to over six seconds on page five hundred thousand, keyset pagination executes in zero point four milliseconds regardless of whether you are on the first page or the fifty-millionth page. Here is the rule to remember: Replace LIMIT OFFSET with keyset pagination using WHERE id > cursor for constant O(1) performance at any depth.

---

## CHALLENGE 6: EXPORTING MASSIVE DATASETS

### Part 1
Our sixth challenge is data export: streaming tens of millions of records out of PostgreSQL for reporting, machine learning pipelines, or cold storage backups. Open `src/challenges/c06_export_streaming.rs`.

### Part 2
Look at the naive implementation in Rust: calling `sqlx::query_as(...).fetch_all(&pool).await`. When you call `fetch_all`, SQLx reads the entire result set over the socket and allocates structs in memory. If you attempt to export fifty million rows, your Rust application will consume tens of gigabytes of RAM within seconds, trigger Linux Out-Of-Memory killer, and crash the process.

### Part 3
The first fix is using asynchronous streams with SQLx's `fetch` method. Instead of `fetch_all`, `query.fetch(&pool)` returns a `Stream` of rows. We consume rows chunk by chunk using Tokio asynchronous iterators, formatting them to CSV and writing them directly to a compressed file stream. Memory consumption drops to a flat thirty megabytes, bounded by our buffer size.

### Part 4
For maximum throughput, we bypass SQL row mapping entirely and use PostgreSQL's native `COPY ... TO STDOUT` protocol. We acquire a raw connection and stream the raw database binary or CSV stream directly into our network socket. The export runs at wire speed, exporting one million rows per second. Here is the rule to remember: Never use fetch_all for unbounded datasets; stream rows with fetch or stream directly with COPY TO STDOUT.

---

## CHALLENGE 7: QUERYING THE LAST 24 HOURS OF EVENTS

### Part 1
Our seventh challenge involves high-volume time-series data: an `events` table recording audit logs, clicks, or transactions. The table contains one hundred crore rows, but ninety-nine percent of queries only care about events from the last twenty-four hours. Open `src/challenges/c07_time_series.rs`.

### Part 2
Look at the naive approach: a standard B-tree index on the `created_at` timestamp column. While the index allows fast lookups for recent rows, maintaining a B-tree across a billion rows consumes over twenty gigabytes of disk and RAM. Every new event inserted must update this massive index tree, causing heavy write amplification.

### Part 3
Our first alternative is a BRIN index: `CREATE INDEX idx_events_brin ON events USING BRIN (created_at)`. Because events are inserted in natural chronological order, the timestamp column is physically correlated with page blocks on disk. The BRIN index records only the minimum and maximum timestamp for each block range. The entire index occupies only twelve megabytes—less than one percent of the size of a B-tree—and delivers query times under two milliseconds.

### Part 4
The ultimate production solution is declarative table partitioning by range: `CREATE TABLE events (id BIGINT, created_at TIMESTAMPTZ, ...) PARTITION BY RANGE (created_at)`. We partition our table into daily or monthly tables. When querying events from the last twenty-four hours, PostgreSQL's query optimizer performs partition pruning: it completely ignores historical partitions, scanning only the current active partition. Query latency drops to zero point three milliseconds, and index maintenance overhead disappears. Here is the rule to remember: For chronological append-only data, use BRIN indexes or range partitioning to slash index footprint and enable instant partition pruning.

---

## CHALLENGE 8: TOP-N AND LATEST RECORD PER USER

### Part 1
Our eighth challenge is extracting the most recent record or top three transactions for thousands of users across our billion-row dataset. Open `src/challenges/c08_top_n_per_user.rs`.

### Part 2
Look at the naive approach using subqueries in a WHERE clause: `SELECT * FROM orders o1 WHERE o1.id = (SELECT o2.id FROM orders o2 WHERE o2.user_id = o1.user_id ORDER BY created_at DESC LIMIT 1)`. Correlated subqueries like this force PostgreSQL into quadratic nested loop evaluations that crawl when processing large batches of users.

### Part 3
The first modern solution is using PostgreSQL's `DISTINCT ON` feature: `SELECT DISTINCT ON (user_id) * FROM orders ORDER BY user_id, created_at DESC`. With a composite index on `(user_id, created_at DESC)`, PostgreSQL scans the index and extracts the single latest order per user in an elegant single-pass scan.

### Part 4
When you need the top three orders rather than just the latest one, we benchmark two strategies: window functions using `ROW_NUMBER() OVER (PARTITION BY user_id ORDER BY created_at DESC)`, and `LATERAL` subqueries. For a filtered set of users, a `LATERAL` join using our composite index performs an index-only seek for each user, outperforming window functions by a factor of four. Here is the rule to remember: Use DISTINCT ON for the single latest record per group, and use LATERAL joins with composite indexes for top-N records per group.

---

## CHALLENGE 9: DETECTING AND CLEANING BILLION-ROW DUPLICATES

### Part 1
Our ninth challenge is dealing with data corruption: an application bug or flawed ingestion script inserted duplicate records, and we need to identify and purge them across one hundred crore rows. Open `src/challenges/c09_duplicates.rs`.

### Part 2
The standard query to find duplicates is `SELECT email, count(*) FROM users GROUP BY email HAVING count(*) > 1`. Look at what happens when this runs with default Postgres settings: because one billion rows cannot fit into standard `work_mem`, PostgreSQL's query engine spills the hash table to disk, creating hundreds of temporary files. Disk throughput drops, and the query takes hours to complete.

### Part 3
To optimize this, we increase `work_mem` for the diagnostic session to two gigabytes: `SET work_mem = '2GB'`. Now PostgreSQL performs an in-memory hash aggregate or parallel external merge sort with minimal disk passes, reducing execution time from three hours to four minutes.

### Part 4
Once duplicates are identified, how do you delete them without locking the table? A single `DELETE` statement targeting millions of duplicate rows will lock table pages, blow up Write-Ahead Logs, and crash replication followers. We write a batched deletion script in Rust: querying duplicate identifiers in chunks of ten thousand using the internal `ctid` or primary key, and executing bounded deletions with small pauses between batches. Here is the rule to remember: Increase work_mem to prevent disk spills during large aggregations, and always purge duplicate rows in small, batched transactions.

---

## CHALLENGE 10: ZERO-DOWNTIME LIVE MIGRATIONS AND INDEXING

### Part 1
Our tenth challenge is schema evolution on a live system: your application is handling thousands of requests per second, and you must add a new index or backfill a column across one hundred crore rows without causing a single millisecond of downtime. Open `src/challenges/c10_live_migrations.rs`.

### Part 2
Look at the fatal mistake: executing `CREATE INDEX idx_users_email ON users (email)`. A standard `CREATE INDEX` acquires a `SHARE` lock on the table. While read queries can continue, all incoming `INSERT`, `UPDATE`, and `DELETE` transactions are blocked. On a billion rows, building the index takes twenty minutes. During those twenty minutes, incoming writes queue up, your connection pool exhausts, and your application goes completely offline.

### Part 3
The proper solution is `CREATE INDEX CONCURRENTLY idx_users_email ON users (email)`. The `CONCURRENTLY` flag builds the index in two distinct phases without acquiring an exclusive lock. Reads and writes continue uninterrupted. Crucially, before executing concurrent index creation, we configure `SET lock_timeout = '2s'`. If acquiring the initial metadata lock takes more than two seconds, the statement aborts immediately rather than queueing behind long-running transactions.

### Part 4
When backfilling data into a new column, never run a single `UPDATE users SET status_code = 1`. That locks millions of rows and bloats table storage. In Rust, we implement a keyset backfill worker: updating twenty thousand rows at a time based on primary key ranges, committing after each batch, and allowing autovacuum to clean up dead rows incrementally. Here is the rule to remember: Always use CREATE INDEX CONCURRENTLY with lock_timeout, and execute column backfills in batched primary key chunks.

---

## CHALLENGE 11: DELETING OLD DATA AT SCALE

### Part 1
Our eleventh challenge is data retention: company policy mandates deleting records older than ninety days. In our events table, that means deleting ten crore rows every month. Open `src/challenges/c11_data_deletion.rs`.

### Part 2
Look at the naive approach: `DELETE FROM events WHERE created_at < now() - INTERVAL '90 days'`. Executing a massive `DELETE` like this is catastrophic. In PostgreSQL, deleting a row does not free disk space; it merely marks the row as a dead tuple. A ten-crore row delete generates hundreds of gigabytes of Write-Ahead Logs, saturates replication bandwidth, and leaves behind massive table bloat that degrades subsequent query scans.

### Part 3
If you are on an unpartitioned table, you must delete in bounded batches: `DELETE FROM events WHERE id IN (SELECT id FROM events WHERE created_at < $cutoff LIMIT 50000)`. In Rust, we run this in a loop, pausing briefly between batches to let autovacuum reclaim pages and keep replication lag near zero.

### Part 4
The true high-scale solution is range partitioning. Because our events table is partitioned by month, deleting ninety-day-old data does not require scanning or deleting rows. We execute `ALTER TABLE events DETACH PARTITION events_2025_01`, followed by `DROP TABLE events_2025_01`. Look at the execution time on screen: zero point zero eight milliseconds. PostgreSQL unlinks the file from the filesystem. There are zero dead tuples, zero WAL overhead, and zero performance impact on live queries. Here is the rule to remember: Never run large bulk DELETE queries; design high-volume tables with range partitions so data expiration is a metadata drop.

---

## CHALLENGE 12: THE HOT COUNTER CONTENTION PROBLEM

### Part 1
Our twelfth challenge is high-concurrency writes to a single record: a viral post gaining ten thousand likes per second, or an inventory counter under a flash sale. Open `src/challenges/c12_hot_counters.rs`.

### Part 2
Look at the naive query: `UPDATE posts SET view_count = view_count + 1 WHERE id = $1`. When thousands of concurrent requests attempt to execute this statement simultaneously, PostgreSQL locks the target row for each transaction. Transactions queue up in memory, CPU context-switching spikes, and connection pools become exhausted waiting on row locks. Throughput collapses to a crawl.

### Part 3
We benchmark three solutions in Rust. First, in-memory buffering: instead of hitting PostgreSQL on every request, our Axum web layer pushes increments into a Tokio asynchronous channel. A background worker drains the channel every one hundred milliseconds and flushes the aggregated count in a single batched update: `UPDATE posts SET view_count = view_count + $total WHERE id = $1`. Database load drops by ninety-nine percent.

### Part 4
Second, sharded counters: we split the post counter into sixteen separate rows in a `counter_shards` table. When an increment arrives, we randomly select a shard using a fast hash function and increment that specific row. Row contention is slashed by a factor of sixteen. To read the total count, we simply sum the shards: `SELECT sum(count) FROM counter_shards WHERE post_id = $1`. Here is the rule to remember: Never hammer a single row with concurrent updates; buffer increments in memory or shard the counter across multiple rows.

---

## CHALLENGE 13: HIGH-THROUGHPUT TRANSACTIONAL JOB QUEUES

### Part 1
Our thirteenth challenge is building a transactional job queue: background worker processes polling PostgreSQL to process incoming asynchronous jobs. Open `src/challenges/c13_job_queues.rs`.

### Part 2
Look at the naive implementation: multiple worker tasks running `SELECT id FROM jobs WHERE status = 'pending' ORDER BY created_at LIMIT 1`, followed by `UPDATE jobs SET status = 'processing' WHERE id = $1`. Under concurrent load, multiple workers read the exact same job ID. When they attempt to update it, one worker succeeds while the others fail or block. If workers acquire locks in conflicting orders, PostgreSQL aborts them with deadlock errors.

### Part 3
The proper solution is `SELECT id, payload FROM jobs WHERE status = 'pending' ORDER BY created_at LIMIT 1 FOR UPDATE SKIP LOCKED`. Look at how `SKIP LOCKED` transforms concurrency. When Worker One locks a job row, Worker Two does not wait or block; PostgreSQL's lock manager instantly skips the locked row and hands Worker Two the very next available job.

### Part 4
Look at the benchmark results on screen. With thirty-two concurrent worker threads consuming jobs using `SKIP LOCKED`, throughput scales linearly with zero lock contention and zero deadlocks. You get a fully transactional, durable job queue inside PostgreSQL without needing Redis or RabbitMQ. Here is the rule to remember: Build high-concurrency database queues with SELECT ... FOR UPDATE SKIP LOCKED for lock-free parallel worker processing.

---

## PRODUCTION AT SCALE: POOL SIZING, DIAGNOSTICS, AND REPLICAS

### Part 1
Now that we have conquered our thirteen challenges, let us examine what it takes to operate a billion-row PostgreSQL system in production. Open `src/production/diagnostics.rs`.

### Part 2
Let us talk about connection pool sizing. A widespread mistake is assuming that more connections equal more performance. Developers configure pool sizes of two hundred or five hundred connections. On a machine with sixteen CPU cores, having two hundred active connections forces PostgreSQL into relentless context switching, cache thrashing, and lock contention. In our benchmark, a pool size of twenty connections achieves higher throughput and lower latency than a pool of two hundred connections. Formula to remember: pool size equals CPU cores multiplied by two, plus effective spindle count.

### Part 3
Next, query diagnostics. We enable `pg_stat_statements`. This extension records normalized query fingerprints, total execution time, call counts, and buffer hits. By querying `pg_stat_statements`, we instantly identify the top three queries consuming the most cumulative CPU time across our entire system. We also enable `auto_explain` configured to automatically log execution plans for any query exceeding one hundred milliseconds.

### Part 4
Finally, autovacuum and read replicas. Autovacuum must be tuned aggressively for high-volume tables. We lower `autovacuum_vacuum_scale_factor` to zero point zero five so vacuum runs after five percent of rows are modified, preventing catastrophic table bloat. For read scaling, we configure read/write splitting in SQLx: routing writes to our primary connection pool and analytical read queries to read replica pools.

---

## CAPSTONE ARCHITECTURE AND MASTER SUMMARY

### Part 1
Let us bring everything we have learned into our Capstone Project in `src/capstone/service.rs`. We built a production Axum web microservice in Rust that combines all the winning techniques from our thirteen challenges into a single, cohesive architecture.

### Part 2
Look at how cleanly the service is structured: warm connection pools managed with `PgPoolOptions`, sub-millisecond user lookups using covering indexes, keyset pagination endpoints delivering constant O(1) page access, a background job worker processing queue tasks via `SKIP LOCKED`, in-memory Bloom filter gates for registration availability, and structured error mapping translating Postgres error codes into clean HTTP responses.

### Part 3
Look at the Master Results Table on screen. Across all thirteen challenges, the contrast between naive implementations and optimized strategies is staggering:
- Point lookups drop from two minutes forty-four seconds to zero point four two milliseconds.
- Autocomplete drops from minutes to twelve milliseconds.
- Dashboard counts drop from two minutes to zero point one two milliseconds.
- Deep pagination stays at zero point four milliseconds forever.
- Data export streams at one million rows per second without memory growth.
- Old data deletion drops from minutes of WAL thrashing to an instant zero point zero eight millisecond partition drop.
- Job queues scale to thousands of jobs per second with zero deadlocks.

### Part 4
Database performance is not magic; it is the natural consequence of understanding how your storage engine reads, writes, and locks data on disk. 

If this deep dive gave you the confidence to run massive workloads on PostgreSQL and Rust, subscribe to Aarambh Dev Hub, share this course with your engineering team, and let me know in the comments which challenge you found most surprising. Thank you for watching, and keep building high-performance systems.
