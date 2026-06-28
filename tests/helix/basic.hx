
fn outer() {
    let x = 5;

    fn inner() {
        print x;
        x = 10;
    }

    inner();
    print x;
}

outer();

