fn make_counter() {
    let count = 0;

    fn inner() {
        count = count + 1;
        count
    }

    inner
}

let count = make_counter();
print count();
print count();
print count();
print count();

