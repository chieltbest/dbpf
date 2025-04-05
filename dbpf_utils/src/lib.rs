use eframe::AppCreator;
use std::io::Cursor;

pub mod editor;
#[cfg(not(target_arch = "wasm32"))]
pub mod tgi_conflicts;

#[cfg(not(target_arch = "wasm32"))]
pub async fn application_main<Fut>(main: impl FnOnce() -> Fut)
where
    Fut: std::future::Future,
{
    use tokio::time::Instant;
    use tracing_subscriber::layer::SubscriberExt;

    tracing::subscriber::set_global_default(
        tracing_subscriber::registry()
            // .with(tracing_tracy::TracyLayer::new())
            .with(tracing_subscriber::fmt::layer().pretty())
            .with(tracing_subscriber::filter::EnvFilter::from_default_env()),
    )
    .expect("set up the subscriber");

    let start = Instant::now();

    main().await;

    let elapsed = start.elapsed();
    println!("(in {:?})", elapsed);
}

#[cfg(not(target_arch = "wasm32"))]
pub fn graphical_application_main(
    icon: &[u8],
    app_name: &str,
    app_creator: AppCreator<'_>,
) -> Result<(), Box<dyn std::error::Error>> {
    use eframe::egui::{IconData, ViewportBuilder};
    use eframe::NativeOptions;
    use tracing_subscriber::layer::SubscriberExt;

    if let Ok(_) = std::env::var("DISPLAY") {
        unsafe {
            std::env::remove_var("WAYLAND_DISPLAY");
        }
    }

    tracing::subscriber::set_global_default(
        tracing_subscriber::registry()
            .with(tracing_subscriber::fmt::layer())
            .with(tracing_subscriber::filter::EnvFilter::from_default_env()),
    )
    .expect("set up the subscriber");

    let image = image::ImageReader::new(Cursor::new(icon))
        .with_guessed_format()?
        .decode()?;
    let buf = Vec::from(image.as_bytes());

    let native_options = NativeOptions {
        viewport: ViewportBuilder::default()
            .with_icon(IconData {
                width: image.width(),
                height: image.height(),
                rgba: buf,
            })
            .with_drag_and_drop(true)
            .with_resizable(true),
        ..Default::default()
    };

    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(async {
            eframe::run_native(app_name,
                               native_options,
                               app_creator)
        })?;

    Ok(())
}
