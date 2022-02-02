[![License](https://img.shields.io/crates/l/warm-fs.svg)](https://choosealicense.com/licenses/mit/)
[![Crates.io](https://img.shields.io/crates/v/warm-fs.svg)](https://crates.io/crates/warm-fs)
[![Docs.rs](https://docs.rs/warm-fs/badge.svg)](https://docs.rs/warm-fs)

# warm-fs

A File system warmer

Cloud providers tent to restore volumes from snapshots in a cold state:

> For volumes that were created from snapshots, the storage blocks must be pulled down from
Amazon S3 and written to the volume before you can access them. This preliminary action takes
time and can cause a significant increase in the latency of I/O operations the first time
each block is accessed ([source][ebs-initialize]).

It has methods to estimates total size of particular folder and then recursively read files
in a thread pool.

It implements `Iterator` giving an access to the warming process intermediate state.
Refer to [cli example] for progress bar implementation.

[ebs-initialize]: https://docs.aws.amazon.com/AWSEC2/latest/UserGuide/ebs-initialize.html
[cli example]: https://github.com/imbolc/warm-fs/blob/main/examples/cli.rs

## Contributing

We appreciate all kinds of contributions, thank you!

### Note on README

The `README.md` file isn't meant to be changed directly. It instead generated from the crate's docs
by the [cargo-readme] command:

* Install the command if you don't have it: `cargo install cargo-readme`
* Change the crate-level docs in `src/lib.rs`, or wrapping text in `README.tpl`
* Apply the changes: `cargo readme > README.md`

If you have [rusty-hook] installed the changes will apply automatically on commit.

## License

This project is licensed under the [MIT license](LICENSE).

[cargo-readme]: https://github.com/livioribeiro/cargo-readme
[rusty-hook]: https://github.com/swellaby/rusty-hook
