fn outer() {
    let x = 10;

    fn middle() {
        fn inner() {
            print x;
        }

        inner
    }

    middle
}

let middle = outer();
let inner = middle();
inner();

