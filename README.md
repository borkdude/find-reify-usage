# analyze-reify

This project analyzes [Clojure](http://clojure.org/) code for occurrences of
`reify` and lists the reified interfaces and protocols. It is implemented using
[tree-sitter-clojure](https://github.com/sogaiu/tree-sitter-clojure) and Rust.

## Background

This is more a proof of concept than a public facing tool, although it does
solve a real problem for me. I wanted to know the most popular reified
interfaces so I could decide if it made sense supporting `reify` in
[babashka](https://github.com/borkdude/babashka/). Also I am curious about both
[Rust](https://www.rust-lang.org/) and
[tree-sitter](https://github.com/tree-sitter/) so this seemed like a nice
oppurtunity to combine the two.

## Build

Execute `script/build`. You will need `npm` and `cargo`.
This will create a `analyze-reify` binary in `target/release`.

To install the tool on your system:

```
$ cargo install --path .
```

## Usage

Provide multiple paths (files or directories) to `analyze-reify`. It will
scan for `.clj` files and analyze them.

```
$ analyze-reify <path/to/clojure/src>
clojure.core.protocols/CollReduce
clojure.core.protocols/CollReduce
clojure.lang.IDeref
clojure.lang.IDeref
java.util.Iterator
java.util.ListIterator
clojure.core.ArrayManager
```

To get a sorted frequency list, you can combine this tool with
[babashka](https://github.com/borkdude/babashka/):

```
$ analyze-reify <path/to/clojure/src> | bb -io '(->> *input* frequencies (sort-by second >))'
[Specize 11]
[Function 7]
[clojure.lang.IDeref 6]
[Lock 6]
[impl/Channel 3]
[clojure.core.protocols/CollReduce 2]
[clojure.lang.IReduceInit 2]
[clojure.core.ArrayManager 1]
[ThreadFactory 1]
[Supplier 1]
[WebSocket$Listener 1]
[cljs.test/IAsyncTest 1]
[closure/Inputs 1]
[impl/Executor 1]
[SignalHandler 1]
[clojure.lang.ILookup 1]
[java.util.Iterator 1]
[java.util.ListIterator 1]
```

## License

Copyright © 2020 Michiel Borkent

Distributed under the MIT License. See LICENSE.
