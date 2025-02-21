use clap::Parser;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    #[arg(
        required_unless_present = "clean",
        num_args = 1..,
        help = "[Right: see --lhs] Files (and/or directories: -r) to check"
    )]
    pub filenames: Vec<PathBuf>,

    #[arg(
        short = 'l',
        long = "lhs",
        help = "'Left-hand-size' files (and/or directories: -r)\n\
                When specified, each image listed under <LHS_FILENAMES> will be checked\n\
                against each image listed under <FILENAMES>.\n\
                Must be specified for each file or directory individually.\n \
                 e.g. -l <file1> -l <file2> <file3> <file4>"
    )]
    pub lhs_filenames: Vec<PathBuf>,

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
                Only show results with hamming distance <= <THRESHOLD>\n"
    )]
    pub threshold: u32,

    #[arg(
        short = 's',
        long = "hash-size",
        default_value = "8",
        help = "Image hash size in bytes. Larger hashes are slower, but may allow for more\n\
                precise comparisons. Detection threshold should be increased alongside this\n\
                parameter.\n"
    )]
    pub hash_size: u32,

    #[arg(
        short = 'j',
        help = "Maximum number of worker threads to start\n [default: $(nproc)]"
    )]
    pub max_threads: Option<usize>,

    #[arg(
        long,
        help = "Alternative cache directory to use",
        default_value = "~/.cache/dupimg"
    )]
    pub cache_dir: String,

    #[arg(
        long,
        help = "Remove ALL(!) .csv files from cache directory prior to running"
    )]
    pub clean: bool,
}
