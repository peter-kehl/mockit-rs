## Why?

Because
- Ergonomics: `rust-analyzer` & `cargo check` can check/process only one combination of features at
  a time.
- `cargo` build files get inconsistent when you change a feature flag in `Cargo.toml`. You need to
  `cargo clean` first...
