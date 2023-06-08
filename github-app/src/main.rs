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
    let public_repos = repos.iter().filter(|x| !x.private).collect::<Vec<_>>();

    for repo in &public_repos {
        println!(
            "{} ({}) [{}]",
            repo.full_name.yellow(),
            repo.id,
            repo.html_url,
        );
    }

    println!("({} repos)", public_repos.len());

    Ok(())
}
