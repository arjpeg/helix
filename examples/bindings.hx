let a = 5;
let penguin = true;

print a;

assert a == 5;

{
    assert penguin;

    let penguin = false;

    assert !penguin;

    let a = { a + 10 };
    let a = a;

    assert a == 15;

    let penguin = {penguin or penguin};

    assert !penguin;

    print a;
}

assert a == 5;

print a;


