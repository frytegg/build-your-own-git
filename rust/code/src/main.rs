use std::env;
use std::fs;
use flate2::read::ZlibDecoder;
use std::io::Read;
use sha1::{Sha1, Digest};
use flate2::write::ZlibEncoder;
use flate2::Compression;
use std::io::Write;

fn main() {
    let args: Vec<String> = env::args().collect();
    match args.len() {
        // Aucun argument → pas de commande 
        1 => { 
            println!("No command provided"); 
        },

        2 => match args[1].as_str() {
            "init" => run_init(),
            "write-tree" => write_tree(),
            _ => println!("unknown command: {}", args[1]),
        },

        4 => match (args[1].as_str(), args[2].as_str()) {
            ("cat-file", "-p") => {
                let hash = &args[3];
                cat_file(hash);
            }
            ("hash-object", "-w") => {
                let path = &args[3];
                hash_object(path);
            }
            ("ls-tree", "--name-only") => {
                let tree_sha = &args[3];
                ls_tree(tree_sha);

            }
            _ => println!("Invalid usage"),
        },

        7 => match (args[1].as_str(), args[3].as_str(), args[5].as_str()) {
            ("commit-tree", "-p", "-m") => {
                let tree_sha = &args[2];
                let parent_sha = &args[4];
                let msg = &args[6];
                commit_tree(tree_sha, parent_sha, msg);
            }
            _ => println!("Invalid usage"),
        },

        _ => println!("Invalid usage"),
    }
}

// INIT HELPERS //

fn run_init() {
    fs::create_dir(".git").unwrap();
    fs::create_dir(".git/objects").unwrap();
    fs::create_dir(".git/refs").unwrap();
    fs::write(".git/HEAD", "ref: refs/heads/main\n").unwrap();
    println!("Initialized git directory");
}

// CAT FILE HELPERS //

fn object_path(hash: &str) -> String {
    let dir = &hash[0..2];
    let file = &hash [2..];
    format!(".git/objects/{}/{}", dir, file)
}

fn read_object(path: &str) -> Vec<u8> {
    fs::read(path).unwrap()
}

fn decompress(data: &[u8]) -> Vec<u8> {
    let cursor = std::io::Cursor::new(data);
    let mut decoder = ZlibDecoder::new(cursor);
    let mut out = Vec::new();
    decoder.read_to_end(&mut out).unwrap();
    out
}

fn extract_blob_content(data: &[u8]) -> String {
    let pos = data.iter().position(|&b| b == 0).unwrap();
    let content_bytes = &data[pos+1..];
    String::from_utf8(content_bytes.to_vec()).unwrap()
}

// HASH OBJECT HELPERS //

fn build_blob_data(path: &str) -> Vec<u8> {
    let content = fs::read(path).unwrap();
    let header = format!("blob {}\0", content.len());
    let mut out = Vec::new();
    out.extend_from_slice(header.as_bytes());
    out.extend_from_slice(&content);
    out
}

fn compute_sha1(data: &[u8]) -> String {
    let mut hasher = Sha1::new();
    hasher.update(data);
    let digest = hasher.finalize();
    let mut hex = String::new();
    for byte in digest {
        hex.push_str(&format!("{:02x}", byte));
    }
    hex
}

fn write_object(hash: &str, data: &[u8]) {
    let dir = format!(".git/objects/{}", &hash[0..2]);
    fs::create_dir_all(&dir).unwrap();
    let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
    encoder.write_all(data).unwrap();
    let compressed = encoder.finish().unwrap();
    fs::write(&object_path(hash), compressed).unwrap();
}

// READ AND WRITE TREE HELPERS //

fn parse_tree_entries(data: &[u8]) -> Vec<String> {
    let mut names = Vec::new();
    let mut pos = data.iter().position(|&b| b == 0).unwrap() + 1;
    while pos < data.len() {
        let end = pos + data[pos..].iter().position(|&b| b == 0).unwrap();
        let entry = String::from_utf8(data[pos..end].to_vec()).unwrap();
        let name = entry.split_once(' ').unwrap().1.to_string();
        names.push(name);
        pos = end + 1 + 20;
    }
    names
}

fn write_tree_for_path(path: &str) -> String {
    let mut entries: Vec<TreeEntry> = Vec::new();

    for entry in fs::read_dir(path).unwrap() {
        let entry = entry.unwrap();
        let file_type = entry.file_type().unwrap();
        let name = entry.file_name().into_string().unwrap();

        if name == ".git" {
            continue;
        }

        if file_type.is_file() {
            let full_path = format!("{}/{}", path, name);
            let data = build_blob_data(&full_path);
            let sha_hex = compute_sha1(&data);
            write_object(&sha_hex, &data);
            let sha_binary = hex_to_bytes(&sha_hex);

            entries.push(TreeEntry {
                mode: "100644".to_string(),
                name,
                sha: sha_binary,
            });
        } else if file_type.is_dir() {
            let sub_path = format!("{}/{}", path, name);
            let sha_hex = write_tree_for_path(&sub_path);
            let sha_binary = hex_to_bytes(&sha_hex);

            entries.push(TreeEntry {
                mode: "40000".to_string(),
                name,
                sha: sha_binary,
            });
        }
    }
    entries.sort_by(|a, b| a.name.cmp(&b.name));
    let mut body = Vec::new();
    for entry in &entries {
        body.extend_from_slice(entry.mode.as_bytes());
        body.push(b' ');
        body.extend_from_slice(entry.name.as_bytes());
        body.push(0);
        body.extend_from_slice(&entry.sha);
    }

    let header = format!("tree {}\0", body.len());
    let mut full = Vec::new();
    full.extend_from_slice(header.as_bytes());
    full.extend_from_slice(&body);

    let sha_hex = compute_sha1(&full);
    write_object(&sha_hex, &full);

    sha_hex
}


struct TreeEntry {
    mode: String,
    name: String,
    sha: Vec<u8>,
}

fn hex_to_bytes(hex: &str) -> Vec<u8> {
    hex.as_bytes().chunks(2).map(|pair| {
        let s = std::str::from_utf8(pair).unwrap();
        u8::from_str_radix(s, 16).unwrap()
    })
    .collect()
}

// MAIN CLI FUNCTIONS //

fn cat_file(hash: &str) {
    let path = object_path(hash);
    let compressed = read_object(&path);
    let decompressed = decompress(&compressed);
    let content = extract_blob_content(&decompressed);
    print!("{}", content);
}

fn hash_object(path: &str) {
    let data = build_blob_data(path);
    let hash = compute_sha1(&data);
    write_object(&hash, &data);
    println!("{}", hash);
}

fn ls_tree(hash: &str) {
    let path = object_path(hash);
    let compressed = read_object(&path);
    let data = decompress(&compressed);

    let entries = parse_tree_entries(&data);

    for name in entries {
        println!("{}", name);
    }
}

fn write_tree() {
    let sha = write_tree_for_path(".");
    println!("{}", sha);
}

fn commit_tree(tree_sha: &str, parent_sha: &str, message: &str) {

    let name = "Codecrafters";
    let email = "dev@example.com";

    let timestamp = chrono::Utc::now().timestamp();
    let timezone = "+0000";

    let author_line = format!("author {} <{}> {} {}", name, email, timestamp, timezone);
    let committer_line = format!("committer {} <{}> {} {}", name, email, timestamp, timezone);

    let mut body = String::new();
    body.push_str(&format!("tree {}\n", tree_sha));
    body.push_str(&format!("parent {}\n", parent_sha));
    body.push_str(&format!("{}\n", author_line));
    body.push_str(&format!("{}\n", committer_line));
    body.push_str("\n");
    body.push_str(message);
    body.push_str("\n");

    let body_bytes = body.as_bytes();
    let header = format!("commit {}\0", body_bytes.len());
    let mut full = Vec::new();
    full.extend_from_slice(header.as_bytes());
    full.extend_from_slice(body_bytes);
    let sha_hex = compute_sha1(&full);
    write_object(&sha_hex, &full);

    println!("{}", sha_hex);
}
