use helix::program::Program;

fn main() {
    let source = "a + 2 < 5 == true";
    let name = "<stdin>";

    let mut program = Program::new();

    program.register_source(name.to_string(), source.to_string());

    match program.run(0) {
        Ok(_) => (),
        Err(e) => program.pretty_print_error(e),
    }
}
