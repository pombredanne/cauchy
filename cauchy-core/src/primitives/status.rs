#[derive(PartialEq, Clone)]
pub enum Status {
    StatePush,
    StatePull,
    WorkPull,
    Idle,
}

impl Default for Status {
    fn default() -> Self {
        Status::Idle
    }
}

#[derive(PartialEq, Clone)]
pub enum WorkStatus {
    Pulling,
    Idle,
}

impl Default for WorkStatus {
    fn default() -> Self {
        WorkStatus::Idle
    }
}

impl WorkStatus {
    pub fn to_str(&self) -> &'static str {
        match self {
            WorkStatus::Pulling => "pulling",
            WorkStatus::Idle => "idle",
        }
    }
}

impl Status {
    pub fn to_str(&self) -> &'static str {
        match self {
            Status::StatePush => "state pushing",
            Status::StatePull => "state pulling",
            Status::WorkPull => "work pulling",
            Status::Idle => "idle",
        }
    }
}
