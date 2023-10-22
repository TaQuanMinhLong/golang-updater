use serde::Deserialize;

use crate::installer::Version;
use crate::Result;

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

pub const URL: &str = "https://go.dev/dl/";

pub async fn check_update() -> Result<VersionMeta> {
    let res = reqwest::get(format!("{}?mode=json", URL))
        .await?
        .json::<Vec<JsonResponse>>()
        .await?;

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

pub async fn get_file_size(filename: &str) -> Result<u64> {
    let url = format!("{}{}", URL, filename);
    let file_size = reqwest::get(&url)
        .await?
        .content_length()
        .expect("Could not get the content length header");
    Ok(file_size)
}

pub async fn download(
    file_name: &str,
    file_size: u64,
    temp_folder: &std::path::PathBuf,
) -> Result<()> {
    use futures_util::StreamExt;
    use indicatif::*;
    use reqwest::header::RANGE;
    use std::fs::{create_dir, File};
    use std::io::{Read, Seek, SeekFrom, Write};
    use std::os::unix::prelude::FileExt;
    use std::time::Instant;
    use tempfile::tempfile;

    let progress = MultiProgress::new();
    let style = ProgressStyle::default_bar()
        .template("{msg} {spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {bytes}/{total_bytes} ({eta})")
        .expect("Failed to parse template")
        .progress_chars("#>-");

    const NUM_THREADS: u64 = 18;

    let timer_start = Instant::now();

    let mut tasks: Vec<tokio::task::JoinHandle<(u64, File)>> =
        Vec::with_capacity(NUM_THREADS.try_into().unwrap());

    for i in 0..NUM_THREADS {
        let start = file_size / NUM_THREADS * i;
        let end = file_size / NUM_THREADS * (i + 1);
        let pb = progress.add(ProgressBar::new(end - start + 1));
        pb.set_style(style.clone());
        pb.set_message(format!("Thread {}", i + 1));
        let client = reqwest::Client::new();
        let req = client
            .get(format!("{}{}", URL, &file_name))
            .header(RANGE, format!("bytes={}-{}", start, end));

        let handle = tokio::task::spawn(async move {
            let res = req.send().await.expect("Failed to send request");
            let mut source = res.bytes_stream();
            let mut out = tempfile().expect("Failed to create a new temporary file");

            while let Some(chunk) = source.next().await {
                let chunk = chunk.expect("Failed to get chunk");
                let downloaded = chunk.len() as u64;
                pb.set_position(downloaded);
                out.write_all(&chunk).unwrap();
            }
            out.sync_all().unwrap();
            out.seek(SeekFrom::Start(0)).unwrap();
            (start, out)
        });

        tasks.push(handle)
    }

    let mut parts = Vec::with_capacity(NUM_THREADS.try_into().unwrap());

    for handle in tasks {
        let value = handle.await.unwrap();
        parts.push(value)
    }

    parts.sort_by_key(|(start, _)| *start);

    if !temp_folder.exists() {
        create_dir(temp_folder).expect("Failed to create temporary folder")
    }

    let file = File::create(temp_folder.join(file_name))?;

    for (offset, mut part) in parts {
        let mut buf = Vec::new();
        part.read_to_end(&mut buf)?;
        file.write_at(&buf, offset)?;
    }

    let timer_end = Instant::now();

    println!(
        "\nDownload finished in {}ms",
        timer_end.duration_since(timer_start).as_millis()
    );
    Ok(())
}
