# Filez: Simple File System Abstraction for Rust

Filez is a simple file system abstraction library for Rust, which provides an easy way to perform common file operations such as reading, writing, and listing files.

## Usage

Add the following to your `Cargo.toml` file:

```toml
[dependencies]
filez = { git = "https://github.com/yourusername/filez.git", branch = "main" }
tokio = { version = "1", features = ["full"] }
async-trait = "0.1"
```

## Examples

Reading a file:

```rust
use filez::{live, Files};

#[tokio::main]
async fn main() {
    let files = live("path/to/root".to_string());
    let content = files.read("path/to/file.txt").await.unwrap();
    println!("{}", content);
}
```

Writing to a file:

```rust
use filez::{live, Files};

#[tokio::main]
async fn main() {
    let files = live("path/to/root".to_string());
    files.write("path/to/file.txt", "Hello, world!").await.unwrap();
}
```

Listing files:

```rust
use filez::{live, Files};

#[tokio::main]
async fn main() {
    let files = live("path/to/root".to_string());
    let files = files.list("path/to/*.txt").unwrap();
    println!("{:?}", files);
}
```

## API

The library provides the following traits and structs:

- `Files`: A trait with asynchronous methods for file operations.
- `ReadError`: An error struct for handling errors in the read method.
- `WriteError`: An error struct for handling errors in the write method.
- `ListError`: An error struct for handling errors in the list method.
- `ListErrorKind`: An enum representing the kind of error encountered during file listing.


## License
This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.