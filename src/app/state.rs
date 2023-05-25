use std::time::Duration;

#[derive(Clone)]
pub enum AppState {
    Init,
    Initialized {
        duration: Duration,
        counter_tick: u64,
        checkstyle: bool,
        vmcheck_output: bool,
        diff_size: usize,
    },
}

impl AppState {
    pub fn initialized() -> Self {
        let duration = Duration::from_secs(1);
        let counter_tick = 0;
        let checkstyle = false;
        let vmcheck_output = false;
        let diff_size = 0;
        Self::Initialized {
            duration,
            counter_tick,
            checkstyle,
            vmcheck_output,
            diff_size,
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

    pub fn update_vmcheck(&mut self) {
        if let Self::Initialized { vmcheck_output, .. } = self {
            *vmcheck_output = !*vmcheck_output;
        }
    }

    pub fn get_checkstyle(&self) -> Option<bool> {
        if let Self::Initialized { checkstyle, .. } = self {
            Some(*checkstyle)
        } else {
            None
        }
    }

    pub fn get_vmcheck(&self) -> Option<bool> {
        if let Self::Initialized { vmcheck_output, .. } = self {
            Some(*vmcheck_output)
        } else {
            None
        }
    }

    pub fn set_diffsize(&mut self, size: usize) {
        if let Self::Initialized { diff_size, .. } = self {
            *diff_size = size;
        }
    }

    pub fn get_diffsize(&self) -> Option<usize> {
        if let Self::Initialized { diff_size, .. } = self {
            Some(*diff_size)
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
