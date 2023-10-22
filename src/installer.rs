use crate::utils::*;
use serde::Deserialize;

pub const GO_ROOT: &'static str = "/usr/local/go";
pub const GO_PATH: &'static str = "~/.go";

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub struct Version(u32, u32, u32);

impl Version {
    fn new(major: u32, minor: u32, patch: u32) -> Self {
        Version(major, minor, patch)
    }
}

impl<'de> Deserialize<'de> for Version {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        let mut parts = s.trim_start_matches("go").split('.');
        let major = parts
            .next()
            .unwrap_or("")
            .parse::<u32>()
            .map_err(serde::de::Error::custom)?;
        let minor = parts
            .next()
            .unwrap_or("")
            .parse::<u32>()
            .map_err(serde::de::Error::custom)?;
        let patch = parts
            .next()
            .unwrap_or("")
            .parse::<u32>()
            .map_err(serde::de::Error::custom)?;
        Ok(Version(major, minor, patch))
    }
}

pub fn get_current_version() -> Option<Version> {
    use std::process;
    let output = process::Command::new("go").arg("version").output();
    match output {
        Err(_) => None,
        Ok(output) => {
            let stdout = String::from_utf8(output.stdout).expect("Failed to parse stdout");
            let version_string = stdout.split_whitespace().collect::<Vec<&str>>()[2];
            let mut parts = version_string.trim_start_matches("go").split(".");
            let major = parts
                .next()
                .unwrap_or("")
                .parse::<u32>()
                .expect("Failed to parse 'major' of version string");
            let minor = parts
                .next()
                .unwrap_or("")
                .parse::<u32>()
                .expect("Failed to parse 'minor' of version string");
            let patch = parts
                .next()
                .unwrap_or("")
                .parse::<u32>()
                .expect("Failed to parse 'patch' of version string");

            Some(Version::new(major, minor, patch))
        }
    }
}

pub fn config_path_env() {
    let bashrc = [
        &format!("export GOROOT={}", GO_ROOT),
        &format!("export GOPATH={}", GO_PATH),
        "export PATH=$PATH:$GOPATH/bin",
        "export PATH=$PATH:$GOROOT/bin",
    ];
    std::thread::scope(|s| {
        for line in bashrc {
            s.spawn(move || {
                let res = get_output(&format!("cat ~/.bashrc | grep '{}'", line));
                if res.status.code() == Some(1) && res.stdout.is_empty() {
                    exec_cmd(&format!("echo '{}' >> ~/.bashrc", line))
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
