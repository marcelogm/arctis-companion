#![allow(dead_code)]

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use tempfile::TempDir;

pub struct Outcome {
    pub code: Option<i32>,
    pub stdout: String,
    pub stderr: String,
}

impl Outcome {
    pub fn succeeded(&self) -> bool {
        self.code == Some(0)
    }
}

pub struct Fixture {
    dir: TempDir,
}

impl Fixture {
    pub fn init() -> Self {
        let fixture = Self {
            dir: TempDir::new().expect("tempdir"),
        };
        fixture.git(&["init", "--quiet", "--initial-branch=main"]);
        fixture.git(&["config", "user.name", "Fixture"]);
        fixture.git(&["config", "user.email", "fixture@example.invalid"]);
        fixture.git(&["config", "commit.gpgsign", "false"]);
        fixture
    }

    pub fn path(&self) -> &Path {
        self.dir.path()
    }

    pub fn write(&self, relative: &str, content: &str) -> &Self {
        let target = self.path().join(relative);
        let parent = target.parent().expect("relative path has a parent");
        fs::create_dir_all(parent).expect("create parent directories");
        fs::write(&target, content).expect("write fixture file");
        self.git(&["add", relative]);
        self
    }

    pub fn commit(&self, message: &str) -> &Self {
        let args = [
            "-c",
            "core.hooksPath=/dev/null",
            "commit",
            "--quiet",
            "--allow-empty",
            "--message",
            message,
        ];
        self.git(&args);
        self
    }

    pub fn git(&self, args: &[&str]) -> String {
        git(self.path(), args)
    }

    pub fn xtask(&self, args: &[&str]) -> Outcome {
        xtask_in(self.path(), args)
    }
}

pub fn git(dir: &Path, args: &[&str]) -> String {
    let output = Command::new("git")
        .arg("-C")
        .arg(dir)
        .args(args)
        .output()
        .expect("git runs");
    assert!(
        output.status.success(),
        "git {args:?}: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8_lossy(&output.stdout).into_owned()
}

pub fn git_succeeds(dir: &Path, args: &[&str]) -> bool {
    let status = Command::new("git")
        .arg("-C")
        .arg(dir)
        .args(args)
        .status()
        .expect("git runs");
    status.success()
}

pub fn xtask_in(root: &Path, args: &[&str]) -> Outcome {
    let output = Command::new(env!("CARGO_BIN_EXE_xtask"))
        .args(args)
        .arg("--root")
        .arg(root)
        .output()
        .expect("xtask runs");
    Outcome {
        code: output.status.code(),
        stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
        stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
    }
}

pub fn repo_root() -> PathBuf {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    manifest_dir
        .parent()
        .expect("xtask sits inside the workspace")
        .to_path_buf()
}

pub fn assert_rejected(path: &str, content: &str) {
    let fixture = Fixture::init();
    fixture.write(path, content);
    let outcome = fixture.xtask(&["check-rules"]);
    assert!(!outcome.succeeded(), "{path} should be rejected");
    assert!(
        outcome.stderr.contains(&format!("{path}:")),
        "{}",
        outcome.stderr
    );
}

pub fn assert_accepted(path: &str, content: &str) {
    let fixture = Fixture::init();
    fixture.write(path, content);
    let outcome = fixture.xtask(&["check-rules"]);
    assert!(
        outcome.succeeded(),
        "{path} should be accepted: {}",
        outcome.stderr
    );
}
