use helix::program::Program;

fn main() {
    let source = "-2.2 + 3.0";
    let name = "<stdin>";

    let mut program = Program::new();

    let main = program.add_source(name.to_string(), source.to_string());

    match program.run(main) {
        Ok(value) => println!("{value}"),
        Err(e) => program.pretty_print_error(e),
    }
}
