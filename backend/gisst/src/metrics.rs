// Metrics counters should be declared in a module here
// so that they can be initialized and accessed at least within gisst
// use the otel_api versions. globals are fine.
// many will be observable counters (instance count, etc)
// and others will be regular counters (times insert file or clone or file list called, along with which user did it??)
// and others could be histograms (clone duration, file listing duration)

#[allow(clippy::unused_async)]
pub async fn start_reporting(pool: sqlx::PgPool) {
    use num_traits::cast::ToPrimitive;
    use std::sync::Arc;
    const TABLES: [&str; 13] = [
        "creator",
        "environment",
        "file",
        "instance",
        "instanceobject",
        "instancework",
        "object",
        "replay",
        "save",
        "screenshot",
        "state",
        "users",
        "work",
    ];
    let provider = opentelemetry::global::meter_provider();
    let counts = provider.meter("counts");
    let handle = tokio::runtime::Handle::current();
    let pool = Arc::new(pool);
    for table in TABLES {
        let pool = Arc::clone(&pool);
        let handle = handle.clone();
        counts
            .u64_observable_counter(table)
            .with_callback(move |obs| {
                handle.block_on(async {
                    if let Ok(mut conn) = pool.acquire().await {
                        if let Some(count) = sqlx::query_scalar!(
                            r#"SELECT reltuples::bigint AS estimate 
                               FROM pg_class 
                               WHERE oid = ($1::text)::regclass"#,
                            table
                        )
                        .fetch_one(conn.as_mut())
                        .await
                        .ok()
                        .flatten()
                        .and_then(|num| u64::try_from(num).ok())
                        {
                            obs.observe(count, &[]);
                        }
                    }
                });
            })
            .build();
    }
    let files = provider.meter("files");
    {
        let handle = handle.clone();
        let pool = Arc::clone(&pool);
        let files = files.clone();
        files
            .u64_observable_counter("file_size")
            .with_callback(move |obs| {
                handle.block_on(async {
                    if let Ok(mut conn) = pool.acquire().await {
                        if let Some(size) = sqlx::query_scalar!("SELECT SUM(file_size) FROM file")
                            .fetch_one(conn.as_mut())
                            .await
                            .ok()
                            .flatten()
                        {
                            obs.observe(size.to_u64().unwrap_or(0), &[]);
                        }
                    }
                });
            });
    }
    {
        let handle = handle.clone();
        let pool = Arc::clone(&pool);
        let files = files.clone();
        files
            .u64_observable_counter("file_size_compressed")
            .with_callback(move |obs| {
                handle.block_on(async {
                    if let Ok(mut conn) = pool.acquire().await {
                        if let Some(size) =
                            sqlx::query_scalar!("SELECT SUM(file_compressed_size) FROM file")
                                .fetch_one(conn.as_mut())
                                .await
                                .ok()
                                .flatten()
                        {
                            obs.observe(size.to_u64().unwrap_or(0), &[]);
                        }
                    }
                });
            });
    }
}

#[macro_export]
macro_rules! inc_metric {
    ($conn:expr, $metname:ident, $amt:expr) => {
        inc_metric!($conn, $metname, $amt, {})
    };
    ($conn:expr, $metname:ident, $amt:expr, $($k:ident)+ = $($fields:tt)* ) => {
        inc_metric!($conn, $metname, $amt, { $($k)+ = $($fields)* })
    };
    ($conn:expr, $metname:ident, $amt:expr, { $($attrs:tt)* }) => {
        {
            let conn = $conn.as_mut();
            let amt = $amt;
            let name = stringify!($metname);
            let now = chrono::Utc::now();
            if let Some(metric) = sqlx::query_scalar!(
                r#"INSERT INTO metrics VALUES ($1, NULL, $2, NULL, $3)
                   ON CONFLICT(name) DO UPDATE SET int_value=metrics.int_value+$2, last_observed_time=$3
                   RETURNING int_value"#,
                name,
                amt,
                now
            )
                .fetch_one(conn)
                .await
                .ok()
                .flatten() {
                tracing::info!(monotonic_counter.$metname = metric, $($attrs)*);
            } else {
                tracing::warn!("error posting metric {name} by {amt}");
            }
        }
    };
}
