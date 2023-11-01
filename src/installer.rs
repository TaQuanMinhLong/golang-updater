use std::error::Error;

use crate::config::*;
use crate::utils::*;
use serde::Deserialize;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub struct Version(pub u32, pub u32, pub u32);

impl Version {
    pub fn from_str(s: &str) -> Result<Version, Box<dyn Error>> {
        let mut parts = s.trim_start_matches("go").split('.');
        let major = parts
            .next()
            .unwrap_or("")
            .parse::<u32>()
            .expect("Failed to parse major part as u32");
        let minor = parts
            .next()
            .unwrap_or("")
            .parse::<u32>()
            .expect("Failed to parse minor part as u32");
        let patch = parts
            .next()
            .unwrap_or("")
            .parse::<u32>()
            .expect("Failed to parse patch part as u32");
        Ok(Version(major, minor, patch))
    }
}

impl<'de> Deserialize<'de> for Version {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let input = String::deserialize(deserializer)?;
        let version = Version::from_str(&input).expect("Failed to parse version");
        Ok(version)
    }
}

pub fn get_current_version() -> Option<Version> {
    use std::process;
    let output = process::Command::new("go").arg("version").output();
    match output {
        Err(_) => None,
        Ok(output) => {
            let stdout = String::from_utf8(output.stdout).expect("Failed to parse stdout");
            let version = Version::from_str(&stdout.split_whitespace().nth(2)?)
                .expect("Failed to parse version");
            Some(version)
        }
    }
}

pub fn config_path_env() {
    let shell_profile = [
        &format!("export GOROOT={}", GO_ROOT),
        &format!("export GOPATH={}", GO_PATH),
        "export PATH=$PATH:$GOPATH/bin",
        "export PATH=$PATH:$GOROOT/bin",
    ];
    std::thread::scope(|s| {
        for line in shell_profile {
            s.spawn(move || {
                let res = get_output(&format!("cat {} | grep '{}'", SHELL_PROFILE, line));
                if res.status.code() == Some(1) && res.stdout.is_empty() {
                    exec_cmd(&format!("echo '{}' >> {}", line, SHELL_PROFILE))
                }
            });
        }
    });
}

pub fn remove_old_version() {
    exec_cmd(&format!("sudo rm -rf {}", GO_ROOT))
}

pub fn remove_temp_folder(temp_folder: &std::path::Path) {
    exec_cmd(&format!("sudo rm -rf {}", temp_folder.to_string_lossy()))
}
