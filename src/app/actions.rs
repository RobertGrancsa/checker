use std::collections::HashMap;
use std::fmt::{self, Display};
use std::slice::Iter;

use crate::inputs::key::Key;

/// We define all available action
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum Action {
    Quit,
    Run,
    RunFailed,
    RunCurrent,
    RightList,
    LeftList,
    UpList,
    DownList,
    ActivateValgrind,
    RunCheckstyle,
    RunTaskOne,
    RunTaskTwo,
    SendVMChecker,
    OpenVMChecker,
}

impl Action {
    /// All available actions
    pub fn iterator() -> Iter<'static, Action> {
        static ACTIONS: [Action; 14] = [
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
            Action::SendVMChecker,
            Action::OpenVMChecker,
        ];
        ACTIONS.iter()
    }

    /// List of key associated to action
    pub fn keys(&self) -> &[Key] {
        match self {
            Action::Quit => &[Key::Ctrl('c'), Key::Char('q')],
            Action::Run => &[Key::Char('r')],
            Action::RunFailed => &[Key::Char('f')],
            Action::RunCurrent => &[Key::Enter],
            Action::RightList => &[Key::Right],
            Action::LeftList => &[Key::Left],
            Action::UpList => &[Key::Up],
            Action::DownList => &[Key::Down],
            Action::ActivateValgrind => &[Key::Char('v')],
            Action::RunCheckstyle => &[Key::Char('c')],
            Action::RunTaskOne => &[Key::Char('1')],
            Action::RunTaskTwo => &[Key::Char('2')],
            Action::SendVMChecker => &[Key::Char('p')],
            Action::OpenVMChecker => &[Key::Char('o')],
        }
    }
}

/// Could display a user friendly short description of action
impl Display for Action {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let str = match self {
            Action::Quit => "Quit",
            Action::Run => "Run all tests",
            Action::RunFailed => "Run failed tests",
            Action::RunCurrent => "Run current test",
            Action::RightList => "Switch window to the right",
            Action::LeftList => "Switch window to the left",
            Action::UpList => "Go up the list",
            Action::DownList => "Go down the list",
            Action::ActivateValgrind => "Turn On/Off valgrind",
            Action::RunCheckstyle => "Run checkstyle",
            Action::RunTaskOne => "Run task-1",
            Action::RunTaskTwo => "Run task-2",
            Action::SendVMChecker => "Send homework to vmchecker",
            Action::OpenVMChecker => "Check vmchecker output",
        };
        write!(f, "{}", str)
    }
}

/// The application should have some contextual actions.
#[derive(Default, Debug, Clone)]
pub struct Actions(Vec<Action>);

impl Actions {
    /// Given a key, find the corresponding action
    pub fn find(&self, key: Key) -> Option<&Action> {
        Action::iterator()
            .filter(|action| self.0.contains(action))
            .find(|action| action.keys().contains(&key))
    }

    /// Get contextual actions.
    /// (just for building a help view)
    pub fn actions(&self) -> &[Action] {
        self.0.as_slice()
    }
}

impl From<Vec<Action>> for Actions {
    /// Build contextual action
    ///
    /// # Panics
    ///
    /// If two actions have same key
    fn from(actions: Vec<Action>) -> Self {
        // Check key unicity
        let mut map: HashMap<Key, Vec<Action>> = HashMap::new();
        for action in actions.iter() {
            for key in action.keys().iter() {
                match map.get_mut(key) {
                    Some(vec) => vec.push(*action),
                    None => {
                        map.insert(*key, vec![*action]);
                    }
                }
            }
        }
        let errors = map
            .iter()
            .filter(|(_, actions)| actions.len() > 1) // at least two actions share same shortcut
            .map(|(key, actions)| {
                let actions = actions
                    .iter()
                    .map(Action::to_string)
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("Conflict key {} with actions {}", key, actions)
            })
            .collect::<Vec<_>>();
        if !errors.is_empty() {
            panic!("{}", errors.join("; "))
        }

        // Ok, we can create contextual actions
        Self(actions)
    }
}
