mod args;

use crate::args::Args;
use anyhow::Result;
use clap::Parser;
use colored::Colorize;
use github_lib::GitHubClient;

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    let github = GitHubClient::new("https://api.github.com/", &args.github_token)?;

    let repos = github.list_repos().await?;
    for repo in &repos {
        println!(
            "{} ({}) [{}] [private: {}]",
            repo.full_name.yellow(),
            repo.id,
            repo.html_url,
            repo.private
        );
    }

    println!("({} repos)", repos.len());

    Ok(())
}
