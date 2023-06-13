mod args;

use crate::args::Args;
use anyhow::Result;
use clap::Parser;
use colored::Colorize;
use github_lib::GitHubClient;

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();

    let args = Args::parse();

    let github = GitHubClient::new("https://api.github.com/", &args.github_token)?;

    let repos = github.get_user_repos().await?;

    println!(
        "Filters: private={}, archived={}",
        args.private, args.archived
    );

    let filtered_repos = repos
        .iter()
        .filter(|x| x.private == args.private && x.archived == args.archived)
        .collect::<Vec<_>>();

    for repo in &filtered_repos {
        println!(
            "{}: {} ({}) [{}]",
            repo.html_url.bright_yellow(),
            repo.full_name.yellow(),
            repo.id,
            repo.html_url
        );
    }

    println!("({} repos)", filtered_repos.len());

    Ok(())
}
