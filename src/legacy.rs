// use std::io::{BufRead, BufReader, Write};
use std::process::{exit, Stdio};
use std::time::{Duration, Instant};
use tokio::fs::{self, File};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::Command;

use hw_checker::app::{App, Test};
use tokio::time::timeout;

pub async fn run_tests(mut app: App) {
    let mut score: isize = 0;

    println!("Running makefile");
    let mut make = Command::new("make");
    let make_run = make.arg("build");
    let res = match make_run.output().await {
        Ok(out) => out,
        Err(err) => {
            println!("Error {:?}", err);
            exit(1);
        }
    };

    if let Some(code) = res.status.code() {
        if code != 0 {
            println!("Makefile error or makefile not found, stopping");
            println!("{}", String::from_utf8(res.stdout).unwrap());
            println!("{}", String::from_utf8(res.stderr).unwrap());
            return;
        }
        println!(
            "{}",
            match String::from_utf8(res.stdout) {
                Ok(stdout) => stdout,
                Err(err) => {
                    println!("Error {:?}", err);
                    exit(1);
                }
            }
        );
    }

    for (i, test_list) in app.test_list.iter().enumerate() {
        let app_name = &app.exec_name[i];

        println!();
        println!("==== {app_name} ====");

        for (index, test) in test_list.iter().enumerate() {
            match run_test(test, index, app_name, &app.test_path).await {
                Ok(amount) => score += amount as isize,
                Err(err) => println!("Error {:?}", err),
            };
        }
    }

    println!();
    println!("Running checkstyle");

    let mut cs = Command::new(format!("{}/cs/cs.sh", app.test_path));
    cs.arg(".");

    let output = match cs.output().await {
        Ok(res) => res.stdout,
        Err(err) => {
            println!("Error {:?}", err);
            exit(1);
        }
    };

    app.checkstyle.clear();
    app.checkstyle.push_str(match std::str::from_utf8(&output) {
        Ok(res) => res,
        Err(err) => {
            println!("Error {:?}", err);
            exit(1);
        }
    });

    let mut out_file = match File::create(format!("{}checkstyle.txt", app.test_path)).await {
        Ok(res) => res,
        Err(err) => {
            println!("Error {:?}", err);
            exit(1);
        }
    };

    match out_file.write_all(&output).await {
        Ok(_) => {}
        Err(err) => {
            println!("Error {:?}", err);
            exit(1);
        }
    };

    if app.checkstyle.is_empty() {
        println!("No coding style errors found");

        println!("+10 points: Checkstyle");
        score += 10;
    } else {
        println!("{}", app.checkstyle);
        println!(
            "Check {}chekstyle.txt for all the {} errors\n",
            app.test_path,
            app.checkstyle.lines().count()
        );

        let mut errors = vec![0, 0, 0];

        app.checkstyle.lines().for_each(|line| {
            match line {
                _ if line.contains("CHECK") => errors[0] += 1,
                _ if line.contains("WARNING") => errors[1] += 1,
                _ if line.contains("ERROR") => errors[2] += 1,
                _ => (),
            };
        });

        if errors.iter().sum::<isize>() > 0 {
            println!("Found {} errors, reduce them to 0 to get 10 points", errors.iter().sum::<isize>());
        } else {
            println!("+10 points: Checkstyle");
            score += 10;
        }
    }

    println!("Running make clean");
    let mut make = Command::new("make");
    make.arg("clean");
    let mut child = match make.spawn() {
        Ok(chld) => chld,
        Err(err) => {
            println!("Error {:?}", err);
            exit(1);
        }
    };

    match child.wait().await {
        Ok(_) => {}
        Err(err) => {
            println!("Error {:?}", err);
        }
    };

    println!("Total: {score}\n");
}

async fn run_test(
    test: &Test,
    index: usize,
    app_name: &String,
    path: &String,
) -> Result<usize, std::io::Error> {
    let mut run: Command;
    if index < 13 {
        run = Command::new("valgrind");
        run.arg(format!(
            "--log-file={}output/{:02}-{}.valgrind",
            path, index, app_name
        ))
        .arg("--leak-check=full")
        .arg("--track-origins=yes")
        .arg("--show-leak-kinds=all")
        .arg("--error-exitcode=69")
        .arg(format!("./{}", app_name));
    } else {
        run = Command::new(format!("./{}", app_name));
    }

    print!("Running {app_name} test {index}");

    let mut out_file =
        File::create(format!("{}output/{:02}-{}.out", path, index, app_name)).await?;

    let ref_file = fs::read(format!("{}ref/{:02}-{}.ref", path, index, app_name)).await?;

    let in_file = std::fs::File::open(format!("{}input/{:02}-{}.in", path, index, app_name))?;

    run.stdin(in_file).stdout(Stdio::piped());
    if let Ok(mut child) = run.spawn() {
        let mut log_file = String::new();
        let start = Instant::now();

        if let Some(ref mut stdout) = child.stdout {
            let mut lines = BufReader::new(stdout).lines();

            loop {
                let time_left = test.timeout as u128 - start.elapsed().as_millis();
                if let Ok(res) =
                    timeout(Duration::from_millis(time_left as u64), lines.next_line()).await
                {
                    if let Ok(Some(line)) = res {
                        let l: String = format!("{}\n", line);
                        log_file.push_str(&l);
                    } else {
                        break;
                    }

                    if start.elapsed().as_millis() > test.timeout as u128 {
                        println!("\t\tTime: {:.5}", start.elapsed().as_secs_f64());
                        println!(
                            "Test {index:02}{}TIMEOUT: 0/{}",
                            ".".repeat(26),
                            test.test_score
                        );

                        if let Err(err) = child.kill().await {
                            println!("ERROR: Can't kill child: {:?}", err);
                        }

                        return Ok(0);
                    }
                } else {
                    println!("\t\tTime: {:.5}", start.elapsed().as_secs_f64());
                    println!(
                        "Test {index:02}{}TIMEOUT: 0/{}",
                        ".".repeat(26),
                        test.test_score
                    );

                    if let Err(err) = child.kill().await {
                        println!("ERROR: Can't kill child: {:?}", err);
                    }

                    return Ok(0);
                }
            }
        } else {
            eprintln!("Broken pipe");
        }

        if let Ok(out) = child.wait_with_output().await {
            if out.status.code().is_none() {
                println!("\t\tTime: {:.5}", start.elapsed().as_secs_f64());
                println!(
                    "Test {index:02}{}CRASHED: 0/{}",
                    ".".repeat(26),
                    test.test_score
                );

                log_file.push_str("Crashed\n");
                out_file.write_all(log_file.as_bytes()).await?;

                return Ok(0);
            } else if let Some(69) = out.status.code() {
                println!("\t\tTime: {:.5}", start.elapsed().as_secs_f64());
                println!(
                    "Test {index:02}{}MEMLEAKS: 0/{}",
                    ".".repeat(25),
                    test.test_score
                );
                
                log_file.push_str("MEMLEAKS: Check .valgrind file for memory leak info\n");
                out_file.write_all(log_file.as_bytes()).await?;

                return Ok(0);
            }
        } else {
            eprintln!("Cannot wait for child");
        }

        println!("\t\tTime: {:.5}", start.elapsed().as_secs_f64());

        out_file.write_all(log_file.as_bytes()).await?;
        if log_file
            == match std::str::from_utf8(&ref_file) {
                Ok(res) => res,
                Err(err) => {
                    println!("Error {:?}", err);
                    exit(1);
                }
            }
        {
            println!(
                "Test {index:02}{}PASSED: {}/{}",
                ".".repeat(27),
                test.test_score,
                test.test_score
            );

            return Ok(test.test_score);
        } else {
            println!(
                "Test {index:02}{}FAILED: 0/{}",
                ".".repeat(27),
                test.test_score
            );
        }
    }

    Ok(0)
}
