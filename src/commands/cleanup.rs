use dialoguer::Confirm;
use serde::{Deserialize, Serialize};
use std::process::Command;

pub fn clean_up() {
    let non_descendant_branches = git_branchless_query_branches("draft() - descendants(main())");
    for branch in non_descendant_branches {
        if let Some(pr) = get_pr(&branch) {
            if !is_merged(&pr) {
                continue;
            }

            // Run git sl 'stack(branch)' to show the stack with color output
            let output = Command::new("git")
                .args(["sl", &format!("stack({})", branch)])
                .env("CLICOLOR_FORCE", "1")  // Force color output
                .output()
                .expect("Failed to execute git sl command");

            let stdout = String::from_utf8_lossy(&output.stdout);
            print!("{}", stdout);

            let confirmation = Confirm::new()
                .default(true)
                .wait_for_newline(true)
                .with_prompt(format!("Hide \"{}\"?", branch))
                .interact()
                .unwrap();

            if !confirmation {
                continue;
            }

            let diverging_ancestors = git_branchless_query_diverging_ancestors(&branch);
            let root = diverging_ancestors
                .first()
                .expect("There should be at least 1 ancestor");
            git_branchless_hide_and_delete_descendants(root);
        }
    }
}

fn is_merged(pr: &GitHubPullRequest) -> bool {
    let merged_prefix = "[merged] ";
    return pr.state == "MERGED" || (pr.title.starts_with(merged_prefix) && pr.state == "CLOSED");
}

fn git_branchless_query_branches(query: &str) -> Vec<String> {
    let output = Command::new("git")
        .args(["branchless", "query", "-b", query])
        .output()
        .expect("Failed to execute git branchless query command");

    let stdout = String::from_utf8_lossy(&output.stdout);
    parse_git_branchless_branches(&stdout)
}

fn parse_git_branchless_branches(branchless_stdout: &str) -> Vec<String> {
    branchless_stdout
        .split('\n')
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string())
        .collect()
}

fn git_branchless_query_diverging_ancestors(branch: &str) -> Vec<String> {
    let output = Command::new("git")
        .args([
            "branchless",
            "query",
            "-r",
            &format!("ancestors({}) - ancestors(main())", branch).to_owned(),
        ])
        .output()
        .expect("Failed to execute git branchless query command");
    let stdout = String::from_utf8_lossy(&output.stdout);
    stdout
        .split('\n')
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string())
        .collect()
}

fn git_branchless_hide_and_delete_descendants(commit: &str) {
    let output = Command::new("git")
        .args(["branchless", "hide", "-rD", commit])
        .output()
        .expect("Failed to execute git branchless query command");
    let stdout = String::from_utf8_lossy(&output.stdout);
    println!("{}", stdout);
}

fn get_pr(branch: &str) -> Option<GitHubPullRequest> {
    let output = std::process::Command::new("gh")
        .args(["pr", "view", branch, "--json", "state,title,commits"])
        .output()
        .expect("Failed to execute gh pr view command");
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        if stderr.starts_with("no pull requests found for branch ") {
            return None;
        }
        panic!("{}", stderr);
    }
    let stdout = String::from_utf8_lossy(&output.stdout);
    let pull_request: GitHubPullRequest =
        serde_json::from_str(&stdout).expect("Failed to parse PR JSON");
    Some(pull_request)
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct GitHubCommit {
    oid: String,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct GitHubPullRequest {
    title: String,
    state: String,
    commits: Vec<GitHubCommit>,
}
