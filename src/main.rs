mod config;
mod downloader;
mod installer;
mod utils;

use config::*;
use downloader::*;
use installer::*;
use utils::*;

pub type Result<T> = core::result::Result<T, Box<dyn std::error::Error>>;

fn main() -> Result<()> {
    println!("\nChecking for updates...");
    let version_meta = check_update()?;
    let latest = version_meta.version;
    let current = get_current_version();

    if current.is_some() {
        let current = current.unwrap();
        if current == latest {
            println!("Go version has already been latest");
            std::process::exit(1)
        }
    }
    println!("\nRemoving old version...");
    remove_old_version();

    let file_name = &version_meta.filename;
    let file_size = get_file_size(&file_name)?;
    println!("\nDownloading latest version...");
    download(&file_name, file_size)?;

    let temp_dir = get_temp_dir();
    exec_cmd(&format!(
        "sudo tar -C {} -xzf {}",
        GO_ROOT.trim_end_matches("/go"),
        &temp_dir.join(&file_name).to_string_lossy()
    ));

    println!("\nRemoving temporary download...");
    remove_temp_folder(&temp_dir);

    println!("\nConfig PATH environment variable");
    config_path_env();

    let version = get_stdout("go version");
    println!("\nSuccessfully updated go: {}", version);

    Ok(())
}
