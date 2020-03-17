use tree_sitter::{Parser, Language, Node};
use std::path::{Path, PathBuf};
use std::env;
use glob::glob;
use std::fs::metadata;
use rayon::prelude::*;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::{SystemTime};

extern "C" { fn tree_sitter_clojure() -> Language; }

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

fn print_reify_usage_from_file_path(path: &Path) {
    let language: Language = unsafe { tree_sitter_clojure() };
    let mut parser = Parser::new();
    parser.set_language(language).unwrap();

    let contents = std::fs::read_to_string(path).unwrap();
    let bytes = contents.as_bytes();
    let tree = parser.parse(&bytes, None).unwrap();
    let root_node = tree.root_node();
    print_reify_usage_from_node(root_node, &bytes);
}

fn paths_from_arg(arg: &String) -> glob::Paths {
    let md = metadata(arg).unwrap();
    let is_dir = md.is_dir();
    let pat = if is_dir {
        String::from(arg) + "/**/*.clj"
    } else {
        String::from(arg)
    };
    glob(&pat).unwrap()
}

fn main() {
    let start = SystemTime::now();
    let args: Vec<String> = env::args().skip(1).collect();
    let atomic_counter = AtomicUsize::new(0);
    args.into_par_iter().for_each(|arg| {
        let paths: Vec<Result<PathBuf,_>> = paths_from_arg(&arg).collect();
        paths.into_par_iter().for_each(|entry| {
            match entry {
                Ok(path) => {
                    let path = path.as_path();
                    

                    if path.is_file() {
                        atomic_counter.fetch_add(1, Ordering::Relaxed);
                        print_reify_usage_from_file_path(path);
                    }
                },
                Err(e) => panic!(format!("Unexpected error while analyzing {}", e))
            }
        });
    });
    let since_start = SystemTime::now().duration_since(start)
        .expect("Time went backwards");
    eprintln!("Processed {} files in {}ms. 😎"
              , atomic_counter.load(Ordering::SeqCst)
              , since_start.as_millis());
}
