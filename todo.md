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

## Importing modules

Currently, there is no way to import modules, which is a big problem. This is a high priority feature.

Some ideas for syntax:

```helix
let a = import("a.hx) # this imports the file a.hx and assigns it to a

# for importing everything into the current scope
import("a.hx)



```
