use std::{thread, time, str, fs::{File, self}, io::{BufReader, BufWriter, BufRead, Write}};
use log::{info, warn};
use std::process::{Command, Stdio};
// use std::;

use super::App;

const STATUS_RUNNING: &str =  "RUNNING";
// const STATUS_DONE: String =  String::from("DONE");


pub fn run_all<'a>(app: &mut App) {
	// info!("Before {:p}", &app[0].status);

    for test_index in 0..app.test_list.len() {
		thread::scope(|s| {
			s.spawn(|| 	run_test(test_index, app));
		});
    }
}


pub fn run_test<'a>(index: usize, app: &mut App) {
	let current_test = &mut app.test_list[index];
	
    current_test.status.clear();
    current_test.status.push_str(STATUS_RUNNING);

	info!("Running test number {} with status {}", index, current_test.status);

	// let file = File::create(format!("{}input/{}-test.in", app.test_path, index)).unwrap();
	let _out_file = File::create(format!("{}output/{}-test.out", app.test_path, index)).unwrap();
	let in_file = fs::read(format!("{}input/{}-test.in", app.test_path, index)).unwrap();
	let mut binding = Command::new(app.exec_name.clone());
	
	let run = binding
		.stdin(Stdio::piped())
		.stdout(Stdio::piped());

	info!("Executing {:?}", run);

	match run.spawn() {
		Ok(mut child) => {
			current_test.status.clear();
			current_test.status.push_str("10");
			info!("{:?}", child);

			let child_stdin = child.stdin.as_mut().unwrap();

			for line in in_file.lines() {
				if let Ok(mut curr) = line {
					curr.push('\n');
					match child_stdin.write(curr.as_bytes()) {
						Ok(_) => {
							info!("wrote {}", curr);
						},
						Err(error) => {
							warn!("{:?}", error);
						}
					}
				}
			}
			drop(child_stdin);
			
			current_test.log.clear();
			if let Some(ref mut stdout) = child.stdout {
				info!("printing stdout");
				for line in BufReader::new(stdout).lines() {
					let l: String = format!("{}\n", line.unwrap());
					info!("file_contains {}", l);
					current_test.log.push_str(&l);
				}
			}

			if let Ok(out) = child.wait_with_output() {
				info!("exit status {:?}", out.status.code());
				info!("{:?}", out.stdout);
				// current_test.log.push_str(str::from_utf8(out.stdout));
			}

			// current_test.log.push_str(&out..status.to_string());
			// info!("Stdout is {:?}", &spawned.st);
		},
		Err(error) => {
			current_test.status.clear();
			current_test.status.push_str("ERR");

			current_test.log.clear();
			current_test.log.push_str(&error.to_string());
			warn!("{:?}", error);
			// current_test
		},
	}
}