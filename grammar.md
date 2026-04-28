# Helix Grammar

Formal grammar for the Helix language, written in EBNF. Operator precedence is expressed through
the production hierarchy (lowest precedence at the top, highest at the bottom).

## Notation

```
rule       = ...   ; production
|                  ; alternation
{ ... }            ; zero or more repetitions
[ ... ]            ; optional (zero or one)
( ... )            ; grouping
'...'              ; literal terminal
UPPER              ; lexer-level terminal (defined below)
```

---

## Top-level

```ebnf
program = { statement } EOF ;

block   = '{' { statement } [ expr ] '}' ;
```

A `block` is itself an expression. The optional trailing `expr` (with no semicolon) is its *tail*
value — what the block evaluates to.

---

## Statements

```ebnf
statement = print_stmt
          | let_stmt
          | assert_stmt
          | while_stmt
          | expr_stmt
          ;

print_stmt  = 'print' expr ';' ;

let_stmt    = 'let' IDENT '=' expr ';' ;

assert_stmt = 'assert' expr ';' ;

while_stmt  = 'while' expr block [';'] ;

expr_stmt   = expr ( ';' | (* no semicolon, only valid as the final statement in a block/repl *) ) ;
```

> In a source file, every `expr_stmt` **must** end with `;`, unless the expression is a `block`
> (which doesn't require one). In a block or REPL, the last expression may omit `;` to become the
> tail value.

---

## Expressions

Precedence from lowest to highest:

```ebnf
expr       = assignment ;

assignment = IDENT '=' expr        (* right-associative *)
           | or
           ;

or         = and { 'or' and } ;

and        = equality { 'and' equality } ;

equality   = comparison { ( '==' | '!=' ) comparison } ;

comparison = term { ( '<' | '<=' | '>' | '>=' ) term } ;

term       = factor { ( '+' | '-' ) factor } ;

factor     = unary { ( '*' | '/' ) unary } ;

unary      = ( '+' | '-' | '!' ) unary
           | atom
           ;

atom       = INTEGER
           | STRING
           | 'true'
           | 'false'
           | IDENT
           | '(' expr ')'
           | block
           | if_expr
           ;
```

---

## If expression

```ebnf
if_expr = 'if' expr block [ 'else' ( if_expr | block ) ] ;
```

`if` is an expression — it evaluates to the tail of whichever branch runs (or `unit` if there is
no matching branch or the branch has no tail).

---

## Terminals

| Terminal  | Description                                       | Examples              |
|-----------|---------------------------------------------------|-----------------------|
| `INTEGER` | Decimal integer literal                           | `0`, `42`, `-1`       |
| `STRING`  | Double-quoted string literal                      | `"hello"`, `"world"`  |
| `IDENT`   | Identifier — starts with a letter or `_`         | `x`, `my_var`, `foo`  |
| `EOF`     | End of input                                      |                       |

### Keywords (reserved — cannot be used as identifiers)

`and`, `or`, `true`, `false`, `print`, `let`, `assert`, `if`, `else`, `while`, `fn`

### Operators

| Token | Meaning              |
|-------|----------------------|
| `+`   | Add / unary plus     |
| `-`   | Subtract / negate    |
| `*`   | Multiply             |
| `/`   | Divide               |
| `!`   | Logical NOT          |
| `=`   | Assign               |
| `==`  | Equals               |
| `!=`  | Not equals           |
| `<`   | Less than            |
| `<=`  | Less than or equal   |
| `>`   | Greater than         |
| `>=`  | Greater than or equal|

### Grouping symbols

`(` `)` `{` `}`
