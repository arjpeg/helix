# Things to implement

## Support for multiple property lookups during assignment.

This would allow for code such as:

```helix
a.b.c = 1
```

or even

```helix
a.b().c = 1
```

this would also allow for single line function calls, such as:

```helix
a.b().c()
```

currently the code wouldn't work, although something like this would:

```helix
let _ = a.b().c()
```

just because of the way the parser works.

---
