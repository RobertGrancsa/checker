use std::io::{Error, ErrorKind};
use std::time::Duration;
use std::{process::Stdio, sync::Arc};

use log::{debug, error, info, warn};
use similar::{ChangeTag, TextDiff};
use tokio::fs::{self, File};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::Command;
use tokio::time::{timeout, Instant};

use super::IoEvent;
use crate::app::{get_list_index, App, Data};

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
            IoEvent::RunTest(index, exec) => self.run_test(index, exec).await,
            IoEvent::RunAll(size) => self.run_all(size).await,
            IoEvent::RunFailed(indexes) => self.run_failed(indexes).await,
            IoEvent::SaveData(data) => self.save_data(data).await,
            IoEvent::LoadChecksyle => self.load_cs().await,
            IoEvent::Make => self.run_make().await,
            IoEvent::UpdateRef => self.update_ref().await,
            // IoEvent::SendVMChecker => self.send_vmchecker().await,
            // IoEvent::LoadVMChecker => self.load_vmchecker().await,
        };

        self.update_ref().await.unwrap();

        if let Err(Some(output)) = result {
            error!("Oops, something wrong happened: \n{}", output.to_string());
        }
    }

    /// We use dummy implementation here, just wait 1s
    async fn do_initialize(&mut self) -> Result<(), Option<Error>> {
        let mut app = self.app.lock().await;
        app.initialized(); // we could update the app state
                           // info!("Application initialized");

        self.run_make().await?;

        Ok(())
    }

    /*
     *
     * Depcrecated, will return to these later
     *
    async fn load_vmchecker(&self) -> Result<(), Option<Error>> {
        let client = reqwest::Client::new();
        let cookie;

        let (username, password) = Self::get_credentials().await?;

        match client
            .post("https://vmchecker.cs.pub.ro/services/services.py/login")
            .body(format!("username={}&password={}", username, password))
            .send()
            .await
        {
            Ok(res) => {
                cookie = res.headers().get("set-cookie").unwrap().clone();

                if let Ok(body) = res.text().await {
                    if body.contains("false") {
                        return Err(Some(Error::new(ErrorKind::Other, body)));
                    }
                }
            }
            Err(err) => {
                return Err(Some(Error::new(ErrorKind::Other, err.to_string())));
            }
        };

        match client.get("https://vmchecker.cs.pub.ro/services/services.py/getResults?courseId=SD&assignmentId=3-mk-kNN")
                    .header("cookie", &cookie)
                    .send().await {
            Ok(res) => {
    			let v: Value = serde_json::from_str(&res.text().await.unwrap()).unwrap();

    			let mut app = self.app.lock().await;

    			match v.get(5) {
    				Some(output) => {
    					app.vmchecker_out.clear();
    					app.vmchecker_out.push_str(&output["Execuția testelor (stdout)"].to_string());
    				},
    				None => {
    					app.vmchecker_out.clear();
    					app.vmchecker_out.push_str(&v[2].to_string());
    				},
    			}
    		},
            Err(err) => {
                return Err(Some(Error::new(ErrorKind::Other, err.to_string())));
            },
        };

        Ok(())
    }

    async fn send_vmchecker(&self) -> Result<(), Option<Error>> {
        let client = reqwest::Client::new();
        let cookie;

        let (username, password) = Self::get_credentials().await?;

        match client
            .post("https://vmchecker.cs.pub.ro/services/services.py/login")
            .body(format!("username={}&password={}", username, password))
            .send()
            .await
        {
            Ok(res) => {
                cookie = res.headers().get("set-cookie").unwrap().clone();

                if let Ok(body) = res.text().await {
                    if body.contains("false") {
                        return Err(Some(Error::new(ErrorKind::Other, body)));
                    }
                }
            }
            Err(err) => {
                return Err(Some(Error::new(ErrorKind::Other, err.to_string())));
            }
        };

        let mut make = Command::new("make");
        make.arg("pack");

        let res = make.output().await?;

        if let Some(code) = res.status.code() {
            if code != 0 {
                return Err(Some(Error::new(
                    ErrorKind::Other,
                    std::str::from_utf8(&res.stderr).unwrap(),
                )));
            }
            info!("\n{}", String::from_utf8(res.stdout).unwrap());
        }

        let body = Self::build_request().await;

        match client.post("https://vmchecker.cs.pub.ro/services/services.py/uploadAssignment")
                    .header("cookie", &cookie)
                    .header("Content-Type", "multipart/form-data; boundary=---------------------------27347846016281380843096153774")
                    .body(body)
                    .send().await {
            Ok(res) => {
                info!("{:?}", res.text().await);

                info!("Uploaded file to VMChecker, waiting for results");
            },
            Err(err) => {
                return Err(Some(Error::new(ErrorKind::Other, err.to_string())));
            },
        };

        Ok(())
    }

    async fn get_credentials() -> Result<(String, String), Option<Error>> {
        let mut username = String::new();
        let mut password = String::new();
        let mut buf = String::new();

        match File::open(".env").await {
            Ok(file) => {
                let mut bufread = BufReader::new(file);
                bufread.read_line(&mut buf).await.unwrap();

                let cred: Vec<&str> = buf.split('=').collect();
                username.push_str(cred[1]);

                buf.clear();
                bufread.read_line(&mut buf).await.unwrap();

                let cred: Vec<&str> = buf.split('=').collect();
                password.push_str(cred[1]);
            }
            Err(_) => {
                let mut file = File::create(".env").await.unwrap();
                file.write(String::from("username=\npassword=\n").as_bytes())
                    .await
                    .unwrap();
                return Err(Some(Error::new(
                    ErrorKind::Other,
                    "No env file found, creating... Please input your credentials",
                )));
            }
        }

        Ok((
            String::from(username.strip_suffix('\n').unwrap()),
            String::from(password.strip_suffix('\n').unwrap()),
        ))
    }

    async fn build_request() -> Vec<u8> {
        let mut body: Vec<u8> = Vec::new();

        let mut paths = fs::read_dir("./").await.unwrap();
        let mut zip = String::new();

        while let Ok(Some(path)) = paths.next_entry().await {
            if path.path().display().to_string().contains("Tema3.zip") {
                zip.push_str(&path.path().display().to_string());
            }
        }

        let mut zip_buffer = BufReader::new(File::open(&zip).await.unwrap());

        // Add zip file
        body.append(&mut "-----------------------------27347846016281380843096153774\r\n".into());
        body.append(&mut format!("Content-Disposition: form-data; name=\"archiveFile\"; filename=\"{}\"\r\nContent-Type: application/x-zip-compressed\r\n\r\n", zip).into());

        loop {
            if let Ok(byte) = zip_buffer.read_u8().await {
                body.push(byte);
            } else {
                break;
            }
        }

        body.push(b'\r');
        body.push(b'\n');

        // Add the section
        body.append(&mut "-----------------------------27347846016281380843096153774\r\n".into());
        body.append(&mut "Content-Disposition: form-data; name=\"courseId\"\r\n\r\nSD\r\n".into());

        // Add the task
        body.append(&mut "-----------------------------27347846016281380843096153774\r\n".into());
        body.append(
            &mut "Content-Disposition: form-data; name=\"assignmentId\"\r\n\r\n3-mk-kNN\r\n".into(),
        );
        body.append(&mut "-----------------------------27347846016281380843096153774--\r\n".into());

        body
    }
    */

    async fn update_ref(&self) -> Result<(), Option<Error>> {
        let mut app = self.app.lock().await;

        let index = app.test_list_state.selected().unwrap();
        let (test_index, exec_index) = get_list_index(&app.test_list, index);

        app.current_ref = fs::read_to_string(format!(
            "{}ref/{:02}-{}.ref",
            app.test_path, app.test_list[exec_index][test_index].id, app.exec_name[exec_index]
        ))
        .await?;

        app.diff =
            TextDiff::from_lines(&app.current_ref, &app.test_list[exec_index][test_index].log)
                .iter_all_changes()
                .map(|item| {
                    let sign = match item.tag() {
                        ChangeTag::Delete => "-",
                        ChangeTag::Insert => "+",
                        ChangeTag::Equal => " ",
                    };

                    match item.missing_newline() {
                        true => (sign, format!("{}", item)),
                        false => (sign, format!("{}⏎", item)),
                    }
                })
                .collect();

        Ok(())
    }

    async fn save_data(&mut self, data: Data) -> Result<(), Option<Error>> {
        debug!("Saving data");
        tokio::fs::write(DB_PATH, serde_json::to_string_pretty(&data).unwrap()).await?;
        Ok(())
    }

    async fn load_cs(&mut self) -> Result<(), Option<Error>> {
        info!("Running checkstyle");

        let mut app = self.app.lock().await;

        let mut cs = Command::new(format!("{}/cs/cs.sh", app.test_path));
        cs.arg(".");

        let output = cs.output().await?.stdout;

        app.checkstyle.clear();
        app.checkstyle
            .push_str(std::str::from_utf8(&output).unwrap());

        let mut out_file = File::create(format!("{}checkstyle.txt", app.test_path)).await?;

        out_file.write_all(&output).await?;

        let mut new_error = vec![0, 0, 0];

        app.checkstyle
        .lines().for_each(|line| {
            match line {
                _ if line.contains("CHECK") => new_error[0] += 1,
                _ if line.contains("WARNING") => new_error[1] += 1,
                _ if line.contains("ERROR") => new_error[2] += 1,
                _ => (),
            };
        });

        app.errors = new_error;

        Ok(())
    }

    async fn run_make(&self) -> Result<(), Option<Error>> {
        info!("Running makefile");
        let mut make = Command::new("make");
        let make_run = make.arg("build");
        let res = make_run.output().await?;

        if let Some(code) = res.status.code() {
            if code != 0 {
                return Err(Some(Error::new(
                    ErrorKind::Other,
                    std::str::from_utf8(&res.stderr).unwrap(),
                )));
            }
            info!("\n{}", String::from_utf8(res.stdout).unwrap());
        }

        Ok(())
    }

    async fn run_all(&mut self, size: usize) -> Result<(), Option<Error>> {
        let mut threads = Vec::new();

        self.run_make().await?;

        for index in 0..size {
            let copy = Arc::clone(&self.app);

            let thread = tokio::spawn(async move {
                debug!("Waiting on mutex");
                let mut app = copy.lock().await;

                let (test_index, exec_index) = get_list_index(&app.test_list, index);

                app.test_list[exec_index][test_index].status.clear();
                app.test_list[exec_index][test_index]
                    .status
                    .push_str("STARTING");
                app.dispatch(IoEvent::RunTest(test_index, exec_index)).await;
            });

            threads.push(thread);
        }

        for thread in threads {
            thread.await.unwrap();
        }

        Ok(())
    }

    async fn run_failed(&self, indexes: Vec<(usize, usize)>) -> Result<(), Option<Error>> {
        let mut threads = Vec::new();

        self.run_make().await?;

        for index in indexes {
            let copy = Arc::clone(&self.app);

            let thread = tokio::spawn(async move {
                debug!("Waiting on mutex");
                let mut app = copy.lock().await;

                let (test_index, exec_index) = index;

                app.test_list[exec_index][test_index].status.clear();
                app.test_list[exec_index][test_index]
                    .status
                    .push_str("STARTING");
                app.dispatch(IoEvent::RunTest(test_index, exec_index)).await;
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
    async fn run_test(&self, index: usize, exec: usize) -> Result<(), Option<Error>> {
        let mut app = self.app.lock().await;

        let app_name = String::from(&app.exec_name[exec]);

        let mut out_file = File::create(format!(
            "{}output/{:02}-{}.out",
            app.test_path, index, app_name
        ))
        .await?;

        let valgrind = app.valgrind_enabled;

        let ref_prom = fs::read(format!(
            "{}ref/{:02}-{}.ref",
            app.test_path, index, app_name
        ));

        let ref_file = match ref_prom.await {
            Ok(f1) => f1,
            Err(a) => {
                error!(
                    "Cannot find {}",
                    format!("{}input/{:02}-{}.ref", app.test_path, index, app_name)
                );

                let current_test = &mut app.test_list[exec][index];
                current_test.status.clear();
                current_test.status.push_str("ERROR");
                return Err(Some(a));
            }
        };

        let mut binding: Command;
        if valgrind {
            binding = Command::new("valgrind");
            binding
                .arg(format!(
                    "--log-file={}output/{:02}-{}.valgrind",
                    app.test_path, index, app_name
                ))
                .arg("--leak-check=full")
                .arg("--track-origins=yes")
                .arg("--show-leak-kinds=all")
                .arg("--error-exitcode=69")
                .arg(format!("./{}", app_name));
        } else {
            binding = Command::new(format!("./{}", app_name));
        }

        let current_test = &mut app.test_list[exec][index];
        current_test.status.clear();
        current_test.status.push_str("RUNNING");
        let timelimit = current_test.timeout;

        info!(
            "Running {} with test number {} with status {}",
            app_name, index, current_test.status
        );
        let in_file = std::fs::File::open(format!(
            "{}input/{:02}-{}.in",
            app.test_path, index, app_name
        ))?;
        drop(app);

        let run = binding.stdin(in_file).stdout(Stdio::piped());

        debug!("Executing {:?}", run);
        match run.spawn() {
            Ok(mut child) => {
                debug!("{:?}", child);

                let mut log = String::new();
                let mut res = String::new();
                let start = Instant::now();

                debug!("Finished input, waiting for stdout");

                if let Some(ref mut stdout) = child.stdout {
                    let mut lines = BufReader::new(stdout).lines();

                    loop {
                        if let Ok(res_line) =
                            timeout(Duration::from_millis(timelimit), lines.next_line()).await
                        {
                            if let Ok(Some(line)) = res_line {
                                let l: String = format!("{}\n", line);
                                debug!("file_contains {}", l);
                                log.push_str(&l);
                            } else {
                                debug!("Finished reading from stdout");
                                break;
                            }

                            if start.elapsed().as_millis() > timelimit as u128 {
                                warn!("timeout");
                                res.push_str("TIMEOUT");

                                let mut app = self.app.lock().await;
                                let current_test = &mut app.test_list[exec][index];
                                current_test.status.clear();
                                current_test.status.push_str(&res);
                                current_test.log.clear();

                                if valgrind {
                                    current_test.time_valgrind = start.elapsed().as_secs_f64();
                                } else {
                                    current_test.time_normal = start.elapsed().as_secs_f64();
                                }

                                child.kill().await?;

                                return Ok(());
                            }
                        } else {
                            warn!("timeout");
                            res.push_str("TIMEOUT");

                            let mut app = self.app.lock().await;
                            let current_test = &mut app.test_list[exec][index];
                            current_test.status.clear();
                            current_test.status.push_str(&res);
                            current_test.log.clear();

                            if valgrind {
                                current_test.time_valgrind = start.elapsed().as_secs_f64();
                            } else {
                                current_test.time_normal = start.elapsed().as_secs_f64();
                            }

                            child.kill().await?;

                            return Ok(());
                        }
                    }
                } else {
                    warn!("I think it crashed");
                }

                debug!("time here is {}", start.elapsed().as_secs_f64());

                if let Ok(out) = child.wait_with_output().await {
                    debug!("exit status {:?}", out.status.code());
                    if out.status.code().is_none() {
                        let runtime = start.elapsed().as_secs_f64();

                        log.push_str(&out.status.to_string().split_off(8));
                        res.push_str("CRASHED");

                        let mut app = self.app.lock().await;
                        let current_test = &mut app.test_list[exec][index];

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
                        let current_test = &mut app.test_list[exec][index];

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

                let correct: bool = if log == std::str::from_utf8(&ref_file).unwrap() {
                    true
                } else {
                    res.push('0');
                    false
                };

                let runtime = start.elapsed().as_secs_f64();
                debug!("time={:5}", runtime);

                out_file.write_all(log.as_bytes()).await?;

                let mut app = self.app.lock().await;
                let current_test = &mut app.test_list[exec][index];

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
                let current_test = &mut app.test_list[exec][index];

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
