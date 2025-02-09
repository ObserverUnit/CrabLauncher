use serde::Deserialize;
use serde_json::Value;

use crate::json::client::Os;

#[derive(Deserialize, Debug, Clone)]
pub struct Rule {
    pub action: String,
    pub features: Option<Value>,
    pub os: Option<Os>,
}

impl Rule {
    fn matches(&self) -> bool {
        if let Some(ref os) = self.os {
            if let Some(ref os_name) = os.name {
                if *os_name != crate::OS {
                    return false;
                }
            }

            if let Some(ref os_arch) = os.arch {
                if *os_arch != crate::ARCH {
                    return false;
                }
            }
        }
        true
    }
    pub fn is_allowed(&self) -> bool {
        let is_matched = self.matches();
        match self.action.as_str() {
            "allow" => is_matched,
            "disallow" => !is_matched,
            _ => false,
        }
    }
}
