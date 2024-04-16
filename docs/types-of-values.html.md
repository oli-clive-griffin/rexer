[![](image/logotype.png "Crafting Interpreters")](/)

<div class="contents">

### [Types of Values<span class="small">18</span>](#top)

- [<span class="small">18.1</span> Tagged Unions](#tagged-unions)
- [<span class="small">18.2</span> Lox Values and C
  Values](#lox-values-and-c-values)
- [<span class="small">18.3</span> Dynamically Typed
  Numbers](#dynamically-typed-numbers)
- [<span class="small">18.4</span> Two New Types](#two-new-types)
- 
- [Challenges](#challenges)

<div class="prev-next">

<a href="compiling-expressions.html" class="left"
title="Compiling Expressions">← Previous</a>
[↑ Up](a-bytecode-virtual-machine.html "A Bytecode Virtual Machine")
<a href="strings.html" class="right" title="Strings">Next →</a>

</div>

</div>

[![](image/logotype.png "Crafting Interpreters")](/)
<a href="compiling-expressions.html" class="prev"
title="Compiling Expressions">←</a>
<a href="strings.html" class="next" title="Strings">→</a>

<div class="page">

<div class="nav-wrapper">

[![](image/logotype.png "Crafting Interpreters")](/)

<div class="expandable">

### [Types of Values<span class="small">18</span>](#top)

- [<span class="small">18.1</span> Tagged Unions](#tagged-unions)
- [<span class="small">18.2</span> Lox Values and C
  Values](#lox-values-and-c-values)
- [<span class="small">18.3</span> Dynamically Typed
  Numbers](#dynamically-typed-numbers)
- [<span class="small">18.4</span> Two New Types](#two-new-types)
- 
- [Challenges](#challenges)

<div class="prev-next">

<a href="compiling-expressions.html" class="left"
title="Compiling Expressions">← Previous</a>
[↑ Up](a-bytecode-virtual-machine.html "A Bytecode Virtual Machine")
<a href="strings.html" class="right" title="Strings">Next →</a>

</div>

</div>

<span id="expand-nav">≡</span>

</div>

<div class="number">

18

</div>

# Types of Values

> When you are a Bear of Very Little Brain, and you Think of Things, you
> find sometimes that a Thing which seemed very Thingish inside you is
> quite different when it gets out into the open and has other people
> looking at it.
>
> A. A. Milne, *Winnie-the-Pooh*

The past few chapters were huge, packed full of complex techniques and
pages of code. In this chapter, there’s only one new concept to learn
and a scattering of straightforward code. You’ve earned a respite.

Lox is <span id="unityped">dynamically</span> typed. A single variable
can hold a Boolean, number, or string at different points in time. At
least, that’s the idea. Right now, in clox, all values are numbers. By
the end of the chapter, it will also support Booleans and `nil`. While
those aren’t super interesting, they force us to figure out how our
value representation can dynamically handle different types.

There is a third category next to statically typed and dynamically
typed: **unityped**. In that paradigm, all variables have a single type,
usually a machine register integer. Unityped languages aren’t common
today, but some Forths and BCPL, the language that inspired C, worked
like this.

As of this moment, clox is unityped.

## <a href="#tagged-unions" id="tagged-unions"><span
class="small">18 . 1</span>Tagged Unions</a>

The nice thing about working in C is that we can build our data
structures from the raw bits up. The bad thing is that we *have* to do
that. C doesn’t give you much for free at compile time and even less at
runtime. As far as C is concerned, the universe is an undifferentiated
array of bytes. It’s up to us to decide how many of those bytes to use
and what they mean.

In order to choose a value representation, we need to answer two key
questions:

1.  **How do we represent the type of a value?** If you try to, say,
    multiply a number by `true`, we need to detect that error at runtime
    and report it. In order to do that, we need to be able to tell what
    a value’s type is.

2.  **How do we store the value itself?** We need to not only be able to
    tell that three is a number, but that it’s different from the number
    four. I know, seems obvious, right? But we’re operating at a level
    where it’s good to spell these things out.

Since we’re not just designing this language but building it ourselves,
when answering these two questions we also have to keep in mind the
implementer’s eternal quest: to do it *efficiently*.

Language hackers over the years have come up with a variety of clever
ways to pack the above information into as few bits as possible. For
now, we’ll start with the simplest, classic solution: a **tagged
union**. A value contains two parts: a type “tag”, and a payload for the
actual value. To store the value’s type, we define an enum for each kind
of value the VM supports.

<div class="codehilite">

``` insert-before
#include "common.h"
```

<div class="source-file">

*value.h*

</div>

``` insert
typedef enum {
  VAL_BOOL,
  VAL_NIL, 
  VAL_NUMBER,
} ValueType;
```

``` insert-after
typedef double Value;
```

</div>

<div class="source-file-narrow">

*value.h*

</div>

The cases here cover each kind of value that has *built-in support in
the VM*. When we get to adding classes to the language, each class the
user defines doesn’t need its own entry in this enum. As far as the VM
is concerned, every instance of a class is the same type: “instance”.

In other words, this is the VM’s notion of “type”, not the user’s.

For now, we have only a couple of cases, but this will grow as we add
strings, functions, and classes to clox. In addition to the type, we
also need to store the data for the value<span class="em">—</span>the
`double` for a number, `true` or `false` for a Boolean. We could define
a struct with fields for each possible type.

![A struct with two fields laid next to each other in
memory.](image/types-of-values/struct.png)

But this is a waste of memory. A value can’t simultaneously be both a
number and a Boolean. So at any point in time, only one of those fields
will be used. C lets you optimize this by defining a
<span id="sum">union</span>. A union looks like a struct except that all
of its fields overlap in memory.

If you’re familiar with a language in the ML family, structs and unions
in C roughly mirror the difference between product and sum types,
between tuples and algebraic data types.

![A union with two fields overlapping in
memory.](image/types-of-values/union.png)

The size of a union is the size of its largest field. Since the fields
all reuse the same bits, you have to be very careful when working with
them. If you store data using one field and then access it using
<span id="reinterpret">another</span>, you will reinterpret what the
underlying bits mean.

Using a union to interpret bits as different types is the quintessence
of C. It opens up a number of clever optimizations and lets you slice
and dice each byte of memory in ways that memory-safe languages
disallow. But it is also wildly unsafe and will happily saw your fingers
off if you don’t watch out.

As the name “tagged union” implies, our new value representation
combines these two parts into a single struct.

<div class="codehilite">

``` insert-before
} ValueType;
```

<div class="source-file">

*value.h*  
add after enum *ValueType*  
replace 1 line

</div>

``` insert
typedef struct {
  ValueType type;
  union {
    bool boolean;
    double number;
  } as; 
} Value;
```

``` insert-after

typedef struct {
```

</div>

<div class="source-file-narrow">

*value.h*, add after enum *ValueType*, replace 1 line

</div>

There’s a field for the type tag, and then a second field containing the
union of all of the underlying values. On a 64-bit machine with a
typical C compiler, the layout looks like this:

A smart language hacker gave me the idea to use “as” for the name of the
union field because it reads nicely, almost like a cast, when you pull
the various values out.

![The full value struct, with the type and as fields next to each other
in memory.](image/types-of-values/value.png)

The four-byte type tag comes first, then the union. Most architectures
prefer values be aligned to their size. Since the union field contains
an eight-byte double, the compiler adds four bytes of
<span id="pad">padding</span> after the type field to keep that double
on the nearest eight-byte boundary. That means we’re effectively
spending eight bytes on the type tag, which only needs to represent a
number between zero and three. We could stuff the enum in a smaller
size, but all that would do is increase the padding.

We could move the tag field *after* the union, but that doesn’t help
much either. Whenever we create an array of
Values<span class="em">—</span>which is where most of our memory usage
for Values will be<span class="em">—</span>the C compiler will insert
that same padding *between* each Value to keep the doubles aligned.

So our Values are 16 bytes, which seems a little large. We’ll improve it
[later](optimization.html). In the meantime, they’re still small enough
to store on the C stack and pass around by value. Lox’s semantics allow
that because the only types we support so far are **immutable**. If we
pass a copy of a Value containing the number three to some function, we
don’t need to worry about the caller seeing modifications to the value.
You can’t “modify” three. It’s three forever.

## <a href="#lox-values-and-c-values" id="lox-values-and-c-values"><span
class="small">18 . 2</span>Lox Values and C Values</a>

That’s our new value representation, but we aren’t done. Right now, the
rest of clox assumes Value is an alias for `double`. We have code that
does a straight C cast from one to the other. That code is all broken
now. So sad.

With our new representation, a Value can *contain* a double, but it’s
not *equivalent* to it. There is a mandatory conversion step to get from
one to the other. We need to go through the code and insert those
conversions to get clox working again.

We’ll implement these conversions as a handful of macros, one for each
type and operation. First, to promote a native C value to a clox Value:

<div class="codehilite">

``` insert-before
} Value;
```

<div class="source-file">

*value.h*  
add after struct *Value*

</div>

``` insert

#define BOOL_VAL(value)   ((Value){VAL_BOOL, {.boolean = value}})
#define NIL_VAL           ((Value){VAL_NIL, {.number = 0}})
#define NUMBER_VAL(value) ((Value){VAL_NUMBER, {.number = value}})
```

``` insert-after

typedef struct {
```

</div>

<div class="source-file-narrow">

*value.h*, add after struct *Value*

</div>

Each one of these takes a C value of the appropriate type and produces a
Value that has the correct type tag and contains the underlying value.
This hoists statically typed values up into clox’s dynamically typed
universe. In order to *do* anything with a Value, though, we need to
unpack it and get the C value back out.

<div class="codehilite">

``` insert-before
} Value;
```

<div class="source-file">

*value.h*  
add after struct *Value*

</div>

``` insert

#define AS_BOOL(value)    ((value).as.boolean)
#define AS_NUMBER(value)  ((value).as.number)
```

``` insert-after

#define BOOL_VAL(value)   ((Value){VAL_BOOL, {.boolean = value}})
```

</div>

<div class="source-file-narrow">

*value.h*, add after struct *Value*

</div>

There’s no `AS_NIL` macro because there is only one `nil` value, so a
Value with type `VAL_NIL` doesn’t carry any extra data.

<span id="as-null">These</span> macros go in the opposite direction.
Given a Value of the right type, they unwrap it and return the
corresponding raw C value. The “right type” part is important! These
macros directly access the union fields. If we were to do something
like:

<div class="codehilite">

    Value value = BOOL_VAL(true);
    double number = AS_NUMBER(value);

</div>

Then we may open a smoldering portal to the Shadow Realm. It’s not safe
to use any of the `AS_` macros unless we know the Value contains the
appropriate type. To that end, we define a last few macros to check a
Value’s type.

<div class="codehilite">

``` insert-before
} Value;
```

<div class="source-file">

*value.h*  
add after struct *Value*

</div>

``` insert

#define IS_BOOL(value)    ((value).type == VAL_BOOL)
#define IS_NIL(value)     ((value).type == VAL_NIL)
#define IS_NUMBER(value)  ((value).type == VAL_NUMBER)
```

``` insert-after

#define AS_BOOL(value)    ((value).as.boolean)
```

</div>

<div class="source-file-narrow">

*value.h*, add after struct *Value*

</div>

<span id="universe">These</span> macros return `true` if the Value has
that type. Any time we call one of the `AS_` macros, we need to guard it
behind a call to one of these first. With these eight macros, we can now
safely shuttle data between Lox’s dynamic world and C’s static one.

![The earthly C firmament with the Lox heavens
above.](image/types-of-values/universe.png)

The `_VAL` macros lift a C value into the heavens. The `AS_` macros
bring it back down.

## <a href="#dynamically-typed-numbers"
id="dynamically-typed-numbers"><span
class="small">18 . 3</span>Dynamically Typed Numbers</a>

We’ve got our value representation and the tools to convert to and from
it. All that’s left to get clox running again is to grind through the
code and fix every place where data moves across that boundary. This is
one of those sections of the book that isn’t exactly mind-blowing, but I
promised I’d show you every single line of code, so here we are.

The first values we create are the constants generated when we compile
number literals. After we convert the lexeme to a C double, we simply
wrap it in a Value before storing it in the constant table.

<div class="codehilite">

``` insert-before
  double value = strtod(parser.previous.start, NULL);
```

<div class="source-file">

*compiler.c*  
in *number*()  
replace 1 line

</div>

``` insert
  emitConstant(NUMBER_VAL(value));
```

``` insert-after
}
```

</div>

<div class="source-file-narrow">

*compiler.c*, in *number*(), replace 1 line

</div>

Over in the runtime, we have a function to print values.

<div class="codehilite">

``` insert-before
void printValue(Value value) {
```

<div class="source-file">

*value.c*  
in *printValue*()  
replace 1 line

</div>

``` insert
 printf("%g", AS_NUMBER(value));
```

``` insert-after
}
```

</div>

<div class="source-file-narrow">

*value.c*, in *printValue*(), replace 1 line

</div>

Right before we send the Value to `printf()`, we unwrap it and extract
the double value. We’ll revisit this function shortly to add the other
types, but let’s get our existing code working first.

### <a href="#unary-negation-and-runtime-errors"
id="unary-negation-and-runtime-errors"><span
class="small">18 . 3 . 1</span>Unary negation and runtime errors</a>

The next simplest operation is unary negation. It pops a value off the
stack, negates it, and pushes the result. Now that we have other types
of values, we can’t assume the operand is a number anymore. The user
could just as well do:

<div class="codehilite">

    print -false; // Uh...

</div>

We need to handle that gracefully, which means it’s time for *runtime
errors*. Before performing an operation that requires a certain type, we
need to make sure the Value *is* that type.

For unary negation, the check looks like this:

<div class="codehilite">

``` insert-before
      case OP_DIVIDE:   BINARY_OP(/); break;
```

<div class="source-file">

*vm.c*  
in *run*()  
replace 1 line

</div>

``` insert
      case OP_NEGATE:
        if (!IS_NUMBER(peek(0))) {
          runtimeError("Operand must be a number.");
          return INTERPRET_RUNTIME_ERROR;
        }
        push(NUMBER_VAL(-AS_NUMBER(pop())));
        break;
```

``` insert-after
      case OP_RETURN: {
```

</div>

<div class="source-file-narrow">

*vm.c*, in *run*(), replace 1 line

</div>

First, we check to see if the Value on top of the stack is a number. If
it’s not, we report the runtime error and <span id="halt">stop</span>
the interpreter. Otherwise, we keep going. Only after this validation do
we unwrap the operand, negate it, wrap the result and push it.

Lox’s approach to error-handling is
rather<span class="ellipse"> . . . </span>*spare*. All errors are fatal
and immediately halt the interpreter. There’s no way for user code to
recover from an error. If Lox were a real language, this is one of the
first things I would remedy.

To access the Value, we use a new little function.

<div class="codehilite">

<div class="source-file">

*vm.c*  
add after *pop*()

</div>

    static Value peek(int distance) {
      return vm.stackTop[-1 - distance];
    }

</div>

<div class="source-file-narrow">

*vm.c*, add after *pop*()

</div>

It returns a Value from the stack but doesn’t <span id="peek">pop</span>
it. The `distance` argument is how far down from the top of the stack to
look: zero is the top, one is one slot down, etc.

Why not just pop the operand and then validate it? We could do that. In
later chapters, it will be important to leave operands on the stack to
ensure the garbage collector can find them if a collection is triggered
in the middle of the operation. I do the same thing here mostly out of
habit.

We report the runtime error using a new function that we’ll get a lot of
mileage out of over the remainder of the book.

<div class="codehilite">

<div class="source-file">

*vm.c*  
add after *resetStack*()

</div>

    static void runtimeError(const char* format, ...) {
      va_list args;
      va_start(args, format);
      vfprintf(stderr, format, args);
      va_end(args);
      fputs("\n", stderr);

      size_t instruction = vm.ip - vm.chunk->code - 1;
      int line = vm.chunk->lines[instruction];
      fprintf(stderr, "[line %d] in script\n", line);
      resetStack();
    }

</div>

<div class="source-file-narrow">

*vm.c*, add after *resetStack*()

</div>

You’ve certainly *called* variadic
functions<span class="em">—</span>ones that take a varying number of
arguments<span class="em">—</span>in C before: `printf()` is one. But
you may not have *defined* your own. This book isn’t a C
<span id="tutorial">tutorial</span>, so I’ll skim over it here, but
basically the `...` and `va_list` stuff let us pass an arbitrary number
of arguments to `runtimeError()`. It forwards those on to `vfprintf()`,
which is the flavor of `printf()` that takes an explicit `va_list`.

If you are looking for a C tutorial, I love *[The C Programming
Language](https://www.cs.princeton.edu/~bwk/cbook.html)*, usually called
“K&R” in honor of its authors. It’s not entirely up to date, but the
quality of the writing more than makes up for it.

Callers can pass a format string to `runtimeError()` followed by a
number of arguments, just like they can when calling `printf()`
directly. `runtimeError()` then formats and prints those arguments. We
won’t take advantage of that in this chapter, but later chapters will
produce formatted runtime error messages that contain other data.

After we show the hopefully helpful error message, we tell the user
which <span id="stack">line</span> of their code was being executed when
the error occurred. Since we left the tokens behind in the compiler, we
look up the line in the debug information compiled into the chunk. If
our compiler did its job right, that corresponds to the line of source
code that the bytecode was compiled from.

We look into the chunk’s debug line array using the current bytecode
instruction index *minus one*. That’s because the interpreter advances
past each instruction before executing it. So, at the point that we call
`runtimeError()`, the failed instruction is the previous one.

Just showing the immediate line where the error occurred doesn’t provide
much context. Better would be a full stack trace. But we don’t even have
functions to call yet, so there is no call stack to trace.

In order to use `va_list` and the macros for working with it, we need to
bring in a standard header.

<div class="codehilite">

<div class="source-file">

*vm.c*  
add to top of file

</div>

``` insert
#include <stdarg.h>
```

``` insert-after
#include <stdio.h>
```

</div>

<div class="source-file-narrow">

*vm.c*, add to top of file

</div>

With this, our VM can not only do the right thing when we negate numbers
(like it used to before we broke it), but it also gracefully handles
erroneous attempts to negate other types (which we don’t have yet, but
still).

### <a href="#binary-arithmetic-operators"
id="binary-arithmetic-operators"><span
class="small">18 . 3 . 2</span>Binary arithmetic operators</a>

We have our runtime error machinery in place now, so fixing the binary
operators is easier even though they’re more complex. We support four
binary operators today: `+`, `-`, `*`, and `/`. The only difference
between them is which underlying C operator they use. To minimize
redundant code between the four operators, we wrapped up the commonality
in a big preprocessor macro that takes the operator token as a
parameter.

That macro seemed like overkill a [few chapters
ago](a-virtual-machine.html#binary-operators), but we get the benefit
from it today. It lets us add the necessary type checking and
conversions in one place.

<div class="codehilite">

``` insert-before
#define READ_CONSTANT() (vm.chunk->constants.values[READ_BYTE()])
```

<div class="source-file">

*vm.c*  
in *run*()  
replace 6 lines

</div>

``` insert
#define BINARY_OP(valueType, op) \
    do { \
      if (!IS_NUMBER(peek(0)) || !IS_NUMBER(peek(1))) { \
        runtimeError("Operands must be numbers."); \
        return INTERPRET_RUNTIME_ERROR; \
      } \
      double b = AS_NUMBER(pop()); \
      double a = AS_NUMBER(pop()); \
      push(valueType(a op b)); \
    } while (false)
```

``` insert-after

  for (;;) {
```

</div>

<div class="source-file-narrow">

*vm.c*, in *run*(), replace 6 lines

</div>

Yeah, I realize that’s a monster of a macro. It’s not what I’d normally
consider good C practice, but let’s roll with it. The changes are
similar to what we did for unary negate. First, we check that the two
operands are both numbers. If either isn’t, we report a runtime error
and yank the ejection seat lever.

If the operands are fine, we pop them both and unwrap them. Then we
apply the given operator, wrap the result, and push it back on the
stack. Note that we don’t wrap the result by directly using
`NUMBER_VAL()`. Instead, the wrapper to use is passed in as a macro
<span id="macro">parameter</span>. For our existing arithmetic
operators, the result is a number, so we pass in the `NUMBER_VAL` macro.

Did you know you can pass macros as parameters to macros? Now you do!

<div class="codehilite">

``` insert-before
      }
```

<div class="source-file">

*vm.c*  
in *run*()  
replace 4 lines

</div>

``` insert
      case OP_ADD:      BINARY_OP(NUMBER_VAL, +); break;
      case OP_SUBTRACT: BINARY_OP(NUMBER_VAL, -); break;
      case OP_MULTIPLY: BINARY_OP(NUMBER_VAL, *); break;
      case OP_DIVIDE:   BINARY_OP(NUMBER_VAL, /); break;
```

``` insert-after
      case OP_NEGATE:
```

</div>

<div class="source-file-narrow">

*vm.c*, in *run*(), replace 4 lines

</div>

Soon, I’ll show you why we made the wrapping macro an argument.

## <a href="#two-new-types" id="two-new-types"><span
class="small">18 . 4</span>Two New Types</a>

All of our existing clox code is back in working order. Finally, it’s
time to add some new types. We’ve got a running numeric calculator that
now does a number of pointless paranoid runtime type checks. We can
represent other types internally, but there’s no way for a user’s
program to ever create a Value of one of those types.

Not until now, that is. We’ll start by adding compiler support for the
three new literals: `true`, `false`, and `nil`. They’re all pretty
simple, so we’ll do all three in a single batch.

With number literals, we had to deal with the fact that there are
billions of possible numeric values. We attended to that by storing the
literal’s value in the chunk’s constant table and emitting a bytecode
instruction that simply loaded that constant. We could do the same thing
for the new types. We’d store, say, `true`, in the constant table, and
use an `OP_CONSTANT` to read it out.

But given that there are literally (heh) only three possible values we
need to worry about with these new types, it’s
gratuitous<span class="em">—</span>and
<span id="small">slow!</span><span class="em">—</span>to waste a
two-byte instruction and a constant table entry on them. Instead, we’ll
define three dedicated instructions to push each of these literals on
the stack.

I’m not kidding about dedicated operations for certain constant values
being faster. A bytecode VM spends much of its execution time reading
and decoding instructions. The fewer, simpler instructions you need for
a given piece of behavior, the faster it goes. Short instructions
dedicated to common operations are a classic optimization.

For example, the Java bytecode instruction set has dedicated
instructions for loading 0.0, 1.0, 2.0, and the integer values from -1
through 5. (This ends up being a vestigial optimization given that most
mature JVMs now JIT-compile the bytecode to machine code before
execution anyway.)

<div class="codehilite">

``` insert-before
  OP_CONSTANT,
```

<div class="source-file">

*chunk.h*  
in enum *OpCode*

</div>

``` insert
  OP_NIL,
  OP_TRUE,
  OP_FALSE,
```

``` insert-after
  OP_ADD,
```

</div>

<div class="source-file-narrow">

*chunk.h*, in enum *OpCode*

</div>

Our scanner already treats `true`, `false`, and `nil` as keywords, so we
can skip right to the parser. With our table-based Pratt parser, we just
need to slot parser functions into the rows associated with those
keyword token types. We’ll use the same function in all three slots.
Here:

<div class="codehilite">

``` insert-before
  [TOKEN_ELSE]          = {NULL,     NULL,   PREC_NONE},
```

<div class="source-file">

*compiler.c*  
replace 1 line

</div>

``` insert
  [TOKEN_FALSE]         = {literal,  NULL,   PREC_NONE},
```

``` insert-after
  [TOKEN_FOR]           = {NULL,     NULL,   PREC_NONE},
```

</div>

<div class="source-file-narrow">

*compiler.c*, replace 1 line

</div>

Here:

<div class="codehilite">

``` insert-before
  [TOKEN_THIS]          = {NULL,     NULL,   PREC_NONE},
```

<div class="source-file">

*compiler.c*  
replace 1 line

</div>

``` insert
  [TOKEN_TRUE]          = {literal,  NULL,   PREC_NONE},
```

``` insert-after
  [TOKEN_VAR]           = {NULL,     NULL,   PREC_NONE},
```

</div>

<div class="source-file-narrow">

*compiler.c*, replace 1 line

</div>

And here:

<div class="codehilite">

``` insert-before
  [TOKEN_IF]            = {NULL,     NULL,   PREC_NONE},
```

<div class="source-file">

*compiler.c*  
replace 1 line

</div>

``` insert
  [TOKEN_NIL]           = {literal,  NULL,   PREC_NONE},
```

``` insert-after
  [TOKEN_OR]            = {NULL,     NULL,   PREC_NONE},
```

</div>

<div class="source-file-narrow">

*compiler.c*, replace 1 line

</div>

When the parser encounters `false`, `nil`, or `true`, in prefix
position, it calls this new parser function:

<div class="codehilite">

<div class="source-file">

*compiler.c*  
add after *binary*()

</div>

    static void literal() {
      switch (parser.previous.type) {
        case TOKEN_FALSE: emitByte(OP_FALSE); break;
        case TOKEN_NIL: emitByte(OP_NIL); break;
        case TOKEN_TRUE: emitByte(OP_TRUE); break;
        default: return; // Unreachable.
      }
    }

</div>

<div class="source-file-narrow">

*compiler.c*, add after *binary*()

</div>

Since `parsePrecedence()` has already consumed the keyword token, all we
need to do is output the proper instruction. We
<span id="switch">figure</span> that out based on the type of token we
parsed. Our front end can now compile Boolean and nil literals to
bytecode. Moving down the execution pipeline, we reach the interpreter.

We could have used separate parser functions for each literal and saved
ourselves a switch but that felt needlessly verbose to me. I think it’s
mostly a matter of taste.

<div class="codehilite">

``` insert-before
      case OP_CONSTANT: {
        Value constant = READ_CONSTANT();
        push(constant);
        break;
      }
```

<div class="source-file">

*vm.c*  
in *run*()

</div>

``` insert
      case OP_NIL: push(NIL_VAL); break;
      case OP_TRUE: push(BOOL_VAL(true)); break;
      case OP_FALSE: push(BOOL_VAL(false)); break;
```

``` insert-after
      case OP_ADD:      BINARY_OP(NUMBER_VAL, +); break;
```

</div>

<div class="source-file-narrow">

*vm.c*, in *run*()

</div>

This is pretty self-explanatory. Each instruction summons the
appropriate value and pushes it onto the stack. We shouldn’t forget our
disassembler either.

<div class="codehilite">

``` insert-before
    case OP_CONSTANT:
      return constantInstruction("OP_CONSTANT", chunk, offset);
```

<div class="source-file">

*debug.c*  
in *disassembleInstruction*()

</div>

``` insert
    case OP_NIL:
      return simpleInstruction("OP_NIL", offset);
    case OP_TRUE:
      return simpleInstruction("OP_TRUE", offset);
    case OP_FALSE:
      return simpleInstruction("OP_FALSE", offset);
```

``` insert-after
    case OP_ADD:
```

</div>

<div class="source-file-narrow">

*debug.c*, in *disassembleInstruction*()

</div>

With this in place, we can run this Earth-shattering program:

<div class="codehilite">

    true

</div>

Except that when the interpreter tries to print the result, it blows up.
We need to extend `printValue()` to handle the new types too:

<div class="codehilite">

``` insert-before
void printValue(Value value) {
```

<div class="source-file">

*value.c*  
in *printValue*()  
replace 1 line

</div>

``` insert
  switch (value.type) {
    case VAL_BOOL:
      printf(AS_BOOL(value) ? "true" : "false");
      break;
    case VAL_NIL: printf("nil"); break;
    case VAL_NUMBER: printf("%g", AS_NUMBER(value)); break;
  }
```

``` insert-after
}
```

</div>

<div class="source-file-narrow">

*value.c*, in *printValue*(), replace 1 line

</div>

There we go! Now we have some new types. They just aren’t very useful
yet. Aside from the literals, you can’t really *do* anything with them.
It will be a while before `nil` comes into play, but we can start
putting Booleans to work in the logical operators.

### <a href="#logical-not-and-falsiness"
id="logical-not-and-falsiness"><span
class="small">18 . 4 . 1</span>Logical not and falsiness</a>

The simplest logical operator is our old exclamatory friend unary not.

<div class="codehilite">

    print !true; // "false"

</div>

This new operation gets a new instruction.

<div class="codehilite">

``` insert-before
  OP_DIVIDE,
```

<div class="source-file">

*chunk.h*  
in enum *OpCode*

</div>

``` insert
  OP_NOT,
```

``` insert-after
  OP_NEGATE,
```

</div>

<div class="source-file-narrow">

*chunk.h*, in enum *OpCode*

</div>

We can reuse the `unary()` parser function we wrote for unary negation
to compile a not expression. We just need to slot it into the parsing
table.

<div class="codehilite">

``` insert-before
  [TOKEN_STAR]          = {NULL,     binary, PREC_FACTOR},
```

<div class="source-file">

*compiler.c*  
replace 1 line

</div>

``` insert
  [TOKEN_BANG]          = {unary,    NULL,   PREC_NONE},
```

``` insert-after
  [TOKEN_BANG_EQUAL]    = {NULL,     NULL,   PREC_NONE},
```

</div>

<div class="source-file-narrow">

*compiler.c*, replace 1 line

</div>

Because I knew we were going to do this, the `unary()` function already
has a switch on the token type to figure out which bytecode instruction
to output. We merely add another case.

<div class="codehilite">

``` insert-before
  switch (operatorType) {
```

<div class="source-file">

*compiler.c*  
in *unary*()

</div>

``` insert
    case TOKEN_BANG: emitByte(OP_NOT); break;
```

``` insert-after
    case TOKEN_MINUS: emitByte(OP_NEGATE); break;
    default: return; // Unreachable.
  }
```

</div>

<div class="source-file-narrow">

*compiler.c*, in *unary*()

</div>

That’s it for the front end. Let’s head over to the VM and conjure this
instruction into life.

<div class="codehilite">

``` insert-before
      case OP_DIVIDE:   BINARY_OP(NUMBER_VAL, /); break;
```

<div class="source-file">

*vm.c*  
in *run*()

</div>

``` insert
      case OP_NOT:
        push(BOOL_VAL(isFalsey(pop())));
        break;
```

``` insert-after
      case OP_NEGATE:
```

</div>

<div class="source-file-narrow">

*vm.c*, in *run*()

</div>

Like our previous unary operator, it pops the one operand, performs the
operation, and pushes the result. And, as we did there, we have to worry
about dynamic typing. Taking the logical not of `true` is easy, but
there’s nothing preventing an unruly programmer from writing something
like this:

<div class="codehilite">

    print !nil;

</div>

For unary minus, we made it an error to negate anything that isn’t a
<span id="negate">number</span>. But Lox, like most scripting languages,
is more permissive when it comes to `!` and other contexts where a
Boolean is expected. The rule for how other types are handled is called
“falsiness”, and we implement it here:

Now I can’t help but try to figure out what it would mean to negate
other types of values. `nil` is probably its own negation, sort of like
a weird pseudo-zero. Negating a string could, uh, reverse it?

<div class="codehilite">

<div class="source-file">

*vm.c*  
add after *peek*()

</div>

    static bool isFalsey(Value value) {
      return IS_NIL(value) || (IS_BOOL(value) && !AS_BOOL(value));
    }

</div>

<div class="source-file-narrow">

*vm.c*, add after *peek*()

</div>

Lox follows Ruby in that `nil` and `false` are falsey and every other
value behaves like `true`. We’ve got a new instruction we can generate,
so we also need to be able to *un*generate it in the disassembler.

<div class="codehilite">

``` insert-before
    case OP_DIVIDE:
      return simpleInstruction("OP_DIVIDE", offset);
```

<div class="source-file">

*debug.c*  
in *disassembleInstruction*()

</div>

``` insert
    case OP_NOT:
      return simpleInstruction("OP_NOT", offset);
```

``` insert-after
    case OP_NEGATE:
```

</div>

<div class="source-file-narrow">

*debug.c*, in *disassembleInstruction*()

</div>

### <a href="#equality-and-comparison-operators"
id="equality-and-comparison-operators"><span
class="small">18 . 4 . 2</span>Equality and comparison operators</a>

That wasn’t too bad. Let’s keep the momentum going and knock out the
equality and comparison operators too: `==`, `!=`, `<`, `>`, `<=`, and
`>=`. That covers all of the operators that return Boolean results
except the logical operators `and` and `or`. Since those need to
short-circuit (basically do a little control flow) we aren’t ready for
them yet.

Here are the new instructions for those operators:

<div class="codehilite">

``` insert-before
  OP_FALSE,
```

<div class="source-file">

*chunk.h*  
in enum *OpCode*

</div>

``` insert
  OP_EQUAL,
  OP_GREATER,
  OP_LESS,
```

``` insert-after
  OP_ADD,
```

</div>

<div class="source-file-narrow">

*chunk.h*, in enum *OpCode*

</div>

Wait, only three? What about `!=`, `<=`, and `>=`? We could create
instructions for those too. Honestly, the VM would execute faster if we
did, so we *should* do that if the goal is performance.

But my main goal is to teach you about bytecode compilers. I want you to
start internalizing the idea that the bytecode instructions don’t need
to closely follow the user’s source code. The VM has total freedom to
use whatever instruction set and code sequences it wants as long as they
have the right user-visible behavior.

The expression `a != b` has the same semantics as `!(a == b)`, so the
compiler is free to compile the former as if it were the latter. Instead
of a dedicated `OP_NOT_EQUAL` instruction, it can output an `OP_EQUAL`
followed by an `OP_NOT`. Likewise, `a <= b` is the
<span id="same">same</span> as `!(a > b)` and `a >= b` is `!(a < b)`.
Thus, we only need three new instructions.

*Is* `a <= b` always the same as `!(a > b)`? According to [IEEE
754](https://en.wikipedia.org/wiki/IEEE_754), all comparison operators
return false when an operand is NaN. That means `NaN <= 1` is false and
`NaN > 1` is also false. But our desugaring assumes the latter is always
the negation of the former.

For the book, we won’t get hung up on this, but these kinds of details
will matter in your real language implementations.

Over in the parser, though, we do have six new operators to slot into
the parse table. We use the same `binary()` parser function from before.
Here’s the row for `!=`:

<div class="codehilite">

``` insert-before
  [TOKEN_BANG]          = {unary,    NULL,   PREC_NONE},
```

<div class="source-file">

*compiler.c*  
replace 1 line

</div>

``` insert
  [TOKEN_BANG_EQUAL]    = {NULL,     binary, PREC_EQUALITY},
```

``` insert-after
  [TOKEN_EQUAL]         = {NULL,     NULL,   PREC_NONE},
```

</div>

<div class="source-file-narrow">

*compiler.c*, replace 1 line

</div>

The remaining five operators are a little farther down in the table.

<div class="codehilite">

``` insert-before
  [TOKEN_EQUAL]         = {NULL,     NULL,   PREC_NONE},
```

<div class="source-file">

*compiler.c*  
replace 5 lines

</div>

``` insert
  [TOKEN_EQUAL_EQUAL]   = {NULL,     binary, PREC_EQUALITY},
  [TOKEN_GREATER]       = {NULL,     binary, PREC_COMPARISON},
  [TOKEN_GREATER_EQUAL] = {NULL,     binary, PREC_COMPARISON},
  [TOKEN_LESS]          = {NULL,     binary, PREC_COMPARISON},
  [TOKEN_LESS_EQUAL]    = {NULL,     binary, PREC_COMPARISON},
```

``` insert-after
  [TOKEN_IDENTIFIER]    = {NULL,     NULL,   PREC_NONE},
```

</div>

<div class="source-file-narrow">

*compiler.c*, replace 5 lines

</div>

Inside `binary()` we already have a switch to generate the right
bytecode for each token type. We add cases for the six new operators.

<div class="codehilite">

``` insert-before
  switch (operatorType) {
```

<div class="source-file">

*compiler.c*  
in *binary*()

</div>

``` insert
    case TOKEN_BANG_EQUAL:    emitBytes(OP_EQUAL, OP_NOT); break;
    case TOKEN_EQUAL_EQUAL:   emitByte(OP_EQUAL); break;
    case TOKEN_GREATER:       emitByte(OP_GREATER); break;
    case TOKEN_GREATER_EQUAL: emitBytes(OP_LESS, OP_NOT); break;
    case TOKEN_LESS:          emitByte(OP_LESS); break;
    case TOKEN_LESS_EQUAL:    emitBytes(OP_GREATER, OP_NOT); break;
```

``` insert-after
    case TOKEN_PLUS:          emitByte(OP_ADD); break;
```

</div>

<div class="source-file-narrow">

*compiler.c*, in *binary*()

</div>

The `==`, `<`, and `>` operators output a single instruction. The others
output a pair of instructions, one to evalute the inverse operation, and
then an `OP_NOT` to flip the result. Six operators for the price of
three instructions!

That means over in the VM, our job is simpler. Equality is the most
general operation.

<div class="codehilite">

``` insert-before
      case OP_FALSE: push(BOOL_VAL(false)); break;
```

<div class="source-file">

*vm.c*  
in *run*()

</div>

``` insert
      case OP_EQUAL: {
        Value b = pop();
        Value a = pop();
        push(BOOL_VAL(valuesEqual(a, b)));
        break;
      }
```

``` insert-after
      case OP_ADD:      BINARY_OP(NUMBER_VAL, +); break;
```

</div>

<div class="source-file-narrow">

*vm.c*, in *run*()

</div>

You can evaluate `==` on any pair of objects, even objects of different
types. There’s enough complexity that it makes sense to shunt that logic
over to a separate function. That function always returns a C `bool`, so
we can safely wrap the result in a `BOOL_VAL`. The function relates to
Values, so it lives over in the “value” module.

<div class="codehilite">

``` insert-before
} ValueArray;
```

<div class="source-file">

*value.h*  
add after struct *ValueArray*

</div>

``` insert
bool valuesEqual(Value a, Value b);
```

``` insert-after
void initValueArray(ValueArray* array);
```

</div>

<div class="source-file-narrow">

*value.h*, add after struct *ValueArray*

</div>

And here’s the implementation:

<div class="codehilite">

<div class="source-file">

*value.c*  
add after *printValue*()

</div>

    bool valuesEqual(Value a, Value b) {
      if (a.type != b.type) return false;
      switch (a.type) {
        case VAL_BOOL:   return AS_BOOL(a) == AS_BOOL(b);
        case VAL_NIL:    return true;
        case VAL_NUMBER: return AS_NUMBER(a) == AS_NUMBER(b);
        default:         return false; // Unreachable.
      }
    }

</div>

<div class="source-file-narrow">

*value.c*, add after *printValue*()

</div>

First, we check the types. If the Values have
<span id="equal">different</span> types, they are definitely not equal.
Otherwise, we unwrap the two Values and compare them directly.

Some languages have “implicit conversions” where values of different
types may be considered equal if one can be converted to the other’s
type. For example, the number 0 is equivalent to the string “0” in
JavaScript. This looseness was a large enough source of pain that JS
added a separate “strict equality” operator, `===`.

PHP considers the strings “1” and “01” to be equivalent because both can
be converted to equivalent numbers, though the ultimate reason is
because PHP was designed by a Lovecraftian eldritch god to destroy the
mind.

Most dynamically typed languages that have separate integer and
floating-point number types consider values of different number types
equal if the numeric values are the same (so, say, 1.0 is equal to 1),
though even that seemingly innocuous convenience can bite the unwary.

For each value type, we have a separate case that handles comparing the
value itself. Given how similar the cases are, you might wonder why we
can’t simply `memcmp()` the two Value structs and be done with it. The
problem is that because of padding and different-sized union fields, a
Value contains unused bits. C gives no guarantee about what is in those,
so it’s possible that two equal Values actually differ in memory that
isn’t used.

![The memory respresentations of two equal values that differ in unused
bytes.](image/types-of-values/memcmp.png)

(You wouldn’t believe how much pain I went through before learning this
fact.)

Anyway, as we add more types to clox, this function will grow new cases.
For now, these three are sufficient. The other comparison operators are
easier since they work only on numbers.

<div class="codehilite">

``` insert-before
        push(BOOL_VAL(valuesEqual(a, b)));
        break;
      }
```

<div class="source-file">

*vm.c*  
in *run*()

</div>

``` insert
      case OP_GREATER:  BINARY_OP(BOOL_VAL, >); break;
      case OP_LESS:     BINARY_OP(BOOL_VAL, <); break;
```

``` insert-after
      case OP_ADD:      BINARY_OP(NUMBER_VAL, +); break;
```

</div>

<div class="source-file-narrow">

*vm.c*, in *run*()

</div>

We already extended the `BINARY_OP` macro to handle operators that
return non-numeric types. Now we get to use that. We pass in `BOOL_VAL`
since the result value type is Boolean. Otherwise, it’s no different
from plus or minus.

As always, the coda to today’s aria is disassembling the new
instructions.

<div class="codehilite">

``` insert-before
    case OP_FALSE:
      return simpleInstruction("OP_FALSE", offset);
```

<div class="source-file">

*debug.c*  
in *disassembleInstruction*()

</div>

``` insert
    case OP_EQUAL:
      return simpleInstruction("OP_EQUAL", offset);
    case OP_GREATER:
      return simpleInstruction("OP_GREATER", offset);
    case OP_LESS:
      return simpleInstruction("OP_LESS", offset);
```

``` insert-after
    case OP_ADD:
```

</div>

<div class="source-file-narrow">

*debug.c*, in *disassembleInstruction*()

</div>

With that, our numeric calculator has become something closer to a
general expression evaluator. Fire up clox and type in:

<div class="codehilite">

    !(5 - 4 > 3 * 2 == !nil)

</div>

OK, I’ll admit that’s maybe not the most *useful* expression, but we’re
making progress. We have one missing built-in type with its own literal
form: strings. Those are much more complex because strings can vary in
size. That tiny difference turns out to have implications so large that
we give strings [their very own chapter](strings.html).

<div class="challenges">

## <a href="#challenges" id="challenges">Challenges</a>

1.  We could reduce our binary operators even further than we did here.
    Which other instructions can you eliminate, and how would the
    compiler cope with their absence?

2.  Conversely, we can improve the speed of our bytecode VM by adding
    more specific instructions that correspond to higher-level
    operations. What instructions would you define to speed up the kind
    of user code we added support for in this chapter?

</div>

<a href="strings.html" class="next">Next Chapter: “Strings” →</a>
Handcrafted by Robert Nystrom — <a
href="https://github.com/munificent/craftinginterpreters/blob/master/LICENSE"
target="_blank">© 2015 – 2021</a>

</div>
