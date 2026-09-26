//! cargo-all-features is used extensively in CI. This includes for doctests, but those fail to run
//! on binary-only crates, so we add this empty lib.rs to placate `cargo test --doc`
