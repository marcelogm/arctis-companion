mod support;

use support::Fixture;

const TRAILER: &str = "Co-Authored-By: Someone <someone@example.invalid>";

fn range_outcome(messages: &[&str]) -> support::Outcome {
    let fixture = Fixture::init();
    for message in messages {
        fixture.commit(message);
    }
    fixture.xtask(&["check-commits", "--range", "HEAD"])
}

fn message_file_outcome(content: &str) -> support::Outcome {
    let fixture = Fixture::init();
    fixture.write("COMMIT_EDITMSG", content);
    let path = fixture.path().join("COMMIT_EDITMSG");
    fixture.xtask(&[
        "check-commits",
        "--message",
        path.to_str().expect("utf-8 path"),
    ])
}

#[test]
fn conventional_commits_pass() {
    let outcome = range_outcome(&["feat(core): add time", "fix!: handle wake"]);
    assert!(outcome.succeeded(), "{}", outcome.stderr);
}

#[test]
fn co_author_trailer_in_a_commit_fails() {
    let outcome = range_outcome(&["feat: add time", &format!("fix: wake\n\n{TRAILER}")]);
    assert!(!outcome.succeeded());
    assert!(
        outcome.stderr.contains("co-authored-by"),
        "{}",
        outcome.stderr
    );
}

#[test]
fn tool_footer_in_a_commit_fails() {
    assert!(!range_outcome(&["docs: add notes\n\nGenerated with a tool"]).succeeded());
}

#[test]
fn non_conventional_subject_fails() {
    assert!(!range_outcome(&["Add things"]).succeeded());
}

#[test]
fn unknown_type_fails() {
    assert!(!range_outcome(&["feature: add things"]).succeeded());
}

#[test]
fn pr_title_with_a_co_author_fails() {
    let outcome = Fixture::init().xtask(&[
        "check-commits",
        "--pr-title",
        "feat: add Co-Authored-By support",
    ]);
    assert!(!outcome.succeeded());
}

#[test]
fn non_conventional_pr_title_fails() {
    assert!(
        !Fixture::init()
            .xtask(&["check-commits", "--pr-title", "Add things"])
            .succeeded()
    );
}

#[test]
fn pr_body_with_a_co_author_fails() {
    let body = format!("Sets up the workspace.\n\n{TRAILER}");
    assert!(
        !Fixture::init()
            .xtask(&["check-commits", "--pr-body", &body])
            .succeeded()
    );
}

#[test]
fn co_author_trailer_in_any_case_fails() {
    let body = "Sets up the workspace.\n\nco-authored-BY: someone <someone@example.invalid>";
    assert!(
        !Fixture::init()
            .xtask(&["check-commits", "--pr-body", body])
            .succeeded()
    );
}

#[test]
fn clean_pr_title_and_body_pass() {
    let args = [
        "check-commits",
        "--pr-title",
        "build: set up the workspace",
        "--pr-body",
        "Sets up the workspace.\n\nRelated to #1",
    ];
    let outcome = Fixture::init().xtask(&args);
    assert!(outcome.succeeded(), "{}", outcome.stderr);
}

#[test]
fn message_file_skips_git_comment_lines_for_the_subject() {
    let outcome = message_file_outcome("# Please enter the commit message\nfeat: add time\n");
    assert!(outcome.succeeded(), "{}", outcome.stderr);
}

#[test]
fn hash_prefixed_co_author_in_a_message_file_fails() {
    let outcome = message_file_outcome(&format!("feat: add time\n\n#{TRAILER}\n"));
    assert!(!outcome.succeeded());
    assert!(
        outcome.stderr.contains("co-authored-by"),
        "{}",
        outcome.stderr
    );
}

#[test]
fn message_file_ignores_the_verbose_diff() {
    let content =
        format!("fix: wake\n# ------------------------ >8 ------------------------\n+{TRAILER}\n");
    assert!(message_file_outcome(&content).succeeded());
}

#[test]
fn message_file_with_a_co_author_fails() {
    assert!(!message_file_outcome(&format!("feat: add time\n\n{TRAILER}\n")).succeeded());
}

#[test]
fn no_source_fails() {
    assert!(!Fixture::init().xtask(&["check-commits"]).succeeded());
}
