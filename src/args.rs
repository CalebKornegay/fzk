use clap::Parser;

#[derive(Parser, Debug)]
#[command(
    name = "fzk",
    author = "Caleb Kornegay <caleb.kornegay@gmail.com>",
    version = "0.0.1",
    about = "A TUI app to fuzzy find and kill pesky processes",
    long_about = "This tool helps you find pesky processes using fuzzy search.\nAuthor: Caleb Kornegay <caleb.kornegay@gmail.com>"
)]

pub struct Args {
    #[arg(short = 't', long, help="The similarity threshold for matching (default 0.3)")]
    pub threshold: Option<f32>,

    #[arg(short = 'i', long, help="The update interval for processes in seconds (default 3)")]
    pub update_interval: Option<f32>,

    #[arg(short = 'n', long, help="The maximum number of matches from fuzzy matcher (default 20)")]
    pub num_matches: Option<usize>
}
