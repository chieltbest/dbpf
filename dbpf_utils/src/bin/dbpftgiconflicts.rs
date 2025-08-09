use std::{env, path::PathBuf, sync::mpsc::channel};

use dbpf_utils::{application_main, tgi_conflicts::find_conflicts};

#[tokio::main]
async fn main() {
	application_main(|| async {
		env::args_os().skip(1).for_each(|arg| {
			let dir = PathBuf::from(arg);

			let (tx, rx) = channel();

			tokio::task::spawn(find_conflicts(
				Vec::from([dir]),
				tx,
				|_path, _current, _total| {},
			));

			for conflict in rx {
				println!("{:?} --> {:?}", conflict.original, conflict.new);
				for tgi in conflict.tgis {
					println!("{tgi:X?}");
				}
				println!();
			}
		});
	})
	.await;
}
