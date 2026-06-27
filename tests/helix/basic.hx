
fn A() {
    let message = "bruh";

    fn B() {
        print "no closures yet";
        return 42;
    }

    print message;

    B
}

let b = A();
print b();

