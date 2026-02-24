# Build Your Own Git

A minimal git implementation in Rust, built as part of the [CodeCrafters](https://codecrafters.io) "Build Your Own Git" challenge.

## Implemented Commands

| Command | Description |
|---------|-------------|
| `init` | Initialize a new `.git` repository |
| `hash-object -w <path>` | Compute SHA-1 hash and store a blob object |
| `cat-file -p <hash>` | Read and display an object's content |
| `ls-tree --name-only <sha>` | List entries in a tree object |
| `write-tree` | Create tree objects from the working directory |
| `commit-tree <tree> -p <parent> -m <msg>` | Create a commit object |

## Tech

- **Rust** (edition 2021)
- `flate2` for zlib compression
- `sha1` for content-addressed hashing
- `anyhow` / `thiserror` for error handling

## Build & Run

```bash
cd rust/code
cargo build --release
cargo run -- init
cargo run -- hash-object -w file.txt
cargo run -- cat-file -p <sha>
cargo run -- write-tree
cargo run -- commit-tree <tree_sha> -p <parent_sha> -m "message"
```
