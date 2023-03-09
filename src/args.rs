use clap::Parser;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    #[arg(
        short,
        long,
        help = "Traverse directories listed in <FILENAMES>\n\
                When this is set, all non-image files will be ignored."
    )]
    pub recurse: bool,

    #[arg(
        short,
        long,
        help = "Only show results with a similarity score <= <THRESHOLD>"
    )]
    pub threshold: Option<f64>,

    #[arg(help = "Files (and/or directories: -r) to check", required(true))]
    pub filenames: Vec<PathBuf>,

    #[arg(
        short,
        long,
        help = "Scale images on the first comparison instead of at the start\n\
                This may lead to a more unpredictable runtime."
    )]
    pub no_prescale: bool,

    #[arg(
        short = 'j',
        long = "max-threads",
        help = "Maximum number of worker threads to start\n\
                Defaults to $(nproc) * 2"
    )]
    pub max_threads: Option<usize>,
}
