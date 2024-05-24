use std::{env, sync::Arc, thread::available_parallelism, time::Duration};

use hw_checker::app::App;
use hw_checker::io::handler::IoAsyncHandler;
use hw_checker::io::IoEvent;
use hw_checker::start_ui;
use eyre::Result;
use log::{info, LevelFilter};
use tokio::time::timeout;

use crate::legacy::run_tests;

mod legacy;

#[tokio::main]
async fn main() -> Result<()> {
    env::set_var("RUST_BACKTRACE", "1");
    let (sync_io_tx, sync_io_rx) = tokio::sync::mpsc::channel::<IoEvent>(100);

    let args: Vec<String> = env::args().collect();

    if args.len() > 1 && args[1] == "--legacy" {
        info!("Running in legacy mode");
        println!("Running in legacy mode");

        let app = App::new(sync_io_tx.clone());

        if timeout(Duration::from_millis(595000), run_tests(app))
            .await
            .is_err()
        {
            println!("\nTests ran for too long, stopping execution");
        }

        return Ok(());
    }

    // We need to share the App between thread
    let app = Arc::new(tokio::sync::Mutex::new(App::new(sync_io_tx.clone())));
    let app_ui = Arc::clone(&app);
    let receive = Arc::new(tokio::sync::Mutex::new(sync_io_rx));

    // Configure log
    tui_logger::init_logger(LevelFilter::Info).unwrap();
    tui_logger::set_default_level(log::LevelFilter::Info);
    // log4rs::init_file("logging_config.yaml", Default::default()).unwrap();

    // Handle IO in a specifc thread

    let threads = available_parallelism().unwrap().get();

    for _ in 0..threads {
        let app_clone = Arc::clone(&app);
        let receive_clone = Arc::clone(&receive);

        tokio::spawn(async move {
            let mut handler = IoAsyncHandler::new(app_clone);
            loop {
                let mut copy = receive_clone.lock().await;
                if let Some(io_event) = copy.recv().await {
                    drop(copy);
                    handler.handle_io_event(io_event).await;
                }
            }
        });
    }

    start_ui(&app_ui).await?;

    Ok(())
}
