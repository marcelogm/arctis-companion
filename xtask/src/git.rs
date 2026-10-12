use std::path::Path;
use std::process::Command;

pub fn run(root: &Path, args: &[&str]) -> Result<String, String> {
    let output = Command::new("git")
        .arg("-C")
        .arg(root)
        .args(args)
        .output()
        .map_err(|error| format!("git: {error}"))?;
    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).into_owned())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        Err(format!("git {}: {}", args.join(" "), stderr.trim()))
    }
}

pub fn tracked_files(root: &Path) -> Result<Vec<String>, String> {
    run(root, &["ls-files", "-z"]).map(|text| split_nul(&text))
}

pub fn staged_files(root: &Path) -> Result<Vec<String>, String> {
    let args = [
        "diff",
        "--cached",
        "--name-only",
        "--diff-filter=ACMR",
        "-z",
    ];
    run(root, &args).map(|text| split_nul(&text))
}

fn split_nul(text: &str) -> Vec<String> {
    text.split('\0')
        .filter(|name| !name.is_empty())
        .map(str::to_owned)
        .collect()
}
