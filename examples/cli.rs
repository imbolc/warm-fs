use clap::Parser;
use warm_fs::Warmer;

#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// Folders to warm up
    #[clap(short, long)]
    dirs: Vec<String>,

    /// Files to warm up
    #[clap(short, long)]
    files: Vec<String>,

    /// Number threads
    #[clap(short, long, default_value_t = 100)]
    threads: usize,

    /// Do not follow links (sometime they can be circular)
    #[clap(long)]
    follow_links: bool,
}

fn main() {
    let args = Args::parse();

    let mut warmer = Warmer::new(args.threads, args.follow_links);
    warmer.add_dirs(&args.dirs);
    warmer.add_files(&args.files);

    let bar = progress_bar(0);

    bar.set_prefix("Size estimation");
    for n in warmer.iter_estimate() {
        bar.inc_length(n);
    }

    bar.set_prefix("Files reading");
    for n in warmer.iter_warm() {
        bar.inc(n);
    }

    bar.abandon()
}

fn progress_bar(total: u64) -> indicatif::ProgressBar {
    let bar = indicatif::ProgressBar::new(total);
    bar.set_style(indicatif::ProgressStyle::default_bar().template(
        "{prefix} {bar} {bytes} of {total_bytes} {percent}% {binary_bytes_per_sec} ~{eta} {msg}",
    ));
    bar.set_draw_rate(25);
    bar
}
