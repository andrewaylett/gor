# Go in Rust: gor

A _very incomplete_ implementation of Go written as an interpreter in Rust.

This package contains a high-level library interface and the command-line executable.

## Usage

```
$ echo "2+3" > expr.go
$ cargo build
    Finished dev [unoptimized + debuginfo] target(s) in 0.08s
$ target/debug/gor expr.go
Int(5)
```

## Goals

If valid Go code fails to parse, that's a bug.
Or possibly a missing feature, depending on how far I've got so far -- examples of all parseable control structures should be found in `tests/compile`.

Our currently-unimplemented module loader should be able to load modules from anywhere the Go compiler can find them.

Not all invalid Go programs will be rejected by `gor`.
If an invalid Go program is accepted, the (minor) bug is that we accepted the code, rather than any run-time failures that might ensue.

Missing "standard" modules are missing features rather than bugs.

If all required modules and syntax are available, running a Go program in `gor` should provide the same side-effects as compiling it with `go` and running the resulting binary.
If it does not then that's a bug.

## Completeness

The CLI currently evaluates expressions.
It should match Go's precedence rules.

### Types

We currently only really support i64 and bool types.

### Control structures

We have no support for any control structures.

### Tests

Go modules dropped into `tests/compile` will be parsed as part of `cargo test`.
We use `build.rs` to generate Rust test cases.

Eventually™ we'll have compile-failure, compile-only, and executable tests.

## Performance

Not a priority
