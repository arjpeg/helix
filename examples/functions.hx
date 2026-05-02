let global = 123;

fn greet(name) {
    print "hello, " + name + "!";

    print global;
    let global = 50;
    print global;
}

greet("arjpeg");
print global;

