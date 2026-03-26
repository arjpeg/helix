let a = 10;
let b = 4;

{
    assert a == 10;

    a = 2;
    let b = 3;

    {
        let a = 100;
    }
}

assert a == 2;
assert b == 4;

