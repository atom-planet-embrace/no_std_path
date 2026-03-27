# no_std_path

A `no_std` fork of Rust's [`std::path`](https://doc.rust-lang.org/std/path/index.html) module, providing cross-platform path manipulation without the standard library.

## Overview

`no_std_path` provides `Path`, `PathBuf`, `OsStr`, `OsString`, and the full suite of component/iterator types (`Components`, `Ancestors`, `Prefix`, etc.) adapted from the Rust standard library. It supports parsing, joining, decomposing, and comparing paths on both Unix and Windows.

Most of the code is derived from the [Rust standard library](https://github.com/rust-lang/rust) with modifications to remove `std` dependencies.

## Features

| Feature | Default | Description |
|---------|---------|-------------|
| `std`   | Yes     | Re-exports types directly from `std::path` and `std::ffi`, giving full platform support and filesystem methods. |

When `std` is disabled, lightweight custom implementations are provided using only `core` and `alloc`. This mode is suitable for embedded and `no_std` targets such as `thumbv7m-none-eabi`.

## Usage

With the `std` feature (default):

```toml
[dependencies]
no_std_path = "0.1"
```

For `no_std` environments:

```toml
[dependencies]
no_std_path = { version = "0.1", default-features = false }
```

### Example

```rust
use no_std_path::{Path, PathBuf};

let path = Path::new("/home/user/file.txt");
assert_eq!(path.file_name().unwrap().to_str(), Some("file.txt"));
assert_eq!(path.extension().unwrap().to_str(), Some("txt"));

let mut buf = PathBuf::from("/home/user");
buf.push("docs");
buf.push("readme.md");
assert_eq!(buf.as_path(), Path::new("/home/user/docs/readme.md"));
```

## Differences from `std::path`

In `no_std` mode (with `std` feature disabled):

- **No filesystem methods** — Methods like `exists()`, `canonicalize()`, `metadata()`, `read_link()`, `is_file()`, and `is_dir()` are not available since they require OS interaction.
- **`OsStr` / `OsString` are `[u8]` / `Vec<u8>` wrappers** — The standard library's platform-encoded OS strings are replaced with simple byte-slice newtypes. This means all paths are treated as byte sequences (matching Unix semantics) rather than using platform-specific encoding.

When the `std` feature is enabled, all types are re-exported from `std::path` and `std::ffi` directly, so behavior is identical to the standard library.

## Minimum Supported Rust Version

This crate uses Rust edition 2024.

## License

Licensed under either of

- [Apache License, Version 2.0](LICENSE-APACHE)
- [MIT License](LICENSE-MIT)

at your option.

Most code is derived from the Rust standard library, which is also dual-licensed under MIT and Apache 2.0.
