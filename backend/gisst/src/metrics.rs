// Metrics counters should be declared in a module here
// so that they can be initialized and accessed at least within gisst
// use the otel_api versions. globals are fine.
// many will be observable counters (instance count, etc)
// and others will be regular counters (times insert file or clone or file list called, along with which user did it??)
// and others could be histograms (clone duration, file listing duration)

// thread_local!(
//     pub(crate) static COUNTS:OnceCell<>
// )

pub async fn start_reporting(pool: sqlx::PgPool) {
    let pool = std::sync::Arc::new(pool);
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
    for table in TABLES {
        let pool = std::sync::Arc::clone(&pool);
        let handle = handle.clone();
        counts
            .u64_observable_counter(table)
            .with_callback(move |obs| {
                let count = handle.block_on(async {
                    let mut conn = pool.acquire().await.unwrap();
                    sqlx::query_scalar!(
                        r#"SELECT reltuples::bigint AS estimate FROM pg_class
                           WHERE oid = ($1::text)::regclass"#,
                        table
                    )
                    .fetch_one(conn.as_mut())
                    .await
                    .unwrap()
                });
                tracing::info!("counter for {table} is {count:?}");
                obs.observe(
                    count
                        .and_then(|num| u64::try_from(num).ok())
                        .unwrap_or_default(),
                    &[],
                );
            })
            .build();
    }
}
