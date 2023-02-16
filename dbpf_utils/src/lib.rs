use std::future::Future;
use tokio::time::Instant;
use tracing_subscriber::layer::SubscriberExt;

#[tokio::main]
pub async fn application_main<Fut>(main: impl FnOnce() -> Fut)
    where Fut: Future {
    tracing::subscriber::set_global_default(tracing_subscriber::registry()
        .with(tracing_tracy::TracyLayer::new())
        .with(tracing_subscriber::fmt::layer())
        .with(tracing_subscriber::filter::EnvFilter::from_default_env())
    ).expect("set up the subscriber");

    let start = Instant::now();

    main().await;

    let elapsed = start.elapsed();
    println!("(in {:?})", elapsed);
}
