use std::fmt::Display;

#[derive(Debug, Clone)]
pub struct AnyError {
    pub inner: String,
}

impl AnyError {
    pub fn new(id: &'static str) -> Self {
        Self {
            inner: id.to_string(),
        }
    }
}

impl Display for AnyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "AnyError; id: {}", self.inner)
    }
}

impl std::error::Error for AnyError {}
