use clap::{Parser, Subcommand};
use lazy_static::lazy_static;
use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::process::{exit, Command};

lazy_static! {
    static ref PR_TRAIN_CONTENTS_PATTERN: Regex =
        Regex::new(r"(?s)<pr-train-toc>(.*?)</pr-train-toc>").unwrap();
}

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Create pull requests for all branches in the current stack
    // Push {},
    /// Update all pull requests in the current stack
    Update {
        /// Branches to update
        #[arg(default_value_t = String::from("stack()"))]
        revset: String,
    },
}

fn main() {
    let args = Args::parse();

    if !is_github_cli_installed() {
        eprintln!("Github CLI is not installed. Please install it.");
        exit(1)
    }
    if !is_git_branchless_installed() {
        eprintln!("Git Branchless is not installed. Please install it.");
        exit(1)
    }

    match &args.command {
        // Commands::Push {} => {
        //     panic!("Push not implemented")
        // }
        Commands::Update { revset } => command_update(revset),
    }
}

fn is_github_cli_installed() -> bool {
    let output = Command::new("gh")
        .arg("--version")
        .output()
        .expect("Failed to execute command");

    output.status.success()
}

fn is_git_branchless_installed() -> bool {
    let output = Command::new("git")
        .args(["branchless", "--version"])
        .output()
        .expect("Failed to execute command");

    output.status.success()
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

#[test]
fn parse_git_branchless_branches_works() {
    let branches = parse_git_branchless_branches("a\n\nb\nc\n");
    assert_eq!(branches[0], "a");
    assert_eq!(branches[1], "b");
    assert_eq!(branches[2], "c");
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct GitHubPullRequest {
    number: u32,
    base_ref_name: String,
    head_ref_name: String,
    body: String,
}

fn get_open_prs() -> Vec<GitHubPullRequest> {
    let output = Command::new("gh")
        .args([
            "pr",
            "list",
            "-A",
            "@me",
            "--json",
            "number,baseRefName,headRefName,body",
        ])
        .output()
        .expect("Failed to execute gh pr list command");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(output.status.success());
    serde_json::from_str(&stdout).expect("Failed to parse PR list JSON")
}

fn get_pr_train_branches(branch: &str) -> Vec<String> {
    git_branchless_query_branches(&format!(
        "stack() & (ancestors({branch}) + descendants({branch}))"
    ))
}

fn get_base_branch(branch: &str) -> String {
    let branches =
        git_branchless_query_branches(&format!("ancestors({branch}) - {branch} - green"));
    branches.last().unwrap_or(&"master".to_string()).to_string()
}

fn edit_github_pr(branch: &str, base_branch: &str, body: &str) -> Result<(), String> {
    let output = Command::new("gh")
        .args(["pr", "edit", branch, "-B", base_branch, "-b", body])
        .output()
        .expect("Failed to execute gh pr edit command");

    if output.status.success() {
        Ok(())
    } else {
        Err("Failed to edit GitHub PR".to_string())
    }
}

fn get_pr_train_contents(body: &str) -> Option<String> {
    let result = PR_TRAIN_CONTENTS_PATTERN.find(body);
    result.map(|m| m.as_str().to_string())
}

fn upsert_pr_train_contents_to_body(body: &str, pr_train_contents: &str) -> String {
    if PR_TRAIN_CONTENTS_PATTERN.is_match(body) {
        PR_TRAIN_CONTENTS_PATTERN
            .replace_all(
                body,
                &format!("<pr-train-toc>{pr_train_contents}</pr-train-toc>"),
            )
            .to_string()
    } else {
        format!("{body}\n\n<pr-train-toc>{pr_train_contents}</pr-train-toc>\n")
    }
}

fn get_pr_train_numbers_from_contents(pr_train_contents: &str) -> Vec<u32> {
    let re = Regex::new(r"#(\d+)").unwrap();
    re.captures_iter(pr_train_contents)
        .map(|capture| capture[1].to_string().parse::<u32>().unwrap_or_default())
        .collect()
}

fn is_pr_merged(pr_number: &u32) -> bool {
    let output = Command::new("gh")
        .args(["pr", "view", "--json", "title,state", &pr_number.to_string()])
        .output()
        .expect("Failed to execute gh pr view command");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let pr: Value = serde_json::from_str(&stdout).expect("Failed to parse PR JSON");

    pr["title"].as_str().unwrap_or("").starts_with("[merged]") ||
        pr["state"].as_str().unwrap_or("") == "MERGED"
}

fn command_update(revset: &str) {
    let branches = git_branchless_query_branches(revset);
    let open_prs = get_open_prs();

    let branch_pr_map: HashMap<String, GitHubPullRequest> = open_prs
        .into_iter()
        .map(|pr| (pr.head_ref_name.to_string(), pr))
        .collect();
    let branches_with_pr: Vec<String> = branches
        .into_iter()
        .filter(|branch| branch_pr_map.contains_key(branch))
        .collect();

    for branch in &branches_with_pr {
        let base_branch = get_base_branch(branch);

        let pr = branch_pr_map.get(branch).unwrap();

        let pr_train = get_pr_train_branches(branch);

        let new_pr_train_numbers: Vec<u32> = pr_train
            .into_iter()
            .filter_map(|branch| branch_pr_map.get(&branch).map(|pr| pr.number))
            .collect();

        let old_pr_train_contents = get_pr_train_contents(pr.body.as_str());
        let old_pr_train_numbers = old_pr_train_contents
            .as_ref()
            .map(|contents| get_pr_train_numbers_from_contents(contents))
            .unwrap_or_else(Vec::new);

        let prs_not_in_current: Vec<u32> = old_pr_train_numbers
            .into_iter()
            .filter(|number| !new_pr_train_numbers.contains(number))
            .collect();
        let merged_prs: Vec<u32> = prs_not_in_current
            .into_iter()
            .filter(is_pr_merged)
            .collect();

        let pr_train_numbers: Vec<u32> = merged_prs
            .into_iter()
            .chain(new_pr_train_numbers.into_iter())
            .collect();

        let pr_train_list: Vec<String> = pr_train_numbers
            .into_iter()
            .map(|number| {
                if pr.number == number {
                    format!("- #{number} 📍")
                } else {
                    format!("- #{number}")
                }
            })
            .collect();
        let pr_train_contents = if !pr_train_list.is_empty() {
            format!("\n\n## PR Train\n\n{}\n\n", pr_train_list.join("\n"))
        } else {
            String::new()
        };

        let body = upsert_pr_train_contents_to_body(pr.body.as_str(), &pr_train_contents);

        if body == pr.body.as_str() && base_branch == pr.base_ref_name.as_str() {
            println!("Skipped {branch}");
            continue;
        }

        edit_github_pr(branch, &base_branch, &body).expect("Failed to edit GitHub PR");
        println!("Updated {branch}");
    }
}
