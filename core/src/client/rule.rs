use serde::Deserialize;
use serde_json::Value;

use super::Os;

#[derive(Deserialize, Debug, Clone, Copy)]
#[serde(rename_all = "lowercase")]
pub enum RuleAction {
    Allow,
    Disallow,
}
#[derive(Deserialize, Debug, Clone)]
pub struct Rule {
    pub action: RuleAction,
    pub features: Option<Value>,
    pub os: Option<Os>,
}

impl Rule {
    /// Returns true if the current platform allows the given [`Rule`]
    /// use [`Rule::is_allowed`] to check if a rule is allowed on a given platform this only checks if the rule matches the current platform
    fn matches(&self) -> bool {
        // TODO: match features
        (self.os.is_none() || self.os.as_ref().is_some_and(|os| os.matches()))
            && self.features.is_none()
    }
    pub fn is_allowed(&self) -> bool {
        let is_matched = self.matches();
        match self.action {
            RuleAction::Allow => is_matched,
            RuleAction::Disallow => !is_matched,
        }
    }
}
