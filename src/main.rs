use glob::glob;
use rayon::prelude::*;
use std::env;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::{SystemTime};
use tempdir::TempDir;
use tree_sitter::{Parser, Language, Node};
use zip::read::{ZipFile};

extern "C" { fn tree_sitter_clojure() -> Language; }

struct AppCfg<'a> {
    atomic_counter: &'a AtomicUsize,
    tmp_dir: &'a Path,
}

fn node_text<'a>(node: Node<'a>, bytes: &'a [u8]) -> &'a str {
    node.utf8_text(bytes).unwrap()
}

fn print_node(node: Node, bytes: &[u8]) {
    println!("{}",node_text(node, bytes));
}

fn print_reify_usage_from_node(node: Node, bytes: &[u8]) {
    let text = node_text(node, &bytes);
    if text == "reify" {
        let prev_sibling = node.prev_sibling().unwrap();
        let text = node_text(prev_sibling, &bytes);
        if text == "(" {
            print_node(node.next_sibling().unwrap(), &bytes);
        }
    }
    let kind = node.kind();
    if (kind == "list") || (kind == "source_file") {
        let child_count = node.child_count();
        //println!("{:?}", node.kind());
        for child_num in 0..child_count {
            let child = node.child(child_num).unwrap();
            print_reify_usage_from_node(child, &bytes);
        }
    }
}

fn print_reify_usage_from_bytes(bytes: &[u8]) {
    let language: Language = unsafe { tree_sitter_clojure() };
    let mut parser = Parser::new();
    parser.set_language(language).unwrap();
    let tree = parser.parse(&bytes, None).unwrap();
    let root_node = tree.root_node();
    print_reify_usage_from_node(root_node, &bytes);
}

fn print_reify_usage_from_file_path(path: &Path, app_cfg: &AppCfg) {
    //println!("-- print_reify_usage_from_file -- Processing path: {:?}", path);
    let language: Language = unsafe { tree_sitter_clojure() };
    let mut parser = Parser::new();
    parser.set_language(language).unwrap();

    let contents = std::fs::read_to_string(path).unwrap();
    let bytes = contents.as_bytes();
    let tree = parser.parse(&bytes, None).unwrap();
    let root_node = tree.root_node();
    app_cfg.atomic_counter.fetch_add(1, Ordering::Relaxed);
    print_reify_usage_from_node(root_node, &bytes);
}

// references:
// https://github.com/mvdnes/zip-rs/blob/master/examples/extract.rs
// http://siciarz.net/24-days-rust-zip-and-lzma-compression/

fn read_to_string(tmp_dir: &Path, f: &mut ZipFile, s: &mut String) {
    let outpath = f.sanitized_name();
    let outpath = tmp_dir.join(outpath);
    let parent = outpath.parent().unwrap();
    std::fs::create_dir_all(&parent).unwrap();
    let mut outfile = std::fs::File::create(&outpath).unwrap();
    std::io::copy(f, &mut outfile).unwrap();
    let contents = std::fs::read_to_string(outpath.as_path()).unwrap();
    s.push_str(&contents);
}

fn print_reify_usage_from_zipfile_path(path: &Path, app_cfg: &AppCfg) {
    let file = std::fs::File::open(&path).unwrap();
    let archive = zip::ZipArchive::new(file).unwrap();
    let mutex = Mutex::new(archive);
    let range = 0..mutex.lock().unwrap().len();
    range.into_par_iter().for_each(|i| {
        let mut archive = mutex.lock().unwrap();
        let mut file = archive.by_index(i).unwrap();
        if file.is_file() && (file.name()).ends_with(".clj") {
            //let outpath = file.sanitized_name();
            let mut contents = String::new();
            read_to_string(app_cfg.tmp_dir, &mut file, &mut contents);
            app_cfg.atomic_counter.fetch_add(1, Ordering::Relaxed);
            let bytes = &contents.as_bytes();
            print_reify_usage_from_bytes(&bytes);
        }
    });
}

fn print_reify_usage_from_dir(path: &Path, app_cfg: &AppCfg) {
    let mut str = path.to_str().unwrap().to_owned();
    str.push_str("/**/*.*");
    //println!("dir: {:?}", str);
    let paths: Vec<Result<PathBuf,_>> = glob(&str).unwrap().collect();
    //println!("{:?}", paths);
    paths.into_par_iter().for_each(|path| {
        print_reify_dispatch(path.unwrap().as_path(), app_cfg);
    });
}


fn print_reify_dispatch(path: &Path, app_cfg: &AppCfg) {
    if path.is_dir() {
        print_reify_usage_from_dir(path, app_cfg);
        return;
    }
    if path.is_file() {
        match path.extension() {
            None => return,
            Some(ext) => {
                if ext == "jar" {
                    print_reify_usage_from_zipfile_path(path, app_cfg);
                    return;
                }
                if ext == "clj" {
                    print_reify_usage_from_file_path(path, app_cfg);
                }
            }
        }
    }
}

fn main() {
    let start = SystemTime::now();
    let args: Vec<String> = env::args().skip(1).collect();
    let atomic_counter = AtomicUsize::new(0);
    let tmp_dir = TempDir::new("analyze-reify").unwrap();
    let tmp_dir = tmp_dir.path();
    let app_cfg = AppCfg { atomic_counter: &atomic_counter, tmp_dir: &tmp_dir };
    let paths: Vec<&Path> = args.iter().map(|arg| {
        Path::new(arg)
    }).collect();
    paths.into_par_iter().for_each(|path| {
        print_reify_dispatch(&path, &app_cfg);
    });
    let since_start = SystemTime::now().duration_since(start)
        .expect("Time went backwards");
    eprintln!("Processed {} files in {}ms. 😎"
              , atomic_counter.load(Ordering::SeqCst)
              , since_start.as_millis());
    std::fs::remove_dir_all(&tmp_dir).unwrap();
}
