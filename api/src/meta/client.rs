use std::{collections::HashMap, path::PathBuf};

use serde::Deserialize;

use super::utils::{Os, OsName};

#[derive(Deserialize, Debug, Clone, Copy)]
#[serde(rename_all = "lowercase")]
pub enum RuleAction {
    Allow,
    Disallow,
}
#[derive(Deserialize, Debug, Clone)]
pub struct Rule {
    pub action: RuleAction,
    pub features: Option<HashMap<String, bool>>,
    pub os: Option<Os>,
}

impl Rule {
    /// Returns true if the current platform allows the given [`Rule`]
    /// use [`Rule::is_allowed`] to check if a rule is allowed on a given platform this only checks if the rule matches the current platform isn't impacted by the action
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

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Download {
    #[serde(rename = "path")]
    pub sub_path: Option<PathBuf>,
    // TODO: verify sha1 and size
    pub sha1: String,
    pub size: i32,
    pub url: String,
}

#[derive(Debug, Deserialize)]
pub struct Downloads {
    pub client: Download,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum ArgValue {
    Value(String),
    Values(Vec<String>),
}

#[derive(Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum Argument {
    Arg(String),
    Rule { rules: Vec<Rule>, value: ArgValue },
}

impl Argument {
    fn into_raw(self) -> Vec<String> {
        match self {
            Argument::Arg(arg) => vec![arg],
            Argument::Rule { rules, value } => {
                if rules.iter().all(Rule::is_allowed) {
                    match value {
                        ArgValue::Value(value) => vec![value],
                        ArgValue::Values(values) => values,
                    }
                } else {
                    vec![]
                }
            }
        }
    }
}
#[derive(Deserialize, Debug)]
#[serde(untagged)]
pub enum Arguments {
    Args {
        game: Vec<Argument>,
        jvm: Vec<Argument>,
    },
    MinecraftArgs(String),
}

impl Arguments {
    /// maps `Arguments` to (JVM Args, Game Args)
    /// only maps arguments that are allowed by their rules
    pub fn into_raw(self) -> (Vec<String>, Vec<String>) {
        match self {
            Arguments::Args { game, jvm } => {
                let jvm: Vec<String> = jvm.into_iter().map(Argument::into_raw).flatten().collect();
                let game = game.into_iter().map(Argument::into_raw).flatten().collect();
                (jvm, game)
            }
            Arguments::MinecraftArgs(args) => {
                let game = args.split(' ').map(|arg| arg.to_string()).collect();
                // FIXME: a little hack to have jvm args when on older versions
                // TODO: fix this when we have
                // our own meta format
                let jvm = [
                    "-Djava.library.path=${natives_directory}",
                    "-cp",
                    r"${classpath}",
                ];
                let jvm = jvm.into_iter().map(|x| x.to_string()).collect();

                (jvm, game)
            }
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JavaVersion {
    pub component: String,
    pub major_version: u16,
}

#[derive(Debug, Deserialize, Clone)]
pub struct LibraryDownload {
    pub artifact: Option<Download>,
    pub classifiers: Option<HashMap<String, Download>>,
}

#[derive(Debug, Deserialize)]
pub struct Extract {
    pub exclude: Option<Vec<PathBuf>>,
}

pub type Natives = HashMap<OsName, String>;
#[derive(Debug, Deserialize)]
pub struct Library {
    pub downloads: LibraryDownload,
    pub extract: Option<Extract>,
    pub natives: Option<Natives>,
    pub rules: Option<Vec<Rule>>,
}
impl Library {
    pub fn is_allowed(&self) -> bool {
        self.rules.is_none()
            || self
                .rules
                .as_ref()
                .is_some_and(|rules| rules.iter().all(Rule::is_allowed))
    }

    // TODO: consider this when implementing our own meta format
    /// returns the native library [`Download`] required by the library for the current platform
    pub fn platform_native(&self) -> Option<&Download> {
        let natives = self.natives.as_ref()?;
        let classifiers = self.downloads.classifiers.as_ref()?;
        let mut results = natives
            .iter()
            .filter(|(os, _)| **os == crate::OS)
            .map(|(_, native)| classifiers.get(native).unwrap());
        results.next()
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Client {
    #[serde(alias = "minecraftArguments")]
    pub arguments: Arguments,
    pub asset_index: Download,

    pub assets: String,
    pub downloads: Downloads,

    pub java_version: Option<JavaVersion>,
    pub libraries: Vec<Library>,
    pub main_class: String,
}

// assets
#[derive(Deserialize, Debug)]
pub struct Object {
    pub hash: String,
    #[allow(unused)]
    pub size: i32,
}

#[derive(Deserialize, Debug)]
pub struct Index {
    pub objects: HashMap<String, Object>,
}

impl Client {
    /// returns an iterator of all libraries that are required by the current platform
    pub fn libs(&self) -> impl Iterator<Item = &Library> {
        self.libraries.iter().filter(|l| l.is_allowed())
    }
}
