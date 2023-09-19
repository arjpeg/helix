# Helix

> A simple, fast, and lightweight programming language.

---

This project is licensed under the MIT license. See the `LICENSE` file for more information.

---

### What is Helix?

Helix is a simple, fast, and lightweight programming language. It is designed to be easy to learn and use, while still being powerful. It is primarily designed for scripting, but can be used for other purposes as well.

The Helix interpreter is written in the [Rust](https://www.rust-lang.org/) programming language, so it is pretty fast and efficient. It is also designed to be easy to embed in other programs.

---

### Building

On any system with git, you can build Helix by running the following commands:

```bash
git clone https://github.com/arjpeg/helix.git
cd helix
cargo build --release
```

This will create a binary called `helix` in the `target/release` directory. You can run it with `./target/release/helix`, or use `cargo run --release` to run it directly.

---

### Usage

Helix is a scripting language, so it is designed to be run from the command line. You can run a Helix script by passing it as an argument to the Helix interpreter:

```bash
helix my_script.hx
```

or to run it in REPL mode, just run the interpreter without any arguments:

```bash
helix
```

---

### Learning Helix

For now, the best way to learn Helix is to read the [documentation](https://github.com/arjpeg)
