// SPDX-FileCopyrightText: 2023-2025 Chiel Douwes
//
// SPDX-License-Identifier: GPL-3.0-or-later

use std::{future::Future, io::Cursor};

use eframe::AppCreator;
use tracing_panic::panic_hook;

pub mod editor;
#[cfg(not(target_arch = "wasm32"))]
pub mod tgi_conflicts;

#[cfg(not(target_arch = "wasm32"))]
pub async fn application_main<Fut>(main: impl FnOnce() -> Fut)
where
	Fut: Future,
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
	use eframe::{
		egui::{IconData, ViewportBuilder},
		NativeOptions,
	};
	use tracing_subscriber::layer::SubscriberExt;

	unsafe {
		std::env::set_var("RUST_BACKTRACE", "full");
	}

	if std::env::var("DISPLAY").is_ok() {
		unsafe {
			std::env::remove_var("WAYLAND_DISPLAY");
		}
	}

	let log_dir = eframe::storage_dir(app_name).unwrap().join("log");
	std::fs::create_dir_all(log_dir.clone())?;
	let appender = tracing_appender::rolling::daily(log_dir, "rolling.log");
	let (non_blocking_appender, _guard) = tracing_appender::non_blocking(appender);

	tracing::subscriber::set_global_default(
		tracing_subscriber::registry()
			.with(tracing_subscriber::fmt::layer().with_writer(non_blocking_appender))
			.with(tracing_subscriber::fmt::layer().compact())
			.with(tracing_subscriber::filter::EnvFilter::from_default_env()),
	)
	.expect("set up the subscriber");

	std::panic::set_hook(Box::new(panic_hook));

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
		.block_on(async { eframe::run_native(app_name, native_options, app_creator) })?;

	Ok(())
}

#[cfg(not(target_arch = "wasm32"))]
pub fn async_execute<F: Future<Output = ()> + Send + 'static>(f: F) {
	// this is stupid... use any executor of your choice instead
	std::thread::spawn(move || futures::executor::block_on(f));
}
#[cfg(target_arch = "wasm32")]
pub fn async_execute<F: Future<Output = ()> + 'static>(f: F) {
	use wasm_bindgen_futures::wasm_bindgen::JsValue;
	let _ = wasm_bindgen_futures::future_to_promise(async {
		f.await;
		Ok::<JsValue, JsValue>(JsValue::undefined())
	});
}
