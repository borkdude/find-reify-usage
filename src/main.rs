use glob::glob;
use rayon::prelude::*;
use std::env;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::{SystemTime};
use tree_sitter::{Parser, Language, Node};
use std::io::Read;

extern "C" { fn tree_sitter_clojure() -> Language; }

struct AppCfg<'a> {
    atomic_counter: &'a AtomicUsize,
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

fn print_reify_usage_from_zipfile_path(path: &Path, app_cfg: &AppCfg) {
    let file = std::fs::File::open(&path).unwrap();
    let mut archive = zip::ZipArchive::new(file).unwrap();
    let range = 0..archive.len();
    let mut strings: Vec<String> = vec![];
    range.for_each(|i| {
        let mut file = archive.by_index(i).unwrap();
        if file.is_file() && (file.name()).ends_with(".clj") {
            let mut contents = String::new();
            file.read_to_string(&mut contents).unwrap();
            strings.push(contents);
        }
    });
    // to verify that we're actually doing work in all the threads
    //let mut active_threads = AtomicUsize::new(0);
    strings.into_par_iter().for_each(|source| {
        //let thread_count = active_threads.fetch_add(1,Ordering::Relaxed);
        //println!("active threads: {}", thread_count);
        app_cfg.atomic_counter.fetch_add(1, Ordering::Relaxed);
        print_reify_usage_from_bytes(source.as_bytes());
        //let thread_count = active_threads.fetch_sub(1,Ordering::Relaxed);
        //println!("active threads: {}", thread_count);
    });
    //println!("active threads: {}", active_threads.get_mut());
    //println!("{}", rayon::current_num_threads());
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
    let app_cfg = AppCfg { atomic_counter: &atomic_counter, };
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
}
