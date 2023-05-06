use std::env;
use std::sync::Arc;

use checker_tema_3_sd::app::App;
use checker_tema_3_sd::io::handler::IoAsyncHandler;
use checker_tema_3_sd::io::IoEvent;
use checker_tema_3_sd::start_ui;
use eyre::Result;
use log::{info, LevelFilter};

use crate::legacy::run_tests;

mod legacy;

#[tokio::main]
async fn main() -> Result<()> {
    let (sync_io_tx, mut sync_io_rx) = tokio::sync::mpsc::channel::<IoEvent>(100);

    let args: Vec<String> = env::args().collect();

    if args.len() > 1 && args[1] == "--legacy" {
        info!("Running in legacy mode");
        println!("Running in legacy mode");

        let app = App::new(sync_io_tx.clone());

        run_tests(app);

        return Ok(());
    }

    // We need to share the App between thread
    let app = Arc::new(tokio::sync::Mutex::new(App::new(sync_io_tx.clone())));
    let app_ui = Arc::clone(&app);

    // Configure log
    tui_logger::init_logger(LevelFilter::Info).unwrap();
    tui_logger::set_default_level(log::LevelFilter::Info);

    // Handle IO in a specifc thread
    tokio::spawn(async move {
        let mut handler = IoAsyncHandler::new(app);
        while let Some(io_event) = sync_io_rx.recv().await {
            handler.handle_io_event(io_event).await;
        }
    });

    start_ui(&app_ui).await?;

    Ok(())
}
