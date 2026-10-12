use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};

use crate::git;
use crate::report::Report;

const PRE_COMMIT: &str =
    "#!/bin/sh\n[ -d xtask ] || exit 0\nexec cargo xtask check-rules --staged\n";
const COMMIT_MSG: &str = "#!/bin/sh\nif [ -d xtask ]; then exec cargo xtask check-commits --message \"$1\"; fi\nif [ -f .githooks/commit-msg ]; then exec sh .githooks/commit-msg \"$1\"; fi\n";

pub fn run(root: &Path) -> Report {
    install(root).map_or_else(Report::failed, |()| Report::default())
}

fn install(root: &Path) -> Result<(), String> {
    let hooks = hooks_dir(root)?;
    fs::create_dir_all(&hooks).map_err(|error| format!("{}: {error}", hooks.display()))?;
    write_hook(&hooks.join("pre-commit"), PRE_COMMIT)?;
    write_hook(&hooks.join("commit-msg"), COMMIT_MSG)?;
    unset_hooks_path(root)
}

fn hooks_dir(root: &Path) -> Result<PathBuf, String> {
    let common = git::run(
        root,
        &["rev-parse", "--path-format=absolute", "--git-common-dir"],
    )?;
    Ok(PathBuf::from(common.trim()).join("hooks"))
}

fn write_hook(path: &Path, script: &str) -> Result<(), String> {
    let describe = |error: std::io::Error| format!("{}: {error}", path.display());
    fs::write(path, script).map_err(describe)?;
    fs::set_permissions(path, fs::Permissions::from_mode(0o755)).map_err(describe)
}

fn unset_hooks_path(root: &Path) -> Result<(), String> {
    if git::run(root, &["config", "--get", "core.hooksPath"]).is_err() {
        return Ok(());
    }
    git::run(root, &["config", "--unset", "core.hooksPath"]).map(drop)
}
