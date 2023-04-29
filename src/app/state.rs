use std::time::Duration;

#[derive(Clone)]
pub enum AppState {
    Init,
    Initialized {
        duration: Duration,
        counter_tick: u64,
        checkstyle: bool,
    },
}

impl AppState {
    pub fn initialized() -> Self {
        let duration = Duration::from_secs(1);
        let counter_tick = 0;
        let checkstyle = false;
        Self::Initialized {
            duration,
            counter_tick,
            checkstyle,
        }
    }

    pub fn is_initialized(&self) -> bool {
        matches!(self, &Self::Initialized { .. })
    }

    pub fn incr_tick(&mut self) {
        if let Self::Initialized { counter_tick, .. } = self {
            *counter_tick += 1;
        }
    }

    pub fn count_tick(&self) -> Option<u64> {
        if let Self::Initialized { counter_tick, .. } = self {
            Some(*counter_tick)
        } else {
            None
        }
    }

    pub fn update_checkstyle(&mut self) {
        if let Self::Initialized { checkstyle, .. } = self {
            *checkstyle = !*checkstyle;
        }
    }

    pub fn get_checkstyle(&self) -> Option<bool> {
        if let Self::Initialized { checkstyle, .. } = self {
            Some(*checkstyle)
        } else {
            None
        }
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::Init
    }
}
