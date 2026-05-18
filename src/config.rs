use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct Config {
    pub project_root: PathBuf,
}

impl Config {
    pub fn from_env() -> Self {
        let project_root = std::env::var("KUBEVIRT_PROJECT_ROOT")
            .map(PathBuf::from)
            .expect("KUBEVIRT_PROJECT_ROOT must be set to the root of the kubevirt-ui repo");
        Self { project_root }
    }

    pub fn playwright_root(&self) -> PathBuf {
        self.project_root.join("playwright")
    }
}
