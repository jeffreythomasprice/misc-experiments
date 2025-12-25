use std::path::Path;

use anyhow::{Result, anyhow};
use regex::Regex;
use tokio::io::AsyncBufReadExt;

use crate::process::exec;

pub async fn get_page_count(path: &Path) -> Result<u32> {
    let r = Regex::new(r"^NumberOfPages:\s*([0-9]+)$")?;
    let output = exec(
        "pdftk",
        &[
            path.to_str().ok_or(anyhow!("failed to get str from path: {:?}", path))?,
            "dump_data",
        ],
    )
    .await?;
    let result = output
        .split("\n")
        .map(|s| s.trim())
        .filter_map(|s| match r.captures(s).map(|c| c.extract()) {
            Some((_, [value])) => value.parse().ok(),
            None => None,
        })
        .collect::<Vec<_>>();
    match result.first() {
        Some(result) => Ok(*result),
        None => Err(anyhow!("didn't find number of pages result"))?,
    }
}

pub async fn extract_pdf_pages_into_new_pdf(input_path: &Path, output_dir: &Path, first_page: u32, last_page: u32) -> Result<String> {
    let input_file_name = input_path
        .file_name()
        .ok_or(anyhow!("failed to get file name for input path: {:?}", input_path))?
        .to_string_lossy()
        .to_string();
    let input_file_extension = input_path
        .extension()
        .ok_or(anyhow!("failed to get file extension for input path: {:?}", input_path))?
        .to_string_lossy()
        .to_string();

    let file_name_chars = input_file_name.chars().collect::<Vec<_>>();
    let file_ext_chars = input_file_extension.chars().collect::<Vec<_>>();
    let file_name_without_extension = input_file_name.chars().collect::<Vec<_>>()[0..(file_name_chars.len() - file_ext_chars.len() - 1)]
        .iter()
        .collect::<String>();
    let output_file_name = format!("{file_name_without_extension}-{first_page}-{last_page}.{input_file_extension}");
    let output_file_path = output_dir
        .join(&output_file_name)
        .to_str()
        .ok_or(anyhow!(
            "failed to get str from output joining output dir {:?} and file name {}",
            output_dir,
            output_file_name
        ))?
        .to_string();

    exec(
        "pdftk",
        &[
            input_path
                .to_str()
                .ok_or(anyhow!("failed to get str from input path: {:?}", input_path))?,
            "cat",
            &format!("{first_page}-{last_page}"),
            "output",
            &output_file_path,
        ],
    )
    .await?;

    Ok(output_file_path)
}

pub async fn extract_pdf_text(path: &Path) -> Result<String> {
    exec(
        "pdftotext",
        &[path.to_str().ok_or(anyhow!("failed to get str from input path: {:?}", path))?, "-"],
    )
    .await
}
