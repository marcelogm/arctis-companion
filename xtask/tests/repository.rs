mod support;

use support::{repo_root, xtask_in};

#[test]
fn repository_passes_check_rules() {
    let outcome = xtask_in(&repo_root(), &["check-rules"]);
    assert!(outcome.succeeded(), "{}", outcome.stderr);
}
