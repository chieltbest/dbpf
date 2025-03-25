#[cfg(not(target_arch = "wasm32"))]
pub mod tgi_conflicts;
pub mod editor;

#[cfg(not(target_arch = "wasm32"))]
pub async fn application_main<Fut>(main: impl FnOnce() -> Fut)
    where Fut: std::future::Future {
    use tokio::time::Instant;
    use tracing_subscriber::layer::SubscriberExt;

    tracing::subscriber::set_global_default(tracing_subscriber::registry()
        // .with(tracing_tracy::TracyLayer::new())
        .with(tracing_subscriber::fmt::layer().pretty())
        .with(tracing_subscriber::filter::EnvFilter::from_default_env())
    ).expect("set up the subscriber");

    let start = Instant::now();

    main().await;

    let elapsed = start.elapsed();
    println!("(in {:?})", elapsed);
}
