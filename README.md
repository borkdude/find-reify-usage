# analyze-reify

This project analyzes [Clojure](http://clojure.org/) code for occurrences of
`reify` and lists the reified interfaces and protocols. It is implemented using
[tree-sitter-clojure](https://github.com/sogaiu/tree-sitter-clojure) and
[Rust](https://www.rust-lang.org/).

## Rationale

This is more a proof of concept than a public facing tool, although it does
solve a real problem for me. I wanted to know the most popular reified
interfaces so I could decide if it made sense supporting `reify` in
[babashka](https://github.com/borkdude/babashka/). Also I am curious about both
[Rust](https://www.rust-lang.org/) and
[tree-sitter](https://github.com/tree-sitter/) so this seemed like a nice
oppurtunity to combine the two.

## Results

For simple analyses like this it seems a tree-sitter based approach is
feasible. Performance is better than parsing Clojure code into reified data
structures. For comparison, walking over the ASTs in `clojure/core.clj` takes
only around 50ms whereas in a tools.reader based solution it takes around
200ms. This is by no means a scientific benchmark. More research needed. On my
laptop, analyzing all my Clojure projects takes around 1ms per file on average:

```
Processed 829 files in 731ms. 😎
```

## Build

Clone the repo including the submodule:

```
$ git clone https://github.com/borkdude/analyze-reify --recursive
$ cd analyze-reify
```

Then build the `tree-sitter-clojure` source. Note: this script
requires `npm`.

```
$ script/tree-sitter-clojure
```

Then build with the Rust build tool `cargo`:

```
$ cargo build --release
```

or install the tool on to your system:

```
$ cargo install --path .
```

## Usage

Provide one or multiple paths (files, directories or .jar files) to
`analyze-reify`. It will scan for `.clj` files and analyze them.

```
$ analyze-reify ~/git/clojure
clojure.core.protocols/CollReduce
clojure.core.ArrayManager
clojure.lang.IDeref
...

Processed 160 files in 111ms. 😎
```

To get a sorted frequency list, you can combine this tool with
[babashka](https://github.com/borkdude/babashka/):

```
$ analyze-reify ~/git/clojure | bb -io '(->> *input* frequencies (sort-by second >))'
Processed 160 files in 101ms. 😎
[clojure.core.protocols/CollReduce 4]
[clojure.lang.IDeref 4]
[clojure.core.ArrayManager 2]
[java.util.List 2]
[java.util.Iterator 2]
[Elusive 2]
[java.util.ListIterator 2]
[Object 1]
[clojure.lang.ISeq 1]
[clojure.lang.IReduceInit 1]
[clojure.test_clojure.protocols.examples.ExampleInterface 1]
```

## Thanks

Thanks to [sogaiu](https://github.com/sogaiu/) for taking the time to implement
[tree-sitter-clojure](https://github.com/sogaiu/tree-sitter-clojure).

## License

Copyright © 2020 Michiel Borkent

Distributed under the MIT License. See LICENSE.
