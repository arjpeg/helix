# Sorting algorithms and list utilities demo.
#
# Implements bubble sort, selection sort, and binary search,
# then uses them to sort and query some data.

# --- list utilities ---

fn swap(list, i, j) {
    let tmp = list[i];
    list[i] = list[j];
    list[j] = tmp;
}

fn sum(list) {
    let total = 0;
    let i = 0;
    while i < len(list) {
        total = total + list[i];
        i = i + 1;
    }
    total
}

fn min(list) {
    let m = list[0];
    let i = 1;
    while i < len(list) {
        if list[i] < m { m = list[i]; };
        i = i + 1;
    }
    m
}

fn max(list) {
    let m = list[0];
    let i = 1;
    while i < len(list) {
        if list[i] > m { m = list[i]; };
        i = i + 1;
    }
    m
}

fn contains(list, target) {
    let i = 0;
    while i < len(list) {
        if list[i] == target { return true; };
        i = i + 1;
    }
    false
}

# --- sorting ---

fn bubble_sort(list) {
    let n = len(list);
    let i = 0;
    while i < n - 1 {
        let j = 0;
        while j < n - i - 1 {
            if list[j] > list[j + 1] {
                swap(list, j, j + 1);
            };
            j = j + 1;
        }
        i = i + 1;
    }
}

fn selection_sort(list) {
    let n = len(list);
    let i = 0;
    while i < n - 1 {
        let min_idx = i;
        let j = i + 1;
        while j < n {
            if list[j] < list[min_idx] { min_idx = j; };
            j = j + 1;
        }
        if min_idx != i { swap(list, i, min_idx); };
        i = i + 1;
    }
}

# binary search on a sorted list — returns the index, or -1 if not found
fn binary_search(list, target) {
    let lo = 0;
    let hi = len(list) - 1;
    while lo <= hi {
        let mid = (lo + hi) / 2;
        if list[mid] == target { return mid; };
        if list[mid] < target {
            lo = mid + 1;
        } else {
            hi = mid - 1;
        };
    }
    -1
}

# --- sieve of eratosthenes ---
# returns a list of all prime numbers up to `limit`

fn sieve(limit) {
    let is_prime = [];
    let i = 0;
    while i <= limit {
        push(is_prime, true);
        i = i + 1;
    }

    is_prime[0] = false;
    is_prime[1] = false;

    let p = 2;
    while p * p <= limit {
        if is_prime[p] {
            let multiple = p * p;
            while multiple <= limit {
                is_prime[multiple] = false;
                multiple = multiple + p;
            }
        };
        p = p + 1;
    }

    let primes = [];
    let n = 2;
    while n <= limit {
        if is_prime[n] { push(primes, n); };
        n = n + 1;
    }

    primes
}

# --- demo ---

print "bubble sort:";
let data = [64, 25, 12, 22, 11];
print data;
bubble_sort(data);
print data;

print "selection sort:";
let data2 = [29, 10, 14, 37, 13];
print data2;
selection_sort(data2);
print data2;

print "binary search:";
let haystack = [2, 5, 8, 12, 16, 23, 38, 56, 72, 91];
print binary_search(haystack, 23);
print binary_search(haystack, 72);
print binary_search(haystack, 99);

print "list stats:";
let nums = [3, 7, 1, 9, 4, 6, 2, 8, 5];
print sum(nums);
print min(nums);
print max(nums);

print "primes up to 50:";
let primes = sieve(50);
print primes;
print len(primes);

print "is 37 prime?";
print contains(primes, 37);
print "is 35 prime?";
print contains(primes, 35);
