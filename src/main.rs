use helix::program::Program;

fn main() {
    let source = "12 \n23.21.3";
    let name = "<stdin>";

    let mut program = Program::new();

    program.register_source(name.to_string(), source.to_string());

    match program.run(0) {
        Ok(_) => (),
        Err(e) => program.pretty_print_error(e),
    }
}
