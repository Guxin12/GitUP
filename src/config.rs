use std::path::Path;
use std::fs;

#[derive(Debug, Default, Clone)]
pub struct Config {
    pub dir: String,
    pub git_name: String,
    pub git_email: String,
    pub git_remote_name: String,
    pub git_remote_repo: String,
    pub git_branch: String,
}

impl Config {
    pub fn load(path: &Path) -> Self {
        let content = fs::read_to_string(path).unwrap_or_default();
        let mut config = Self::default();
        for line in content.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with(';') || line.starts_with('#') {
                continue;
            }
            if let Some((key, value)) = line.split_once('=') {
                let key = key.trim();
                let mut value = value.trim().to_string();
                if value.starts_with('"') && value.ends_with('"') {
                    value = value[1..value.len()-1].to_string();
                }
                match key {
                    "DIR" => config.dir = value,
                    "GIT_NAME" => config.git_name = value,
                    "GIT_EMAIL" => config.git_email = value,
                    "GIT_REMOTE_NAME" => config.git_remote_name = value,
                    "GIT_REMOTE_REPO" => config.git_remote_repo = value,
                    "GIT_BRANCH" => config.git_branch = value,
                    _ => {}
                }
            }
        }
        config
    }

    pub fn save(&self, path: &Path) -> Result<(), String> {
        let content = format!(
            "DIR=\"{}\"\nGIT_NAME=\"{}\"\nGIT_EMAIL=\"{}\"\nGIT_REMOTE_NAME=\"{}\"\nGIT_REMOTE_REPO=\"{}\"\nGIT_BRANCH=\"{}\"\n",
            self.dir, self.git_name, self.git_email, self.git_remote_name, self.git_remote_repo, self.git_branch
        );
        fs::write(path, content).map_err(|e| format!("保存配置文件失败: {}", e))?;
        Ok(())
    }

    pub fn is_complete(&self) -> bool {
        !self.dir.is_empty() && !self.git_name.is_empty()
    }
}