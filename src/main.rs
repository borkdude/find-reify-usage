use tree_sitter::{Parser, Language, Node};
use std::path::{Path};
use std::env;
use glob::glob;
use std::fs::metadata;

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

fn print_reify_usage_from_file_path(parser: &mut Parser, path: &Path) {
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
    let language = unsafe { tree_sitter_clojure() };
    let mut parser = Parser::new();
    parser.set_language(language).unwrap();

    for arg in env::args().skip(1) {
        for entry in paths_from_arg(&arg) {
            match entry {
                Ok(path) => {
                    let path = path.as_path();
                    if path.is_file() {
                        print_reify_usage_from_file_path(&mut parser, path);
                    }
                },
                Err(e) => panic!(e)
            }
        }
    }
}
