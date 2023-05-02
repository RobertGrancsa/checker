use crate::app::Data;

pub mod handler;

#[derive(Debug, Clone)]
pub enum IoEvent {
    Initialize, // Launch to initialize the application
    RunTest(usize),
    RunAll(usize),
    RunFailed(Vec<usize>),
    SaveData(Data),
    LoadChecksyle,
    UpdateRef,
}
