let x = 1;
let y = 2;

{
    assert x == 1;
    assert y == 2;
};

{
    let x = 99;
    assert x == 99;
};

assert x == 1;

{
    x = 42;
    assert x == 42;
};

assert x == 42;

let outer = 10;
{
    let outer = 20;
    {
        let outer = 30;
        assert outer == 30;
    };
    assert outer == 20;
};
assert outer == 10;

let a = 1;
{
    a = 2;
    {
        a = 3;
    };
    assert a == 3;
};
assert a == 3;

fn scoped() {
    let local = 100;
    local
}

assert scoped() == 100;

let shared = 5;
fn reads_shared() {
    shared
}
assert reads_shared() == 5;

shared = 6;
assert reads_shared() == 6;
