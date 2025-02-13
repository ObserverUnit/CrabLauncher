use serde::Deserialize;

#[derive(Debug, Deserialize, Clone, Copy)]
#[serde(rename_all = "snake_case")]
pub enum VersionKind {
    Release,
    Snapshot,
    OldAlpha,
    OldBeta,
}

#[derive(Deserialize, Debug)]
pub struct Version {
    pub id: String,
    pub url: String,
}

#[derive(Debug, Deserialize)]
pub struct VersionManifest {
    pub versions: Vec<Version>,
}
