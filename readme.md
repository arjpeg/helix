# Helix

Helix is a bytecode-interpreted programming language designed to be expressive without being verbose. It draws from the ergonomics of modern scripting languages while incorporating algebraic data types and a strong emphasis on clean, readable syntax.

## Status

Under active development. Core lexing, parsing, and a tree-walking interpreter are implemented. A bytecode VM is planned.

## Building

Requires Rust and Cargo.

```
cargo build
```

## Usage

```
cargo run -- file.hx
```

## Example

```
func hello_world(name) {
    print name + ", we aren't in Kansas anymore!";
}

printf("What's your name? ");
let name = input();
hello_world(name);
```

## Goals

- Expressive, minimal syntax
- First-class functions and closures
- Algebraic data types
- Bytecode compilation for portable, efficient execution

## License

MIT
