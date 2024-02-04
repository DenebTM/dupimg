use clap::Parser;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    #[arg(
        short,
        long,
        help = "Traverse directories listed in <FILENAMES>\n\
                When specified, all non-image files will be ignored."
    )]
    pub recurse: bool,

    #[arg(
        short,
        long,
        default_value = "0.1",
        help = "Only show results with a similarity score <= <THRESHOLD>\n"
    )]
    pub threshold: Option<f64>,

    #[arg(
        help = "[Right: see --lhs] Files (and/or directories: -r) to check",
        required(true)
    )]
    pub filenames: Vec<PathBuf>,

    #[arg(
        short = 'l',
        long = "lhs",
        help = "Left files (and/or directories: -r) to check (lhs)\n\
                When specified, each image listed under <LEFT_FILENAMES> will be checked\n\
                against each image listed under <FILENAMES>.\n\
                Must be specified for each file or directory individually.\n \
                 e.g. -l <file1> -r -l <dir2>"
    )]
    pub left_filenames: Vec<PathBuf>,

    #[arg(
        short,
        long,
        help = "Scale each image when it is first compared, instead of scaling all images\n\
                before performing the first comparison.\n\
                This may lead to a more unpredictable runtime."
    )]
    pub no_prescale: bool,

    #[arg(
        short = 'j',
        long = "max-threads",
        help = "Maximum number of worker threads to start\n [default: $(nproc)]"
    )]
    pub max_threads: Option<usize>,
}
