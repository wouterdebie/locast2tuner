# From source
{{#include ../locast_account.md}}

The only build requirement `locast2tuner` has is [Rust](https://www.rust-lang.org/) 1.50.0+.

## Installing dependencies
- MacOS: `brew install rust`
- Linux: `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`

## Building
```sh
git clone https://github.com/wouterdebie/locast2tuner
cd locast2tuner
cargo build --release
```

## Installing
You'll end up with a binary in `./target/release/locast2tuner`. You can copy this to the directory of your choosing (`/usr/local/bin` is a good place to start).
