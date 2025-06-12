use clap::Parser;

#[derive(Parser, Debug)]
#[command(
    name = "fzk",
    author = "Caleb Kornegay <caleb.kornegay@gmail.com>",
    version = "0.0.3",
    about = "A TUI app to fuzzy find and kill pesky processes",
    long_about = "This tool helps you find pesky processes using fuzzy search.\nAuthor: Caleb Kornegay <caleb.kornegay@gmail.com>"
)]

pub struct Args {
    #[arg(short = 't', long, help="The similarity threshold for matching between 0.0 and 1.0 (default 0.3)")]
    pub threshold: Option<f32>,

    #[arg(short = 'i', long, help="The update interval for processes in seconds (default 3, minimum 0.75)")]
    pub update_interval: Option<f32>,

    #[arg(short = 'n', long, help="The maximum number of matches from fuzzy matcher (default 25, minimum 1)")]
    pub num_matches: Option<usize>,

    #[arg(short = 'c', long, help="The color of the highlighted process (default lightblue)")]
    pub highlight_color: Option<String>,

    #[arg(short = 'b', long, help="The background color of the entire interface (default 0x12, 0x12, 0x12)")]
    pub background_color: Option<String>,

    #[arg(long, help="Show colors")]
    pub show_colors: bool
}
