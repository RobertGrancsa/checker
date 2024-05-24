use crate::app::Data;

pub mod handler;

#[derive(Debug, Clone)]
pub enum IoEvent {
    Initialize, // Launch to initialize the application
    RunTest(usize, usize),
    RunAll(usize),
    RunFailed(Vec<(usize, usize)>),
    SaveData(Data),
    LoadChecksyle,
    Make,
    UpdateRef,
    // SendVMChecker,
    // LoadVMChecker,
}
