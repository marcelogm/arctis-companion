use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};

use crate::git;
use crate::report::Report;

const HOOKS: [(&str, &str); 2] = [
    ("pre-commit", include_str!("../hooks/pre-commit.sh")),
    ("commit-msg", include_str!("../hooks/commit-msg.sh")),
];
const XTASK_MARK: &str = "cargo xtask";

pub fn run(root: &Path) -> Report {
    install(root).map_or_else(Report::failed, |()| Report::default())
}

fn install(root: &Path) -> Result<(), String> {
    drop_local_hooks_path(root)?;
    refuse_inherited_hooks_path(root)?;
    let hooks = hooks_dir(root)?;
    fs::create_dir_all(&hooks).map_err(|error| format!("{}: {error}", hooks.display()))?;
    HOOKS
        .iter()
        .try_for_each(|(name, script)| write_hook(&hooks.join(name), script))
}

fn drop_local_hooks_path(root: &Path) -> Result<(), String> {
    if git::run(root, &["config", "--local", "--get", "core.hooksPath"]).is_err() {
        return Ok(());
    }
    git::run(root, &["config", "--local", "--unset", "core.hooksPath"]).map(drop)
}

fn refuse_inherited_hooks_path(root: &Path) -> Result<(), String> {
    let Ok(found) = git::run(root, &["config", "--show-scope", "--get", "core.hooksPath"]) else {
        return Ok(());
    };
    Err(format!(
        "core.hooksPath is set in {} config, so git would ignore .git/hooks; unset it there",
        found.split_whitespace().next().unwrap_or("another")
    ))
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
    if is_foreign(path) {
        return Err(format!(
            "{}: an existing hook was not written by xtask; move it away and run install-hooks again",
            path.display()
        ));
    }
    fs::write(path, script).map_err(describe)?;
    fs::set_permissions(path, fs::Permissions::from_mode(0o755)).map_err(describe)
}

fn is_foreign(path: &Path) -> bool {
    path.exists() && !fs::read_to_string(path).is_ok_and(|script| script.contains(XTASK_MARK))
}
