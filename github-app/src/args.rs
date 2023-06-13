use clap::Parser;

#[derive(Debug, Parser)]
pub struct Args {
    #[clap(
        short = 't',
        long = "token",
        help = "GitHub REST API token",
        env = "GITHUB_CLIENT_GITHUB_TOKEN"
    )]
    pub github_token: String,

    #[clap(
        short = 'p',
        long = "private",
        help = "Private",
        default_value = "false"
    )]
    pub private: bool,

    #[clap(
        short = 'a',
        long = "archived",
        help = "Archived",
        default_value = "false"
    )]
    pub archived: bool,
}
