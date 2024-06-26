use std::fs;

use log::{debug, error, warn};
use serde::{Deserialize, Serialize};
use similar::{ChangeTag, TextDiff};
use ratatui::widgets::ListState;

use self::actions::Actions;
use self::state::AppState;
use crate::app::actions::Action;
use crate::inputs::key::Key;
use crate::io::IoEvent;

pub mod actions;
pub mod state;
pub mod ui;

const DB_PATH: &str = "./data.json";
const CHECKSTYLE_SCORE: isize = 10;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Test {
    pub id: usize,
    pub name: String,
    pub status: String,
    pub log: String,
    pub time_normal: f64,
    pub time_valgrind: f64,
    pub timeout: u64,
    pub test_score: usize,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Data {
    tests: Vec<Vec<Test>>,
    test_path: String,
    exec_name: Vec<String>,
    valgrind_enabled: bool,
}

#[derive(Debug, PartialEq, Eq)]
pub enum AppReturn {
    Exit,
    Continue,
}

/// The main application, containing the state
pub struct App {
    /// We could dispatch an IO event
    io_tx: tokio::sync::mpsc::Sender<IoEvent>,
    /// Contextual actions
    actions: Actions,
    /// State
    is_loading: bool,
    pub unwritten_data: bool,
    state: AppState,
    test_num: usize,
    pub selected_tab: usize,
    pub test_list: Vec<Vec<Test>>,
    pub test_list_state: ListState,
    windows_list_state: ListState,
    pub log_list_state: ListState,

    pub valgrind_enabled: bool,
    pub test_path: String,
    pub exec_name: Vec<String>,

    pub current_ref: String,
    pub checkstyle: String,
    pub vmchecker_out: String,
    pub diff: Vec<(&'static str, String)>,
    pub errors: Vec<i32>
}

impl App {
    pub fn new(io_tx: tokio::sync::mpsc::Sender<IoEvent>) -> Self {
        let actions = vec![Action::Quit].into();
        let is_loading = false;
        let state = AppState::default();

        let db_content = fs::read_to_string(DB_PATH).unwrap();
        let json: Data = serde_json::from_str::<Data>(&db_content).unwrap();
        let test_list = json.tests;
        let test_path = json.test_path;
        let exec_name = json.exec_name;
        let mut test_list_state = ListState::default();
        test_list_state.select(Some(0));
        let mut windows_list_state = ListState::default();
        windows_list_state.select(Some(0));
        let mut log_list_state = ListState::default();
        log_list_state.select(None);
        let valgrind_enabled = json.valgrind_enabled;
        let selected_tab = 0usize;
        let unwritten_data = false;
        let test_num = test_list.iter().map(|list| list.len()).sum();
        let mut errors = vec![0, 0, 0];

        let current_ref = fs::read_to_string(format!(
            "{}ref/{:02}-{}.ref",
            test_path, test_list[0][0].id, exec_name[0]
        ))
        .unwrap();

        let checkstyle = fs::read_to_string(format!("{}checkstyle.txt", test_path)).unwrap();
        let vmchecker_out = String::new();

        checkstyle
        .lines().for_each(|line| {
            match line {
                _ if line.contains("CHECK") => errors[0] += 1,
                _ if line.contains("WARNING") => errors[1] += 1,
                _ if line.contains("ERROR") => errors[2] += 1,
                _ => (),
            };
        });

        let diff = TextDiff::from_lines(&current_ref, &test_list[0][0].log)
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

        Self {
            io_tx,
            actions,
            is_loading,
            unwritten_data,
            state,
            test_num,
            selected_tab,
            test_list,
            test_list_state,
            windows_list_state,
            log_list_state,
            valgrind_enabled,
            test_path,
            exec_name,
            current_ref,
            checkstyle,
            vmchecker_out,
            diff,
            errors,
        }
    }

    /// Handle a user action
    pub async fn do_action(&mut self, key: Key) -> AppReturn {
        if let Some(action) = self.actions.find(key) {
            debug!("Run action [{:?}]", action);
            match action {
                Action::Quit => AppReturn::Exit,
                Action::Run => {
                    self.dispatch(IoEvent::RunAll(self.test_num)).await;
                    AppReturn::Continue
                }
                Action::RunTaskOne => {
                    let mut task_one = Vec::new();
                    for test in self.test_list[0].iter() {
                        task_one.push((test.id, 0));
                    }

                    self.dispatch(IoEvent::RunFailed(task_one)).await;
                    AppReturn::Continue
                }
                Action::RunTaskTwo => {
                    let mut task_two = Vec::new();
                    for test in self.test_list[1].iter() {
                        task_two.push((test.id, 1));
                    }

                    self.dispatch(IoEvent::RunFailed(task_two)).await;
                    AppReturn::Continue
                }
                Action::RunTaskThree => {
                    let mut task_three = Vec::new();
                    for test in self.test_list[2].iter() {
                        task_three.push((test.id, 2));
                    }

                    self.dispatch(IoEvent::RunFailed(task_three)).await;
                    AppReturn::Continue
                }
                Action::RunFailed => {
                    let mut failed = Vec::new();
                    for (index, execs) in self.test_list.iter().enumerate() {
                        for test in execs {
                            // TODO make this more eficient
                            if test.status == "0"
                                || test.status == "TIMEOUT"
                                || test.status == "CRASHED"
                                || test.status == "MEMLEAKS"
                                || test.status == "ERROR"
                            {
                                failed.push((test.id, index));
                            }
                        }
                    }

                    self.dispatch(IoEvent::RunFailed(failed)).await;
                    AppReturn::Continue
                }
                Action::RunCurrent => {
                    self.dispatch(IoEvent::Make).await;
                    if let Some(index) = self.test_list_state.selected() {
                        let (test_index, exec_index) = get_list_index(&self.test_list, index);

                        self.test_list[exec_index][test_index].status.clear();
                        self.test_list[exec_index][test_index]
                            .status
                            .push_str("RUNNING");

                        self.dispatch(IoEvent::RunTest(test_index, exec_index))
                            .await;
                    } else {
                        warn!("No test selected");
                    }
                    AppReturn::Continue
                }
                Action::RightList => {
                    if let Some(index) = self.windows_list_state.selected() {
                        if index == 1 {
                            self.windows_list_state.select(Some(0));
                        } else {
                            self.windows_list_state.select(Some(1));
                        }
                    }
                    AppReturn::Continue
                }
                Action::LeftList => {
                    if let Some(index) = self.windows_list_state.selected() {
                        if index == 1 {
                            self.windows_list_state.select(Some(0));
                        } else {
                            self.windows_list_state.select(Some(1));
                        }
                    }
                    AppReturn::Continue
                }
                Action::UpList => {
                    // State based on which tab I am on
                    if let Some(window_index) = self.windows_list_state.selected() {
                        match window_index {
                            0 => {
                                if let Some(selected) = self.test_list_state.selected() {
                                    if selected > 0 {
                                        self.test_list_state.select(Some(selected - 1));
                                    } else {
                                        self.test_list_state.select(Some(self.test_num - 1));
                                    }
                                    self.dispatch(IoEvent::UpdateRef).await;
                                }
                            }
                            1 => {
                                if let Some(selected) = self.log_list_state.selected() {
                                    let size = self.state().get_diffsize().unwrap_or(0);

                                    if selected > 0 {
                                        self.log_list_state.select(Some(selected - 1));
                                    } else {
                                        self.log_list_state.select(Some(size - 1));
                                    }
                                }
                            }
                            _ => return AppReturn::Continue,
                        }
                    }

                    AppReturn::Continue
                }
                Action::DownList => {
                    if let Some(window_index) = self.windows_list_state.selected() {
                        match window_index {
                            0 => {
                                if let Some(selected) = self.test_list_state.selected() {
                                    if selected >= self.test_num - 1 {
                                        self.test_list_state.select(Some(0));
                                    } else {
                                        self.test_list_state.select(Some(selected + 1));
                                    }

                                    self.dispatch(IoEvent::UpdateRef).await;
                                }
                            }
                            1 => {
                                if let Some(selected) = self.log_list_state.selected() {
                                    let size = self.state().get_diffsize().unwrap_or(0);

                                    if selected >= size - 1 {
                                        self.log_list_state.select(Some(0));
                                    } else {
                                        self.log_list_state.select(Some(selected + 1));
                                    }
                                }
                            }
                            _ => return AppReturn::Continue,
                        }
                    }

                    AppReturn::Continue
                }
                Action::ActivateValgrind => {
                    self.valgrind_enabled = !self.valgrind_enabled;

                    AppReturn::Continue
                }
                Action::RunCheckstyle => {
                    self.state.update_checkstyle();
                    if let Some(true) = self.state.get_checkstyle() {
                        self.dispatch(IoEvent::LoadChecksyle).await;
                    }

                    AppReturn::Continue
                } // Action::SendVMChecker => {
                  //     info!("Preparing vmchecker send");
                  //     self.dispatch(IoEvent::SendVMChecker).await;
                  //     AppReturn::Continue
                  // }
                  // Action::OpenVMChecker => {
                  //     self.state.update_vmcheck();
                  //     self.vmchecker_out.push_str("Waiting for server response");
                  //     if let Some(true) = self.state.get_vmcheck() {
                  //         self.dispatch(IoEvent::LoadVMChecker).await;
                  //     }

                  //     AppReturn::Continue
                  // }
            }
        } else {
            warn!("No action accociated to {}", key);
            AppReturn::Continue
        }
    }

    /// We could update the app or dispatch event on tick
    pub async fn update_on_tick(&mut self) -> AppReturn {
        // here we just increment a counter
        self.state.incr_tick();
        if self.unwritten_data && self.state.count_tick().unwrap() % 100 == 0 {
            let data = self.save_data();
            self.dispatch(IoEvent::SaveData(data)).await;
            self.unwritten_data = false;
        }
        AppReturn::Continue
    }

    /// Send a network event to the IO thread
    pub async fn dispatch(&mut self, action: IoEvent) {
        // `is_loading` will be set to false again after the async action has finished in io/handler.rs
        self.is_loading = true;
        if let Err(e) = self.io_tx.send(action).await {
            self.is_loading = false;
            error!("Error from dispatch {}", e);
        };
    }

    pub fn actions(&self) -> &Actions {
        &self.actions
    }
    pub fn state(&self) -> &AppState {
        &self.state
    }

    pub fn is_loading(&self) -> bool {
        self.is_loading
    }

    pub fn initialized(&mut self) {
        // Update contextual actions
        self.actions = vec![
            Action::Quit,
            Action::Run,
            Action::RunFailed,
            Action::RunCurrent,
            Action::RightList,
            Action::LeftList,
            Action::UpList,
            Action::DownList,
            Action::ActivateValgrind,
            Action::RunCheckstyle,
            Action::RunTaskOne,
            Action::RunTaskTwo,
            Action::RunTaskThree,
            // Action::SendVMChecker,
            // Action::OpenVMChecker,
        ]
        .into();
        self.state = AppState::initialized()
    }

    pub fn loaded(&mut self) {
        self.is_loading = false;
    }

    pub fn calculate_score(&self) -> isize {
        let mut score = 0isize;
        for execs in self.test_list.iter() {
            for test in execs {
                score += test.status.parse::<isize>().unwrap_or(0);
            }
        }

        if self.errors.iter().sum::<i32>() == 0 {
            score += CHECKSTYLE_SCORE;
        }

        score
    }

    pub fn save_data(&mut self) -> Data {
        Data {
            tests: self.test_list.to_vec(),
            test_path: self.test_path.clone(),
            exec_name: self.exec_name.clone(),
            valgrind_enabled: self.valgrind_enabled,
        }
    }
}

pub fn get_list_index(lists: &Vec<Vec<Test>>, index: usize) -> (usize, usize) {
    let mut cumulative_index = 0;
    let mut list_index = 0;

    for list in lists {
        cumulative_index += list.len();

        if cumulative_index > index {
            let index_in_list = index - (cumulative_index - list.len());
            return (index_in_list, list_index);
        }

        list_index += 1;
    }

    return (0, 0);
}
