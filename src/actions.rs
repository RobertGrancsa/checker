use std::thread;
use super::App;

pub fn run_all(app: &App) {

    for test in &app.test_list {
		let num = test.id as u32;
        thread::spawn(move || {
            run_test(num);
        });
    }
}


pub fn run_test(number: u32) {

}