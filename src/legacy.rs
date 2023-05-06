use std::fs::{self, File};
use std::io::{BufReader, BufRead, Write};
use std::process::{Command, Stdio};
use std::time::Instant;

use checker::app::{App, Test};

pub fn run_tests(app: App) {
    let mut score: usize = 0;

    println!("Running makefile");
    let mut make = Command::new("make");
    let make_run = make.arg("build");
    let res = make_run.output().unwrap();

    if let Some(code) = res.status.code() {
        if code != 0 {
            println!("Makefile error, stopping");
            return;
        }
        println!("{}", String::from_utf8(res.stdout).unwrap());
    }

    for (i, test_list) in app.test_list.iter().enumerate() {
        let app_name = &app.exec_name[i];

        println!();
        println!("==== {app_name} ====");

        for (index, test) in test_list.iter().enumerate() {
            match run_test(&test, index, &app_name, &app.test_path) {
                Ok(amount) => score += amount,
                Err(err) => println!("Error {:?}", err),
            };
        }
    }

    println!("Final score: {score}/100");
}

fn run_test(test: &Test, index: usize, app_name: &String, path: &String) -> Result<usize, ()> {
    let mut run = Command::new("valgrind");
    run.arg(format!(
        "--log-file={}output/{:02}-{}.valgrind",
        path, index, app_name
    ))
    .arg("--leak-check=full")
    .arg("--track-origins=yes")
    .arg("--show-leak-kinds=all")
    .arg("--error-exitcode=69")
    .arg(format!("./{}", app_name));

    print!("Running {app_name} test {index}");

    let mut out_file =
        File::create(format!("{}output/{:02}-{}.out", path, index, app_name)).unwrap();

    let ref_file = if let Ok(file) = fs::read(format!("{}ref/{:02}-{}.ref", path, index, app_name)) {
        file
    } else {
        return Err(());
    };

    let in_file = if let Ok(file) =
        std::fs::File::open(format!("{}input/{:02}-{}.in", path, index, app_name))
    {
        file
    } else {
        return Err(());
    };

    run.stdin(in_file).stdout(Stdio::piped());
    if let Ok(mut child) = run.spawn() {
        let mut log_file = String::new();
        let start = Instant::now();

        if let Some(ref mut stdout) = child.stdout {
            let mut lines = BufReader::new(stdout).lines();
            
            loop {
                let res = lines.next();
                if let Some(Ok(line)) = res {
                    let l: String = format!("{}\n", line);
                    log_file.push_str(&l);
                } else {
                    break;
                }
                
                if start.elapsed().as_millis() > test.timeout as u128 {
                    return Err(());
                }
            }
        } else {
            eprintln!("Broken pipe");
        }

        if let Ok(out) = child.wait_with_output() {
            if out.status.code().is_none() {
                println!("\t\tTime: {:.5}", start.elapsed().as_secs_f64());
                println!("Test {index:02}{}CRASHED: 0/{}", ".".repeat(26), test.test_score);

                return Ok(0);
            } else if let Some(69) = out.status.code() {
                println!("\t\tTime: {:.5}", start.elapsed().as_secs_f64());
                println!("Test {index:02}{}MEMLEAKS: 0/{}", ".".repeat(25), test.test_score);

                return Ok(0);
            }
        } else {
            eprintln!("Cannot wait for child");
        }

        println!("\t\tTime: {:.5}", start.elapsed().as_secs_f64());

        if log_file == std::str::from_utf8(&ref_file).unwrap() {
            println!("Test {index:02}{}PASSED: {}/{}", ".".repeat(27), test.test_score, test.test_score);
        } else {
            println!("Test {index:02}{}FAILED: 0/{}", ".".repeat(27), test.test_score);
        }

        out_file.write_all(log_file.as_bytes()).unwrap();
    }

    Ok(test.test_score)
}
