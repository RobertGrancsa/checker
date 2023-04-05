use std::fs;

use log::{debug, error, info, warn};
use serde::{Deserialize, Serialize};
use tui::widgets::ListState;

use self::actions::Actions;
use self::state::AppState;
use crate::app::actions::Action;
use crate::inputs::key::Key;
use crate::io::IoEvent;

pub mod actions;
pub mod state;
pub mod ui;

const DB_PATH: &str = "./data.json";

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Test {
    pub id: usize,
    pub name: String,
    pub status: String,
    pub log: String,
    pub time_normal: f64,
    pub time_valgrind: f64,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Data {
    commands: Vec<String>,
    tests: Vec<Test>,
    test_path: String,
    exec_name: String,
}

// pub struct RunningTest {
// 	process: Result<Child>,
// 	index: usize,
// }

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
    pub titles: Vec<&'static str>,
    pub selected_tab: usize,
    pub test_list: Vec<Test>,
    pub commands: Vec<String>,
    test_list_state: ListState,
    cmd_list_state: ListState,

    pub valgrind_enabled: bool,
    pub test_path: String,
    pub exec_name: String,
    // pub running_children: Vec<RunningTest>,
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
        let commands = json.commands;
        let mut test_list_state = ListState::default();
        test_list_state.select(Some(0));
        let mut cmd_list_state = ListState::default();
        cmd_list_state.select(Some(0));
        let valgrind_enabled = false;
        let titles = vec!["Test", "Menu", "Tab2", "Tab3"];
        let selected_tab = 0usize;
        let unwritten_data = false;
        // let running_children: Vec<RunningTest> = Vec::new();

        Self {
            io_tx,
            actions,
            is_loading,
            unwritten_data,
            state,
            titles,
            selected_tab,
            test_list,
            commands,
            test_list_state,
            cmd_list_state,
            valgrind_enabled,
            test_path,
            exec_name,
            // running_children,
        }
    }

    /// Handle a user action
    pub async fn do_action(&mut self, key: Key) -> AppReturn {
        if let Some(action) = self.actions.find(key) {
            debug!("Run action [{:?}]", action);
            match action {
                Action::Quit => AppReturn::Exit,
                Action::Sleep => {
                    if let Some(duration) = self.state.duration().cloned() {
                        // Sleep is an I/O action, we dispatch on the IO channel that's run on another thread
                        self.dispatch(IoEvent::Sleep(duration)).await
                    }
                    AppReturn::Continue
                }
                Action::Run => {
                    self.dispatch(IoEvent::RunAll(self.test_list.len())).await;
                    AppReturn::Continue
                }
                Action::RunCurrent => {
                    if let Some(index) = self.test_list_state.selected() {
                        self.test_list[index].status.clear();
                        self.test_list[index].status.push_str("RUNNING");
                        self.dispatch(IoEvent::RunTest(index)).await;
                    } else {
                        warn!("No test selected");
                    }
                    AppReturn::Continue
                }
                // IncrementDelay and DecrementDelay is handled in the UI thread
                Action::IncrementDelay => {
                    self.state.increment_delay();
                    AppReturn::Continue
                }
                // Note, that we clamp the duration, so we stay >= 0
                Action::DecrementDelay => {
                    self.state.decrement_delay();
                    AppReturn::Continue
                }
                Action::UpList => {
                    // State based on which tab I am on
                    if let Some(selected) = self.test_list_state.selected() {
                        if selected > 0 {
                            self.test_list_state.select(Some(selected - 1));
                        } else {
                            self.test_list_state.select(Some(self.test_list.len() - 1));
                        }
                    }
                    AppReturn::Continue
                }
                Action::DownList => {
                    if let Some(selected) = self.test_list_state.selected() {
                        if selected >= self.test_list.len() - 1 {
                            self.test_list_state.select(Some(0));
                        } else {
                            self.test_list_state.select(Some(selected + 1));
                        }
                    }
                    AppReturn::Continue
                }
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
        if self.unwritten_data && self.state.count_tick().unwrap() % 10 == 0 {
            let data = self.save_data();
            info!("Writing data");
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
            Action::Sleep,
            Action::Run,
            Action::RunCurrent,
            Action::IncrementDelay,
            Action::DecrementDelay,
            Action::UpList,
            Action::DownList,
        ]
        .into();
        self.state = AppState::initialized()
    }

    pub fn loaded(&mut self) {
        self.is_loading = false;
    }

    pub fn slept(&mut self) {
        self.state.incr_sleep();
    }

    pub fn save_data(&mut self) -> Data {
        Data {
            commands: self.commands.to_vec(),
            tests: self.test_list.to_vec(),
            test_path: self.test_path.clone(),
            exec_name: self.exec_name.clone(),
        }
    }
}
