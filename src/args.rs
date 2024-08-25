use clap::Parser;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
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
        help = "Traverse directories listed in <FILENAMES>\n\
                When specified, all non-image files will be ignored."
    )]
    pub recurse: bool,

    #[arg(
        short,
        long,
        default_value = "5",
        help = "Duplicate detection threshold\n\
                Only show results with hash distance <= <THRESHOLD>\n"
    )]
    pub threshold: u32,

    #[arg(
        short,
        long = "s",
        default_value = "8",
        help = "Image hash size in bytes. Larger hashes are slower, but may allow for more precise \
                comparisons. Duplicate detection threshold may need to be increased alongside this."
    )]
    pub hash_size: u32,

    #[arg(
        short = 'j',
        long = "max-threads",
        help = "Maximum number of worker threads to start\n [default: $(nproc)]"
    )]
    pub max_threads: Option<usize>,
}
