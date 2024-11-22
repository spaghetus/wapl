# WAPL

* Willow's Awful Programming Language
* Willow's APL
* WAPL: Awful Programming Language
* WAPL Ain't Programming Language

## Example programs

(WAPL doesn't have comments, but we'll be using parentheses for comments in these examples.)

```
$Hello, world!⎋ (Create the name "Hello, world!")
```

```
#-1234⎋ (Create the number -1234)
```

```
#1234---Whatever⎋ (Also creates the number -1234)
```

```
`#1⎋⎋ (Create a quote containing the number 1)
~ (Expand the quote)
```

```
#123⎋ $n⎋. (Assign the number 123 to the name n)
$n⎋~ (Push the name n and expand it)
```

```
#2⎋ #2⎋ (Create two 2's)
`-⎋ (Quote the subtraction operator)
`+⎋ (Quote the addition operator)
#1⎋ (Create the number 1)
? (Ternary; pick + since 1 is greater than 0)
~ (Expand the output of the ternary, resulting in #2⎋#2⎋+ -> #4⎋)
```

```
` (Body of fib)
    $depth⎋. (Take the argument pushed before this quote and name it d)
    `#1⎋⎋ (Quoted 1, falsy branch of the ternary)
    ` (Quoted calls to fib, truthy branch of the ternary)
        $depth⎋~#1⎋-$fib⎋~~ (Fib with depth-1)
        $depth⎋~#2⎋-$fib⎋~~ (Fib with depth-2)
        + (Add the two calls together)
    ⎋
    $depth⎋~? (Branch based on whether depth > 0)
    ~ (Unquote the branch result)
    $depth⎋_ (Dispose of depth)
⎋ $fib⎋. (Name this quote fib)
#10⎋ $f⎋~~ (Call fib with 10)
```

## Informal specification

* `⎋` is typed using the escape key, it returns the interpreter to normal mode from a special mode
* The grave is the quote mode leader, quote mode pushes a syntax quote on the stack and keeps track of how many mode leaders it sees so sending `⎋` will only end quote mode after all inner modes would have ended
* `$` is the name mode leader, it pushes a name on the stack
* `#` is the number mode leader, it does the same for an integer
* `+` and `-` are postfix operators for integer addition and subtraction, self-explanatory
* `?` is the ternary operator, it pops a number, a truthy value, and a falsy value from the stack, and pushes the truthy value back if the number is greater than zero, or the falsy value otherwise
* `.` is the binding operator, it pops a name and a value from the stack and adds it to the name binding table, potentially shadowing an existing value
* `_` is the deletion operator, it pops a value from the name binding table, potentially restoring a shadowed value
* `~` is the expansion operator, it pops a name or quote from the stack and resolves the name or feeds the quote back into the interpreter
