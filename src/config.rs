use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct Config {
    pub project_root: PathBuf,
}

impl Config {
    pub fn from_env() -> Self {
        let project_root = std::env::var("KUBEVIRT_PROJECT_ROOT")
            .map(PathBuf::from)
            .unwrap_or_else(|_| {
                let home = std::env::var("HOME").unwrap_or_else(|_| ".".into());
                PathBuf::from(home).join("Developer/Projects/kubevirt-ui")
            });
        Self { project_root }
    }

    pub fn playwright_root(&self) -> PathBuf {
        self.project_root.join("playwright")
    }
}
