use std::env;
use std::path::PathBuf;
use std::sync::mpsc::channel;
use dbpf_utils::application_main;
use dbpf_utils::tgi_conflicts::find_conflicts;

#[tokio::main]
async fn main() {
    application_main(|| async {
        env::args_os().skip(1).for_each(|arg| {
            let dir = PathBuf::from(arg);

            let (tx, rx) = channel();

            tokio::task::spawn(find_conflicts(dir, tx));

            for conflict in rx {
                println!("{:?} --> {:?}", conflict.original, conflict.new);
                for tgi in conflict.tgis {
                    println!("{tgi:X?}");
                }
                println!();
            }
        });
    }).await;
}
