# SOM in Rust

[![Build Status](https://travis-ci.org/jtdowney/som.rs.svg)](https://travis-ci.org/jtdowney/som.rs)

My goal is to fully implement [SOM](http://som-st.github.io/) using [Rust](http://www.rust-lang.org/).

## Requirements

* [rust nightlys](http://www.rust-lang.org/install.html)
* [cargo](https://github.com/rust-lang/cargo)

You can get both of those by running:

```
$ curl https://static.rust-lang.org/rustup.sh | sudo bash
```

## Running

```
$ cargo test
$ cargo build
$ target/som examples/Hello.som
```
