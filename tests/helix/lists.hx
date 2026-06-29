# --- construction ---

let empty = [];
assert length(empty) == 0;

let xs = [1, 2, 3, 4, 5];
assert length(xs) == 5;

let mixed = [true, 0, "hello"];
assert length(mixed) == 3;

# --- index get ---

assert xs[0] == 1;
assert xs[4] == 5;
assert xs[2] == 3;

# --- index set ---

xs[0] = 99;
assert xs[0] == 99;

xs[2] = xs[1] + 1;
assert xs[2] == 3;

# --- push ---

let ys = [];
push(ys, 10);
assert length(ys) == 1;
assert ys[0] == 10;

push(ys, 20);
push(ys, 30);
assert length(ys) == 3;
assert ys[1] == 20;
assert ys[2] == 30;

# --- lists are reference types: mutations are shared ---

fn set_first(list, val) {
    list[0] = val;
}

let shared = [1, 2, 3];
set_first(shared, 42);
assert shared[0] == 42;

fn append(list, val) {
    push(list, val);
}

let growing = [];
append(growing, 7);
append(growing, 8);
assert length(growing) == 2;
assert growing[0] == 7;
assert growing[1] == 8;

# --- lists captured by closures retain shared mutations ---

fn make_accumulator() {
    let items = [];
    fn(x) {
        push(items, x);
        length(items)
    }
}

let acc = make_accumulator();
assert acc(10) == 1;
assert acc(20) == 2;
assert acc(30) == 3;

# --- building lists with push ---

fn range(n) {
    let result = [];
    let i = 0;
    while i < n {
        push(result, i);
        i = i + 1;
    }
    result
}

let r = range(5);
assert length(r) == 5;
assert r[0] == 0;
assert r[1] == 1;
assert r[4] == 4;

fn repeat(val, n) {
    let result = [];
    let i = 0;
    while i < n {
        push(result, val);
        i = i + 1;
    }
    result
}

let zeros = repeat(0, 4);
assert length(zeros) == 4;
assert zeros[0] == 0;
assert zeros[3] == 0;

# --- nested lists ---

let matrix = [[1, 2, 3], [4, 5, 6], [7, 8, 9]];
assert length(matrix) == 3;
assert length(matrix[0]) == 3;

assert matrix[0][0] == 1;
assert matrix[0][2] == 3;
assert matrix[1][1] == 5;
assert matrix[2][0] == 7;
assert matrix[2][2] == 9;

# nested index set
matrix[1][1] = 99;
assert matrix[1][1] == 99;

matrix[0][0] = matrix[2][2];
assert matrix[0][0] == 9;

# --- length on strings ---

assert length("hello") == 5;
assert length("") == 0;
assert length("hi") == 2;
