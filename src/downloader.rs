use serde::Deserialize;

use crate::config::*;
use crate::installer::Version;
use crate::utils::*;
use crate::Result;

#[derive(Debug)]
pub struct VersionMeta {
    pub version: Version,
    pub filename: String,
}

#[derive(Debug, Deserialize)]
pub struct JsonResponse {
    pub files: Vec<FileDownload>,
    pub version: Version,
}

#[derive(Debug, Deserialize)]
pub struct FileDownload {
    pub filename: String,
    pub os: String,
    pub arch: String,
}

fn get_arch() -> &'static str {
    let arch = std::env::consts::ARCH;
    match arch {
        "x86" => "386",
        "x86_64" => "amd64",
        "arm" => "armv6l",
        "aarch64" => "arm64",
        "loongarch64" => "loong64",
        "m68k" => "mips",
        "csky" => "mips",
        "mips" => "mips",
        "mips64" => "mips64",
        "powerpc" => "ppc64",
        "powerpc64" => "ppc64",
        "riscv64" => "riscv64",
        "s390x" => "s390x",
        "sparc64" => "amd64",
        _ => "",
    }
}

pub fn check_update() -> Result<VersionMeta> {
    let json_string = get_stdout(&format!(
        "curl {}?mode=json -H \"Accept: application/json\"",
        URL
    ));
    let res = serde_json::from_str::<Vec<JsonResponse>>(&json_string)
        .expect("Failed to deserialize JSON Response");
    let os = std::env::consts::OS;
    let arch = get_arch();
    let latest = &res[0];
    let version = &latest.version;
    let filename = &latest
        .files
        .iter()
        .find(|f| f.os == os && f.arch == arch)
        .expect(&format!(
            "Can not find target file to download with os: {}, arch: {}",
            os, arch
        ))
        .filename;
    let version = version.to_owned();
    let filename = filename.to_owned();

    Ok(VersionMeta { version, filename })
}

pub fn get_file_size(filename: &str) -> Result<u64> {
    let url = format!("{}{}", URL, filename);
    let file_size = get_stdout(&format!(
        "curl -I -L {} | awk '/content-length/ {{print $2}}'",
        url
    ))
    .trim_end()
    .parse::<u64>()?;
    Ok(file_size)
}

pub fn download(file_name: &str, file_size: u64) -> Result<()> {
    use std::fs::{create_dir, read, read_dir, File};
    use std::os::unix::prelude::FileExt;
    use std::thread;
    use std::time::Instant;

    let timer_start = Instant::now();

    let temp_dir = get_temp_dir();

    if !temp_dir.exists() {
        create_dir(&temp_dir).expect("Failed to create temporary folder")
    }

    let num_threads = get_num_threads();

    thread::scope(|s| {
        for i in 0..num_threads {
            let start = file_size / num_threads * i;
            let end = file_size / num_threads * (i + 1);
            let url = format!("{}{}", URL, &file_name);
            let temp_file = format!("temp_{}-{}", start, end);
            let outdir = &temp_dir.join(temp_file);
            let cmd = format!(
                "curl -L -o {} -H \"range: bytes={}-{}\" {}",
                outdir.to_str().unwrap(),
                start,
                end,
                url
            );
            s.spawn(move || exec_cmd(&cmd));
        }
    });

    let file = File::create(temp_dir.join(&file_name))?;

    let entries = read_dir(&temp_dir)?;

    for entry in entries {
        let entry = entry?;
        let filename = entry.file_name();
        let filename = filename.to_string_lossy();
        if filename == file_name {
            continue;
        }
        let offset = filename
            .trim_start_matches("temp_")
            .split("-")
            .next()
            .expect(&format!(
                "Failed to parse temp file: {}",
                &entry.file_name().to_string_lossy()
            ))
            .parse::<u64>()?;
        let buf = read(entry.path())?;
        file.write_at(&buf, offset)?;
    }

    let timer_end = Instant::now();

    println!(
        "\nDownload finished in {}ms",
        timer_end.duration_since(timer_start).as_millis()
    );
    Ok(())
}
