# find-reify-usage

This project finds reified interfaces and protocols in Clojure files provided
via the command line. It is implemented using
[tree-sitter-clojure](https://github.com/sogaiu/tree-sitter-clojure) and Rust.

## Status

This is more a proof of concept than a public facing tool, although it does
solve a real problem for me. I wanted to know the most popular reified
interfaces so I could decide if it made sense supporting `reify` in
[babashka](https://github.com/borkdude/babashka/).

## Build

Execute `script/build`. You will need `npm` and `cargo`.
This will create a `find-reify-usage` binary in `target/release`.

To install the tool on your system:

```
$ cargo install --path .
```

## Usage

```
$ find-reify-usage <path/to/clojure/src>
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
$ find-reify-usage <path/to/clojure/src> | bb -io '(->> *input* frequencies (sort-by second >))'
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
