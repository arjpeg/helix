print "closures capture and mutate their environment:";

fn make_counter() {
    let count = 0;
    fn() {
        count = count + 1;
        count
    }
}

let counter = make_counter();
print counter();
print counter();
print counter();

print "independent counters share no state:";

let a = make_counter();
let b = make_counter();
print a();
print a();
print b();
print a();
print b();

print "a toggler that flips between true and false:";

fn make_toggler() {
    let on = false;
    fn() {
        on = !on;
        on
    }
}

let light = make_toggler();
print light();
print light();
print light();
