use std::io::{Error, ErrorKind};
use std::time::Duration;
use std::{process::Stdio, sync::Arc};

// use eyre::{Result, ErrReport};
use log::{debug, error, info, warn};
use tokio::fs::{self, File};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::join;
use tokio::process::Command;
use tokio::time::{timeout, Instant};

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
            IoEvent::RunTest(index) => self.run_test(index).await,
            IoEvent::RunAll(size) => self.run_all(size).await,
            IoEvent::RunFailed(indexes) => self.run_failed(indexes).await,
            IoEvent::SaveData(data) => self.save_data(data).await,
            IoEvent::LoadChecksyle => self.load_cs().await,
        };

        if let Err(Some(output)) = result {
            error!("Oops, something wrong happen: \n{}", output.to_string());
        }

        let mut app = self.app.lock().await;
        app.loaded();
    }

    /// We use dummy implementation here, just wait 1s
    async fn do_initialize(&mut self) -> Result<(), Option<Error>> {
        let mut app = self.app.lock().await;
        app.initialized(); // we could update the app state
        info!("Application initialized");

        Ok(())
    }

    async fn save_data(&mut self, data: Data) -> Result<(), Option<Error>> {
        debug!("Saving data");
        tokio::fs::write(DB_PATH, serde_json::to_string_pretty(&data).unwrap())
            .await
            .unwrap();
        Ok(())
    }

    async fn load_cs(&mut self) -> Result<(), Option<Error>> {
        info!("Running checkstyle");

        let mut app = self.app.lock().await;
        
        let mut cs = Command::new(format!("{}/cs/cs.sh", app.test_path));
        cs.arg(".");

        let output = cs.output().await.unwrap().stdout;

        app.checkstyle.clear();
        app.checkstyle.push_str(&std::str::from_utf8(&output).unwrap());

        let mut out_file = File::create(format!("{}checkstyle.txt", app.test_path))
            .await
            .unwrap();

        out_file.write_all(&output).await.unwrap();

        Ok(())
    }

    async fn run_all(&mut self, size: usize) -> Result<(), Option<Error>> {
        let mut threads = Vec::new();

        info!("Running makefile");
        let mut make = Command::new("make");
        let make_run = make.arg("build");
        let res = make_run.output().await.unwrap();

        if let Some(code) = res.status.code() {
            if code != 0 {
                return Err(Some(Error::new(
                    ErrorKind::Other,
                    std::str::from_utf8(&res.stderr).unwrap(),
                )));
            }
        }

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

        for thread in threads {
            thread.await.unwrap();
        }

        Ok(())
    }

    async fn run_failed(&mut self, indexes: Vec<usize>) -> Result<(), Option<Error>> {
        let mut threads = Vec::new();

        for index in indexes {
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

        for thread in threads {
            thread.await.unwrap();
        }

        Ok(())
    }

    /**
     * Runs a single test, by opening or creating an output file,
     * adding the input of the program to the stdin of the executable and
     * by comparing the ref file with the program's output.
     *
     * Oh god, this is a mess but it is working
     */
    async fn run_test(&mut self, index: usize) -> Result<(), Option<Error>> {
        let mut app = self.app.lock().await;

        let mut out_file = File::create(format!("{}output/{:02}-test.out", app.test_path, index))
            .await
            .unwrap();

        let valgrind = app.valgrind_enabled;

        let in_prom = fs::read(format!("{}input/{:02}-test.in", app.test_path, index));
        let ref_prom = fs::read(format!("{}ref/{:02}-test.ref", app.test_path, index));

        let (in_file, ref_file) = join!(in_prom, ref_prom);

        let (in_file, ref_file) = match (in_file, ref_file) {
            (Ok(f1), Ok(f2)) => (f1, f2),
            (Err(a), _) => {
                error!(
                    "Cannot find {}",
                    format!("{}ref/{:02}-test.ref", app.test_path, index)
                );

                let current_test = &mut app.test_list[index];
                current_test.status.clear();
                current_test.status.push_str("ERROR");
                return Err(Some(a));
            }
            (_, Err(a)) => {
                error!(
                    "Cannot find {}",
                    format!("{}in/{:02}-test.in", app.test_path, index)
                );

                let current_test = &mut app.test_list[index];
                current_test.status.clear();
                current_test.status.push_str("ERROR");
                return Err(Some(a));
            }
        };

        let mut binding: Command;
        if valgrind {
            binding = Command::new("valgrind");
            binding.arg(format!(
                "--log-file={}output/{:02}-test.valgrind",
                app.test_path, index
            ));
            binding.arg("--leak-check=full");
            binding.arg("--track-origins=yes");
            binding.arg("--show-leak-kinds=all");
            binding.arg("--error-exitcode=69");
            binding.arg(app.exec_name.clone());
        } else {
            binding = Command::new(app.exec_name.clone());
        }

        let current_test = &mut app.test_list[index];
        current_test.status.clear();
        current_test.status.push_str("RUNNING");
        let timelimit = current_test.timeout;

        info!(
            "Running test number {} with status {}",
            index, current_test.status
        );
        drop(app);

        let run = binding.stdin(Stdio::piped()).stdout(Stdio::piped());

        debug!("Executing {:?}", run);
        match run.spawn() {
            Ok(mut child) => {
                debug!("{:?}", child);

                let child_stdin = child.stdin.as_mut().unwrap();
                let mut lines = in_file.lines();

                while let Some(mut line) = lines.next_line().await.unwrap() {
                    line.push('\n');
                    match child_stdin.write(line.as_bytes()).await {
                        Ok(_) => {
                            debug!("wrote {}", line);
                        }
                        Err(error) => {
                            warn!("{:?}", error);
                        }
                    }
                }
                drop(child_stdin);

                let mut log = String::new();
                let mut res = String::new();
                let start = Instant::now();

                if let Some(ref mut stdout) = child.stdout {
                    let mut lines = BufReader::new(stdout).lines();

                    loop {
                        if let Ok(res) =
                            timeout(Duration::from_millis(timelimit as u64), lines.next_line())
                                .await
                        {
                            if let Ok(Some(line)) = res {
                                let l: String = format!("{}\n", line);
                                debug!("file_contains {}", l);
                                log.push_str(&l);
                            } else {
                                debug!("Finished reading from stdout");
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

                            if valgrind {
                                current_test.time_valgrind = start.elapsed().as_secs_f64();
                            } else {
                                current_test.time_normal = start.elapsed().as_secs_f64();
                            }

                            child.kill().await.unwrap();

                            return Ok(());
                        }
                    }
                }

                debug!("time here is {}", start.elapsed().as_secs_f64());

                if let Ok(out) = child.wait_with_output().await {
                    debug!("exit status {:?}", out.status.code());
                    if out.status.code().is_none() {
                        let runtime = start.elapsed().as_secs_f64();

                        log.push_str(&out.status.to_string().split_off(8));
                        res.push_str("CRASHED");

                        let mut app = self.app.lock().await;
                        let current_test = &mut app.test_list[index];

                        current_test.status.clear();
                        current_test.status.push_str(&res);
                        current_test.log.clear();
                        current_test.log.push_str(&log);

                        if valgrind {
                            current_test.time_valgrind = runtime;
                        } else {
                            current_test.time_normal = runtime;
                        }

                        app.unwritten_data = true;
                        return Ok(());
                    } else if let Some(69) = out.status.code() {
                        let runtime = start.elapsed().as_secs_f64();

                        log.push_str("Check output folder for valgrind errors");
                        res.push_str("MEMLEAKS");

                        let mut app = self.app.lock().await;
                        let current_test = &mut app.test_list[index];

                        current_test.status.clear();
                        current_test.status.push_str(&res);
                        current_test.log.clear();
                        current_test.log.push_str(&log);

                        if valgrind {
                            current_test.time_valgrind = runtime;
                        } else {
                            current_test.time_normal = runtime;
                        }

                        app.unwritten_data = true;
                        return Ok(());
                    }
                } else {
                    error!("Oops");
                }

                let correct: bool;
                if log == std::str::from_utf8(&ref_file).unwrap() {
                    correct = true;
                } else {
                    res.push('0');
                    correct = false;
                }
                let runtime = start.elapsed().as_secs_f64();
                debug!("time={:5}", runtime);

                out_file.write_all(log.as_bytes()).await.unwrap();

                let mut app = self.app.lock().await;
                let current_test = &mut app.test_list[index];

                if valgrind {
                    current_test.time_valgrind = runtime;
                } else {
                    current_test.time_normal = runtime;
                }

                if correct {
                    res.push_str(&current_test.test_score.to_string());
                }

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
                current_test.status.push_str("ERROR");

                current_test.log.clear();
                current_test.log.push_str(&error.to_string());

                app.unwritten_data = true;
                warn!("{:?}", error);
            }
        }

        Ok(())
    }
}
