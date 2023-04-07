use std::time::Duration;
use std::{process::Stdio, sync::Arc};

use eyre::Result;
use log::{debug, error, info, warn};
use tokio::fs::{self, File};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::join;
use tokio::process::Command;
use tokio::time::{Instant, timeout};

use super::IoEvent;
use crate::app::{App, Data};

const DB_PATH: &str = "./data.json";

/// In the IO thread, we handle IO event without blocking the UI thread
pub struct IoAsyncHandler {
    app: Arc<tokio::sync::Mutex<App>>,
}

impl IoAsyncHandler {
    pub fn new(app: Arc<tokio::sync::Mutex<App>>) -> Self {
        Self { app }
    }

    /// We could be async here
    pub async fn handle_io_event(&mut self, io_event: IoEvent) {
        let result = match io_event {
            IoEvent::Initialize => self.do_initialize().await,
            IoEvent::Sleep(duration) => self.do_sleep(duration).await,
            IoEvent::RunTest(index) => self.run_test(index).await,
            IoEvent::RunAll(size) => self.run_all(size).await,
            IoEvent::SaveData(data) => self.save_data(data).await,
        };

        if let Err(err) = result {
            error!("Oops, something wrong happen: {:?}", err);
        }

        let mut app = self.app.lock().await;
        app.loaded();
    }

    /// We use dummy implementation here, just wait 1s
    async fn do_initialize(&mut self) -> Result<()> {
        info!("🚀 Initialize the application");
        let mut app = self.app.lock().await;
        // tokio::time::sleep(Duration::from_secs(1)).await;
        app.initialized(); // we could update the app state
        info!("👍 Application initialized");

        Ok(())
    }

    /// Just take a little break
    async fn do_sleep(&mut self, duration: Duration) -> Result<()> {
        info!("😴 Go sleeping for {:?}...", duration);
        tokio::time::sleep(duration).await;
        info!("⏰ Wake up !");
        // Notify the app for having slept
        let mut app = self.app.lock().await;
        app.slept();

        Ok(())
    }

    async fn save_data(&mut self, data: Data) -> Result<()> {
        info!("Saving data");
        tokio::fs::write(DB_PATH, serde_json::to_string_pretty(&data).unwrap()).await?;
        Ok(())
    }

    async fn run_all(&mut self, size: usize) -> Result<()> {
        // let mut app = self.app.lock().await;
        let mut threads = Vec::new();

        for index in 0..size {
            let copy = Arc::clone(&self.app);

            let thread = tokio::spawn(async move {
                debug!("Waiting on mutex");
                let mut app = copy.lock().await;

                app.test_list[index].status.clear();
                app.test_list[index].status.push_str("STARTING");
                app.dispatch(IoEvent::RunTest(index)).await;
            });
            threads.push(thread);
        }
        // let mut awaits = Vec::new();
        for thread in threads {
            // awaits.push(thread.await.unwrap());
            thread.await.unwrap();
        }

        Ok(())
    }

    async fn run_test(&mut self, index: usize) -> Result<()> {
        let mut app = self.app.lock().await;

        let mut out_file = File::create(format!("{}output/{}-test.out", app.test_path, index))
            .await
            .unwrap();

        let in_prom = fs::read(format!("{}input/{}-test.in", app.test_path, index));
        let ref_prom = fs::read(format!("{}ref/{}-test.ref", app.test_path, index));

        let (in_file, ref_file) = join!(in_prom, ref_prom);

        let (in_file, ref_file) = match (in_file, ref_file) {
            (Ok(f1), Ok(f2)) => (f1, f2),
            (Err(a), _) => {
                error!(
                    "Cannot find {}",
                    format!("{}ref/{:02}-test.ref", app.test_path, index)
                );
                return Err(a.into());
            }
            (_, Err(a)) => {
                error!(
                    "Cannot find {}",
                    format!("{}in/{:02}-test.in", app.test_path, index)
                );
                return Err(a.into());
            }
        };

        let mut binding = Command::new(app.exec_name.clone());
        let current_test = &mut app.test_list[index];
        current_test.status.clear();
        current_test.status.push_str("RUNNING");

        info!(
            "Running test number {} with status {}",
            index, current_test.status
        );
        drop(app);

        let run = binding.stdin(Stdio::piped()).stdout(Stdio::piped());

        info!("Executing {:?}", run);
        match run.spawn() {
            Ok(mut child) => {
                info!("{:?}", child);

                let child_stdin = child.stdin.as_mut().unwrap();

                let mut lines = in_file.lines();

                while let Some(mut line) = lines.next_line().await? {
                    line.push('\n');
                    match child_stdin.write(line.as_bytes()).await {
                        Ok(_) => {
                            info!("wrote {}", line);
                        }
                        Err(error) => {
                            warn!("{:?}", error);
                        }
                    }
                }
                drop(child_stdin);

                let start = Instant::now();
                let mut log = String::new();
                let mut res = String::new();

                if let Some(ref mut stdout) = child.stdout {
                    let mut lines = BufReader::new(stdout).lines();

                    loop {
                        if let Ok(res) = timeout(Duration::from_millis(500), lines.next_line()).await {
                            if let Ok(Some(line)) = res {
                                let l: String = format!("{}\n", line);
                                info!("file_contains {}", l);
                                log.push_str(&l);
                            } else {
                                info!("Finished reading from stdout");
                                break;
                            }

                        } else {
                            warn!("timeout");
                            res.push_str("TIMEOUT");

                            let mut app = self.app.lock().await;
                            let current_test = &mut app.test_list[index];
                            current_test.status.clear();
                            current_test.status.push_str(&res);
                            current_test.log.clear();
                            current_test.time_normal = start.elapsed().as_secs_f64();

                            child.kill().await?;

                            return Ok(());
                        }
                    }

                    debug!("got here");
                }

                debug!("time here is {}", start.elapsed().as_secs_f64());

                if let Ok(out) = child.wait_with_output().await {
                    info!("exit status {:?}", out.status.code());
                    if let None = out.status.code() {
                        let runtime = start.elapsed().as_secs_f64();

                        log.push_str(&out.status.to_string().split_off(8));
                        res.push_str("CRASHED");

                        let mut app = self.app.lock().await;
                        let current_test = &mut app.test_list[index];

                        current_test.status.clear();
                        current_test.status.push_str(&res);
                        current_test.log.clear();
                        current_test.log.push_str(&log);
                        current_test.time_normal = runtime;

                        app.unwritten_data = true;
                        return Ok(());
                    }

                    info!("{:?}", out.stdout);
                } else {
                    error!("Oops");
                }

                if &log == std::str::from_utf8(&ref_file).unwrap() {
                    res.push_str("10");
                } else {
                    res.push_str("0");
                }
                let runtime = start.elapsed().as_secs_f64();
                debug!("time={:5}", runtime);

                out_file.write_all(&log.as_bytes()).await.unwrap();

                let mut app = self.app.lock().await;
                let current_test = &mut app.test_list[index];

                current_test.time_normal = runtime;
                current_test.status.clear();
                current_test.status.push_str(&res);
                current_test.log.clear();
                current_test.log.push_str(&log);

                app.unwritten_data = true;
            }
            Err(error) => {
                let mut app = self.app.lock().await;
                let current_test = &mut app.test_list[index];

                current_test.status.clear();
                current_test.status.push_str("ERR");

                current_test.log.clear();
                current_test.log.push_str(&error.to_string());

                app.unwritten_data = true;
                warn!("{:?}", error);
            }
        }

        // debug!("time={:5}", start.elapsed().as_secs_f64());

        Ok(())
    }
}
