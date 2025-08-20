#[derive(Debug, sqlx::Type)]
#[sqlx(type_name = "action_type", rename_all="lowercase")]
pub enum ActionType {
    Warn,
    Kick,
    Ban,
    Softban
}

impl std::fmt::Display for ActionType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ActionType::Warn => write!(f, "warn"),
            ActionType::Kick => write!(f, "kick"),
            ActionType::Ban => write!(f, "ban"),
            ActionType::Softban => write!(f, "softban"),
        }
    }
}


pub fn run_migrations() {
    init_223320250818();
}

pub fn init_223320250818() {
    
}
