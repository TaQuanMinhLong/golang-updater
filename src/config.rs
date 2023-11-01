/// Default GO_ROOT at `/usr/local/go`
pub const GO_ROOT: &'static str = "/usr/local/go";

/// Default GO_PATH at `~/.go`
pub const GO_PATH: &'static str = "~/.go";

/// Default download url
pub const URL: &str = "https://go.dev/dl/";

/// The path to your bash profile
///
/// If you use zsh then it should be `~/.zshrc`
///
/// For bash simply `~/.bashrc`
pub const SHELL_PROFILE: &'static str = "~/.zshrc";

pub fn get_temp_dir() -> std::path::PathBuf {
    std::env::current_dir()
        .expect("Failed to get current directory")
        .join("temp")
}

pub fn get_num_threads() -> u64 {
    std::thread::available_parallelism()
        .expect("Failed to get available parallelism")
        .get() as u64
}
