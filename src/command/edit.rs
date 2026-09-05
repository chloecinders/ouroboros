use serde_json::Value;

use crate::command::args::Field;
use crate::domain::action::Amendment;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Change {
    pub field: &'static str,
    pub policy: Amendment,
    pub before: Value,
    pub after: Value,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Verdict {
    Unchanged,
    Amend(Vec<Change>),
    Reject(&'static str),
}

pub fn compare(fields: &'static [Field], before: &Value, after: &Value) -> Verdict {
    let (Value::Object(before), Value::Object(after)) = (before, after) else {
        return Verdict::Reject("the recorded arguments are unreadable");
    };

    let mut changes = Vec::new();

    for field in fields {
        let was = before.get(field.name).unwrap_or(&Value::Null);
        let now = after.get(field.name).unwrap_or(&Value::Null);

        if was == now {
            continue;
        }

        if field.amend == Amendment::Never {
            return Verdict::Reject("that argument cannot be edited");
        }

        changes.push(Change {
            field: field.name,
            policy: field.amend,
            before: was.clone(),
            after: now.clone(),
        });
    }

    for name in after.keys() {
        if !fields.iter().any(|field| field.name == name) {
            return Verdict::Reject("the edit added an argument this command does not take");
        }
    }

    if changes.is_empty() {
        return Verdict::Unchanged;
    }

    Verdict::Amend(changes)
}
