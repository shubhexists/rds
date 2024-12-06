use core::str;
use regex::Regex;
use serenity::all::standard::CommandError;
use std::process::{Command, Output};
use tracing::{error, info};

pub async fn get_presigned_url(url: &str) -> Result<String, CommandError> {
    let output: Output = Command::new("spotdl").arg("url").arg(url).output()?;

    if output.status.success() {
        let output_str: &str = str::from_utf8(&output.stdout)?;
        info!("SpotDL Output: {}", output_str);

        let re: Regex = Regex::new(r"https://[^\s]+").unwrap();

        let urls: Vec<&str> = re
            .find_iter(output_str)
            .map(|mat: regex::Match<'_>| mat.as_str())
            .collect();

        if let Some(last_url) = urls.last() {
            println!("Extracted URL: {}", last_url);
             Ok(last_url.to_string())
        } else {
             Err("No URL found".into())
        }
    } else {
        let error_msg: &str = str::from_utf8(&output.stderr).unwrap_or("Unknown error");
        error!("SpotDL error: {}", error_msg);
        Err(error_msg.into())
    }
}
