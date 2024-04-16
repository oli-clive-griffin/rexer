[![](image/logotype.png "Crafting Interpreters")](/)

<div class="contents">

### [Local Variables<span class="small">22</span>](#top)

- [<span class="small">22.1</span> Representing Local
  Variables](#representing-local-variables)
- [<span class="small">22.2</span> Block Statements](#block-statements)
- [<span class="small">22.3</span> Declaring Local
  Variables](#declaring-local-variables)
- [<span class="small">22.4</span> Using Locals](#using-locals)
- 
- [Challenges](#challenges)

<div class="prev-next">

<a href="global-variables.html" class="left"
title="Global Variables">← Previous</a>
[↑ Up](a-bytecode-virtual-machine.html "A Bytecode Virtual Machine")
<a href="jumping-back-and-forth.html" class="right"
title="Jumping Back and Forth">Next →</a>

</div>

</div>

[![](image/logotype.png "Crafting Interpreters")](/)
<a href="global-variables.html" class="prev"
title="Global Variables">←</a>
<a href="jumping-back-and-forth.html" class="next"
title="Jumping Back and Forth">→</a>

<div class="page">

<div class="nav-wrapper">

[![](image/logotype.png "Crafting Interpreters")](/)

<div class="expandable">

### [Local Variables<span class="small">22</span>](#top)

- [<span class="small">22.1</span> Representing Local
  Variables](#representing-local-variables)
- [<span class="small">22.2</span> Block Statements](#block-statements)
- [<span class="small">22.3</span> Declaring Local
  Variables](#declaring-local-variables)
- [<span class="small">22.4</span> Using Locals](#using-locals)
- 
- [Challenges](#challenges)

<div class="prev-next">

<a href="global-variables.html" class="left"
title="Global Variables">← Previous</a>
[↑ Up](a-bytecode-virtual-machine.html "A Bytecode Virtual Machine")
<a href="jumping-back-and-forth.html" class="right"
title="Jumping Back and Forth">Next →</a>

</div>

</div>

<span id="expand-nav">≡</span>

</div>

<div class="number">

22

</div>

# Local Variables

> And as imagination bodies forth  
> The forms of things unknown, the poet’s pen  
> Turns them to shapes and gives to airy nothing  
> A local habitation and a name.
>
> William Shakespeare, *A Midsummer Night’s Dream*

The [last chapter](global-variables.html) introduced variables to clox,
but only of the <span id="global">global</span> variety. In this
chapter, we’ll extend that to support blocks, block scope, and local
variables. In jlox, we managed to pack all of that and globals into one
chapter. For clox, that’s two chapters worth of work partially because,
frankly, everything takes more effort in C.

There’s probably some dumb “think globally, act locally” joke here, but
I’m struggling to find it.

But an even more important reason is that our approach to local
variables will be quite different from how we implemented globals.
Global variables are late bound in Lox. “Late” in this context means
“resolved after compile time”. That’s good for keeping the compiler
simple, but not great for performance. Local variables are one of the
most-used <span id="params">parts</span> of a language. If locals are
slow, *everything* is slow. So we want a strategy for local variables
that’s as efficient as possible.

Function parameters are also heavily used. They work like local
variables too, so we’ll use the same implementation technique for them.

Fortunately, lexical scoping is here to help us. As the name implies,
lexical scope means we can resolve a local variable just by looking at
the text of the program<span class="em">—</span>locals are *not* late
bound. Any processing work we do in the compiler is work we *don’t* have
to do at runtime, so our implementation of local variables will lean
heavily on the compiler.

## <a href="#representing-local-variables"
id="representing-local-variables"><span
class="small">22 . 1</span>Representing Local Variables</a>

The nice thing about hacking on a programming language in modern times
is there’s a long lineage of other languages to learn from. So how do C
and Java manage their local variables? Why, on the stack, of course!
They typically use the native stack mechanisms supported by the chip and
OS. That’s a little too low level for us, but inside the virtual world
of clox, we have our own stack we can use.

Right now, we only use it for holding on to
**temporaries**<span class="em">—</span>short-lived blobs of data that
we need to remember while computing an expression. As long as we don’t
get in the way of those, we can stuff our local variables onto the stack
too. This is great for performance. Allocating space for a new local
requires only incrementing the `stackTop` pointer, and freeing is
likewise a decrement. Accessing a variable from a known stack slot is an
indexed array lookup.

We do need to be careful, though. The VM expects the stack to behave
like, well, a stack. We have to be OK with allocating new locals only on
the top of the stack, and we have to accept that we can discard a local
only when nothing is above it on the stack. Also, we need to make sure
temporaries don’t interfere.

Conveniently, the design of Lox is in <span id="harmony">harmony</span>
with these constraints. New locals are always created by declaration
statements. Statements don’t nest inside expressions, so there are never
any temporaries on the stack when a statement begins executing. Blocks
are strictly nested. When a block ends, it always takes the innermost,
most recently declared locals with it. Since those are also the locals
that came into scope last, they should be on top of the stack where we
need them.

This alignment obviously isn’t coincidental. I designed Lox to be
amenable to single-pass compilation to stack-based bytecode. But I
didn’t have to tweak the language too much to fit in those restrictions.
Most of its design should feel pretty natural.

This is in large part because the history of languages is deeply tied to
single-pass compilation and<span class="em">—</span>to a lesser
degree<span class="em">—</span>stack-based architectures. Lox’s block
scoping follows a tradition stretching back to BCPL. As programmers, our
intuition of what’s “normal” in a language is informed even today by the
hardware limitations of yesteryear.

Step through this example program and watch how the local variables come
in and go out of scope:

![A series of local variables come into and out of scope in a stack-like
fashion.](image/local-variables/scopes.png)

See how they fit a stack perfectly? It seems that the stack will work
for storing locals at runtime. But we can go further than that. Not only
do we know *that* they will be on the stack, but we can even pin down
precisely *where* they will be on the stack. Since the compiler knows
exactly which local variables are in scope at any point in time, it can
effectively simulate the stack during compilation and note
<span id="fn">where</span> in the stack each variable lives.

We’ll take advantage of this by using these stack offsets as operands
for the bytecode instructions that read and store local variables. This
makes working with locals deliciously fast<span class="em">—</span>as
simple as indexing into an array.

In this chapter, locals start at the bottom of the VM’s stack array and
are indexed from there. When we add
[functions](calls-and-functions.html), that scheme gets a little more
complex. Each function needs its own region of the stack for its
parameters and local variables. But, as we’ll see, that doesn’t add as
much complexity as you might expect.

There’s a lot of state we need to track in the compiler to make this
whole thing go, so let’s get started there. In jlox, we used a linked
chain of “environment” HashMaps to track which local variables were
currently in scope. That’s sort of the classic, schoolbook way of
representing lexical scope. For clox, as usual, we’re going a little
closer to the metal. All of the state lives in a new struct.

<div class="codehilite">

``` insert-before
} ParseRule;
```

<div class="source-file">

*compiler.c*  
add after struct *ParseRule*

</div>

``` insert

typedef struct {
  Local locals[UINT8_COUNT];
  int localCount;
  int scopeDepth;
} Compiler;
```

``` insert-after

Parser parser;
```

</div>

<div class="source-file-narrow">

*compiler.c*, add after struct *ParseRule*

</div>

We have a simple, flat array of all locals that are in scope during each
point in the compilation process. They are
<span id="order">ordered</span> in the array in the order that their
declarations appear in the code. Since the instruction operand we’ll use
to encode a local is a single byte, our VM has a hard limit on the
number of locals that can be in scope at once. That means we can also
give the locals array a fixed size.

We’re writing a single-pass compiler, so it’s not like we have *too*
many other options for how to order them in the array.

<div class="codehilite">

``` insert-before
#define DEBUG_TRACE_EXECUTION
```

<div class="source-file">

*common.h*

</div>

``` insert

#define UINT8_COUNT (UINT8_MAX + 1)
```

``` insert-after

#endif
```

</div>

<div class="source-file-narrow">

*common.h*

</div>

Back in the Compiler struct, the `localCount` field tracks how many
locals are in scope<span class="em">—</span>how many of those array
slots are in use. We also track the “scope depth”. This is the number of
blocks surrounding the current bit of code we’re compiling.

Our Java interpreter used a chain of maps to keep each block’s variables
separate from other blocks’. This time, we’ll simply number variables
with the level of nesting where they appear. Zero is the global scope,
one is the first top-level block, two is inside that, you get the idea.
We use this to track which block each local belongs to so that we know
which locals to discard when a block ends.

Each local in the array is one of these:

<div class="codehilite">

``` insert-before
} ParseRule;
```

<div class="source-file">

*compiler.c*  
add after struct *ParseRule*

</div>

``` insert

typedef struct {
  Token name;
  int depth;
} Local;
```

``` insert-after

typedef struct {
```

</div>

<div class="source-file-narrow">

*compiler.c*, add after struct *ParseRule*

</div>

We store the name of the variable. When we’re resolving an identifier,
we compare the identifier’s lexeme with each local’s name to find a
match. It’s pretty hard to resolve a variable if you don’t know its
name. The `depth` field records the scope depth of the block where the
local variable was declared. That’s all the state we need for now.

This is a very different representation from what we had in jlox, but it
still lets us answer all of the same questions our compiler needs to ask
of the lexical environment. The next step is figuring out how the
compiler *gets* at this state. If we were
<span id="thread">principled</span> engineers, we’d give each function
in the front end a parameter that accepts a pointer to a Compiler. We’d
create a Compiler at the beginning and carefully thread it through each
function call<span class="ellipse"> . . . </span>but that would mean a
lot of boring changes to the code we already wrote, so here’s a global
variable instead:

In particular, if we ever want to use our compiler in a multi-threaded
application, possibly with multiple compilers running in parallel, then
using a global variable is a *bad* idea.

<div class="codehilite">

``` insert-before
Parser parser;
```

<div class="source-file">

*compiler.c*  
add after variable *parser*

</div>

``` insert
Compiler* current = NULL;
```

``` insert-after
Chunk* compilingChunk;
```

</div>

<div class="source-file-narrow">

*compiler.c*, add after variable *parser*

</div>

Here’s a little function to initialize the compiler:

<div class="codehilite">

<div class="source-file">

*compiler.c*  
add after *emitConstant*()

</div>

    static void initCompiler(Compiler* compiler) {
      compiler->localCount = 0;
      compiler->scopeDepth = 0;
      current = compiler;
    }

</div>

<div class="source-file-narrow">

*compiler.c*, add after *emitConstant*()

</div>

When we first start up the VM, we call it to get everything into a clean
state.

<div class="codehilite">

``` insert-before
  initScanner(source);
```

<div class="source-file">

*compiler.c*  
in *compile*()

</div>

``` insert
  Compiler compiler;
  initCompiler(&compiler);
```

``` insert-after
  compilingChunk = chunk;
```

</div>

<div class="source-file-narrow">

*compiler.c*, in *compile*()

</div>

Our compiler has the data it needs, but not the operations on that data.
There’s no way to create and destroy scopes, or add and resolve
variables. We’ll add those as we need them. First, let’s start building
some language features.

## <a href="#block-statements" id="block-statements"><span
class="small">22 . 2</span>Block Statements</a>

Before we can have any local variables, we need some local scopes. These
come from two things: function bodies and
<span id="block">blocks</span>. Functions are a big chunk of work that
we’ll tackle in [a later chapter](calls-and-functions.html), so for now
we’re only going to do blocks. As usual, we start with the syntax. The
new grammar we’ll introduce is:

<div class="codehilite">

    statement      → exprStmt
                   | printStmt
                   | block ;

    block          → "{" declaration* "}" ;

</div>

When you think about it, “block” is a weird name. Used metaphorically,
“block” usually means a small indivisible unit, but for some reason, the
Algol 60 committee decided to use it to refer to a *compound*
structure<span class="em">—</span>a series of statements. It could be
worse, I suppose. Algol 58 called `begin` and `end` “statement
parentheses”.

<img src="image/local-variables/block.png" class="above"
alt="A cinder block." />

Blocks are a kind of statement, so the rule for them goes in the
`statement` production. The corresponding code to compile one looks like
this:

<div class="codehilite">

``` insert-before
  if (match(TOKEN_PRINT)) {
    printStatement();
```

<div class="source-file">

*compiler.c*  
in *statement*()

</div>

``` insert
  } else if (match(TOKEN_LEFT_BRACE)) {
    beginScope();
    block();
    endScope();
```

``` insert-after
  } else {
```

</div>

<div class="source-file-narrow">

*compiler.c*, in *statement*()

</div>

After <span id="helper">parsing</span> the initial curly brace, we use
this helper function to compile the rest of the block:

This function will come in handy later for compiling function bodies.

<div class="codehilite">

<div class="source-file">

*compiler.c*  
add after *expression*()

</div>

    static void block() {
      while (!check(TOKEN_RIGHT_BRACE) && !check(TOKEN_EOF)) {
        declaration();
      }

      consume(TOKEN_RIGHT_BRACE, "Expect '}' after block.");
    }

</div>

<div class="source-file-narrow">

*compiler.c*, add after *expression*()

</div>

It keeps parsing declarations and statements until it hits the closing
brace. As we do with any loop in the parser, we also check for the end
of the token stream. This way, if there’s a malformed program with a
missing closing curly, the compiler doesn’t get stuck in a loop.

Executing a block simply means executing the statements it contains, one
after the other, so there isn’t much to compiling them. The semantically
interesting thing blocks do is create scopes. Before we compile the body
of a block, we call this function to enter a new local scope:

<div class="codehilite">

<div class="source-file">

*compiler.c*  
add after *endCompiler*()

</div>

    static void beginScope() {
      current->scopeDepth++;
    }

</div>

<div class="source-file-narrow">

*compiler.c*, add after *endCompiler*()

</div>

In order to “create” a scope, all we do is increment the current depth.
This is certainly much faster than jlox, which allocated an entire new
HashMap for each one. Given `beginScope()`, you can probably guess what
`endScope()` does.

<div class="codehilite">

<div class="source-file">

*compiler.c*  
add after *beginScope*()

</div>

    static void endScope() {
      current->scopeDepth--;
    }

</div>

<div class="source-file-narrow">

*compiler.c*, add after *beginScope*()

</div>

That’s it for blocks and scopes<span class="em">—</span>more or
less<span class="em">—</span>so we’re ready to stuff some variables into
them.

## <a href="#declaring-local-variables"
id="declaring-local-variables"><span
class="small">22 . 3</span>Declaring Local Variables</a>

Usually we start with parsing here, but our compiler already supports
parsing and compiling variable declarations. We’ve got `var` statements,
identifier expressions and assignment in there now. It’s just that the
compiler assumes all variables are global. So we don’t need any new
parsing support, we just need to hook up the new scoping semantics to
the existing code.

![The code flow within
varDeclaration().](image/local-variables/declaration.png)

Variable declaration parsing begins in `varDeclaration()` and relies on
a couple of other functions. First, `parseVariable()` consumes the
identifier token for the variable name, adds its lexeme to the chunk’s
constant table as a string, and then returns the constant table index
where it was added. Then, after `varDeclaration()` compiles the
initializer, it calls `defineVariable()` to emit the bytecode for
storing the variable’s value in the global variable hash table.

Both of those helpers need a few changes to support local variables. In
`parseVariable()`, we add:

<div class="codehilite">

``` insert-before
  consume(TOKEN_IDENTIFIER, errorMessage);
```

<div class="source-file">

*compiler.c*  
in *parseVariable*()

</div>

``` insert

  declareVariable();
  if (current->scopeDepth > 0) return 0;
```

``` insert-after
  return identifierConstant(&parser.previous);
```

</div>

<div class="source-file-narrow">

*compiler.c*, in *parseVariable*()

</div>

First, we “declare” the variable. I’ll get to what that means in a
second. After that, we exit the function if we’re in a local scope. At
runtime, locals aren’t looked up by name. There’s no need to stuff the
variable’s name into the constant table, so if the declaration is inside
a local scope, we return a dummy table index instead.

Over in `defineVariable()`, we need to emit the code to store a local
variable if we’re in a local scope. It looks like this:

<div class="codehilite">

``` insert-before
static void defineVariable(uint8_t global) {
```

<div class="source-file">

*compiler.c*  
in *defineVariable*()

</div>

``` insert
  if (current->scopeDepth > 0) {
    return;
  }
```

``` insert-after
  emitBytes(OP_DEFINE_GLOBAL, global);
```

</div>

<div class="source-file-narrow">

*compiler.c*, in *defineVariable*()

</div>

Wait, what? Yup. That’s it. There is no code to create a local variable
at runtime. Think about what state the VM is in. It has already executed
the code for the variable’s initializer (or the implicit `nil` if the
user omitted an initializer), and that value is sitting right on top of
the stack as the only remaining temporary. We also know that new locals
are allocated at the top of the
stack<span class="ellipse"> . . . </span>right where that value already
is. Thus, there’s nothing to do. The temporary simply *becomes* the
local variable. It doesn’t get much more efficient than that.

<span id="locals"></span>

![Walking through the bytecode execution showing that each initializer's
result ends up in the local's
slot.](image/local-variables/local-slots.png)

The code on the left compiles to the sequence of instructions on the
right.

OK, so what’s “declaring” about? Here’s what that does:

<div class="codehilite">

<div class="source-file">

*compiler.c*  
add after *identifierConstant*()

</div>

    static void declareVariable() {
      if (current->scopeDepth == 0) return;

      Token* name = &parser.previous;
      addLocal(*name);
    }

</div>

<div class="source-file-narrow">

*compiler.c*, add after *identifierConstant*()

</div>

This is the point where the compiler records the existence of the
variable. We only do this for locals, so if we’re in the top-level
global scope, we just bail out. Because global variables are late bound,
the compiler doesn’t keep track of which declarations for them it has
seen.

But for local variables, the compiler does need to remember that the
variable exists. That’s what declaring it
does<span class="em">—</span>it adds it to the compiler’s list of
variables in the current scope. We implement that using another new
function.

<div class="codehilite">

<div class="source-file">

*compiler.c*  
add after *identifierConstant*()

</div>

    static void addLocal(Token name) {
      Local* local = &current->locals[current->localCount++];
      local->name = name;
      local->depth = current->scopeDepth;
    }

</div>

<div class="source-file-narrow">

*compiler.c*, add after *identifierConstant*()

</div>

This initializes the next available Local in the compiler’s array of
variables. It stores the variable’s <span id="lexeme">name</span> and
the depth of the scope that owns the variable.

Worried about the lifetime of the string for the variable’s name? The
Local directly stores a copy of the Token struct for the identifier.
Tokens store a pointer to the first character of their lexeme and the
lexeme’s length. That pointer points into the original source string for
the script or REPL entry being compiled.

As long as that string stays around during the entire compilation
process<span class="em">—</span>which it must since, you know, we’re
compiling it<span class="em">—</span>then all of the tokens pointing
into it are fine.

Our implementation is fine for a correct Lox program, but what about
invalid code? Let’s aim to be robust. The first error to handle is not
really the user’s fault, but more a limitation of the VM. The
instructions to work with local variables refer to them by slot index.
That index is stored in a single-byte operand, which means the VM only
supports up to 256 local variables in scope at one time.

If we try to go over that, not only could we not refer to them at
runtime, but the compiler would overwrite its own locals array, too.
Let’s prevent that.

<div class="codehilite">

``` insert-before
static void addLocal(Token name) {
```

<div class="source-file">

*compiler.c*  
in *addLocal*()

</div>

``` insert
  if (current->localCount == UINT8_COUNT) {
    error("Too many local variables in function.");
    return;
  }
```

``` insert-after
  Local* local = &current->locals[current->localCount++];
```

</div>

<div class="source-file-narrow">

*compiler.c*, in *addLocal*()

</div>

The next case is trickier. Consider:

<div class="codehilite">

    {
      var a = "first";
      var a = "second";
    }

</div>

At the top level, Lox allows redeclaring a variable with the same name
as a previous declaration because that’s useful for the REPL. But inside
a local scope, that’s a pretty <span id="rust">weird</span> thing to do.
It’s likely to be a mistake, and many languages, including our own Lox,
enshrine that assumption by making this an error.

Interestingly, the Rust programming language *does* allow this, and
idiomatic code relies on it.

Note that the above program is different from this one:

<div class="codehilite">

    {
      var a = "outer";
      {
        var a = "inner";
      }
    }

</div>

It’s OK to have two variables with the same name in *different* scopes,
even when the scopes overlap such that both are visible at the same
time. That’s shadowing, and Lox does allow that. It’s only an error to
have two variables with the same name in the *same* local scope.

We detect that error like so:

<div class="codehilite">

``` insert-before
  Token* name = &parser.previous;
```

<div class="source-file">

*compiler.c*  
in *declareVariable*()

</div>

``` insert
  for (int i = current->localCount - 1; i >= 0; i--) {
    Local* local = &current->locals[i];
    if (local->depth != -1 && local->depth < current->scopeDepth) {
      break; 
    }

    if (identifiersEqual(name, &local->name)) {
      error("Already a variable with this name in this scope.");
    }
  }
```

``` insert-after
  addLocal(*name);
}
```

</div>

<div class="source-file-narrow">

*compiler.c*, in *declareVariable*()

</div>

Don’t worry about that odd `depth != -1` part yet. We’ll get to what
that’s about later.

Local variables are appended to the array when they’re declared, which
means the current scope is always at the end of the array. When we
declare a new variable, we start at the end and work backward, looking
for an existing variable with the same name. If we find one in the
current scope, we report the error. Otherwise, if we reach the beginning
of the array or a variable owned by another scope, then we know we’ve
checked all of the existing variables in the scope.

To see if two identifiers are the same, we use this:

<div class="codehilite">

<div class="source-file">

*compiler.c*  
add after *identifierConstant*()

</div>

    static bool identifiersEqual(Token* a, Token* b) {
      if (a->length != b->length) return false;
      return memcmp(a->start, b->start, a->length) == 0;
    }

</div>

<div class="source-file-narrow">

*compiler.c*, add after *identifierConstant*()

</div>

Since we know the lengths of both lexemes, we check that first. That
will fail quickly for many non-equal strings. If the
<span id="hash">lengths</span> are the same, we check the characters
using `memcmp()`. To get to `memcmp()`, we need an include.

It would be a nice little optimization if we could check their hashes,
but tokens aren’t full LoxStrings, so we haven’t calculated their hashes
yet.

<div class="codehilite">

``` insert-before
#include <stdlib.h>
```

<div class="source-file">

*compiler.c*

</div>

``` insert
#include <string.h>
```

``` insert-after

#include "common.h"
```

</div>

<div class="source-file-narrow">

*compiler.c*

</div>

With this, we’re able to bring variables into being. But, like ghosts,
they linger on beyond the scope where they are declared. When a block
ends, we need to put them to rest.

<div class="codehilite">

``` insert-before
  current->scopeDepth--;
```

<div class="source-file">

*compiler.c*  
in *endScope*()

</div>

``` insert

  while (current->localCount > 0 &&
         current->locals[current->localCount - 1].depth >
            current->scopeDepth) {
    emitByte(OP_POP);
    current->localCount--;
  }
```

``` insert-after
}
```

</div>

<div class="source-file-narrow">

*compiler.c*, in *endScope*()

</div>

When we pop a scope, we walk backward through the local array looking
for any variables declared at the scope depth we just left. We discard
them by simply decrementing the length of the array.

There is a runtime component to this too. Local variables occupy slots
on the stack. When a local variable goes out of scope, that slot is no
longer needed and should be freed. So, for each variable that we
discard, we also emit an `OP_POP` <span id="pop">instruction</span> to
pop it from the stack.

When multiple local variables go out of scope at once, you get a series
of `OP_POP` instructions that get interpreted one at a time. A simple
optimization you could add to your Lox implementation is a specialized
`OP_POPN` instruction that takes an operand for the number of slots to
pop and pops them all at once.

## <a href="#using-locals" id="using-locals"><span
class="small">22 . 4</span>Using Locals</a>

We can now compile and execute local variable declarations. At runtime,
their values are sitting where they should be on the stack. Let’s start
using them. We’ll do both variable access and assignment at the same
time since they touch the same functions in the compiler.

We already have code for getting and setting global variables,
and<span class="em">—</span>like good little software
engineers<span class="em">—</span>we want to reuse as much of that
existing code as we can. Something like this:

<div class="codehilite">

``` insert-before
static void namedVariable(Token name, bool canAssign) {
```

<div class="source-file">

*compiler.c*  
in *namedVariable*()  
replace 1 line

</div>

``` insert
  uint8_t getOp, setOp;
  int arg = resolveLocal(current, &name);
  if (arg != -1) {
    getOp = OP_GET_LOCAL;
    setOp = OP_SET_LOCAL;
  } else {
    arg = identifierConstant(&name);
    getOp = OP_GET_GLOBAL;
    setOp = OP_SET_GLOBAL;
  }
```

``` insert-after

  if (canAssign && match(TOKEN_EQUAL)) {
```

</div>

<div class="source-file-narrow">

*compiler.c*, in *namedVariable*(), replace 1 line

</div>

Instead of hardcoding the bytecode instructions emitted for variable
access and assignment, we use a couple of C variables. First, we try to
find a local variable with the given name. If we find one, we use the
instructions for working with locals. Otherwise, we assume it’s a global
variable and use the existing bytecode instructions for globals.

A little further down, we use those variables to emit the right
instructions. For assignment:

<div class="codehilite">

``` insert-before
  if (canAssign && match(TOKEN_EQUAL)) {
    expression();
```

<div class="source-file">

*compiler.c*  
in *namedVariable*()  
replace 1 line

</div>

``` insert
    emitBytes(setOp, (uint8_t)arg);
```

``` insert-after
  } else {
```

</div>

<div class="source-file-narrow">

*compiler.c*, in *namedVariable*(), replace 1 line

</div>

And for access:

<div class="codehilite">

``` insert-before
    emitBytes(setOp, (uint8_t)arg);
  } else {
```

<div class="source-file">

*compiler.c*  
in *namedVariable*()  
replace 1 line

</div>

``` insert
    emitBytes(getOp, (uint8_t)arg);
```

``` insert-after
  }
```

</div>

<div class="source-file-narrow">

*compiler.c*, in *namedVariable*(), replace 1 line

</div>

The real heart of this chapter, the part where we resolve a local
variable, is here:

<div class="codehilite">

<div class="source-file">

*compiler.c*  
add after *identifiersEqual*()

</div>

    static int resolveLocal(Compiler* compiler, Token* name) {
      for (int i = compiler->localCount - 1; i >= 0; i--) {
        Local* local = &compiler->locals[i];
        if (identifiersEqual(name, &local->name)) {
          return i;
        }
      }

      return -1;
    }

</div>

<div class="source-file-narrow">

*compiler.c*, add after *identifiersEqual*()

</div>

For all that, it’s straightforward. We walk the list of locals that are
currently in scope. If one has the same name as the identifier token,
the identifier must refer to that variable. We’ve found it! We walk the
array backward so that we find the *last* declared variable with the
identifier. That ensures that inner local variables correctly shadow
locals with the same name in surrounding scopes.

At runtime, we load and store locals using the stack slot index, so
that’s what the compiler needs to calculate after it resolves the
variable. Whenever a variable is declared, we append it to the locals
array in Compiler. That means the first local variable is at index zero,
the next one is at index one, and so on. In other words, the locals
array in the compiler has the *exact* same layout as the VM’s stack will
have at runtime. The variable’s index in the locals array is the same as
its stack slot. How convenient!

If we make it through the whole array without finding a variable with
the given name, it must not be a local. In that case, we return `-1` to
signal that it wasn’t found and should be assumed to be a global
variable instead.

### <a href="#interpreting-local-variables"
id="interpreting-local-variables"><span
class="small">22 . 4 . 1</span>Interpreting local variables</a>

Our compiler is emitting two new instructions, so let’s get them
working. First is loading a local variable:

<div class="codehilite">

``` insert-before
  OP_POP,
```

<div class="source-file">

*chunk.h*  
in enum *OpCode*

</div>

``` insert
  OP_GET_LOCAL,
```

``` insert-after
  OP_GET_GLOBAL,
```

</div>

<div class="source-file-narrow">

*chunk.h*, in enum *OpCode*

</div>

And its implementation:

<div class="codehilite">

``` insert-before
      case OP_POP: pop(); break;
```

<div class="source-file">

*vm.c*  
in *run*()

</div>

``` insert
      case OP_GET_LOCAL: {
        uint8_t slot = READ_BYTE();
        push(vm.stack[slot]); 
        break;
      }
```

``` insert-after
      case OP_GET_GLOBAL: {
```

</div>

<div class="source-file-narrow">

*vm.c*, in *run*()

</div>

It takes a single-byte operand for the stack slot where the local lives.
It loads the value from that index and then pushes it on top of the
stack where later instructions can find it.

It seems redundant to push the local’s value onto the stack since it’s
already on the stack lower down somewhere. The problem is that the other
bytecode instructions only look for data at the *top* of the stack. This
is the core aspect that makes our bytecode instruction set
*stack*-based. [Register-based](a-virtual-machine.html#design-note)
bytecode instruction sets avoid this stack juggling at the cost of
having larger instructions with more operands.

Next is assignment:

<div class="codehilite">

``` insert-before
  OP_GET_LOCAL,
```

<div class="source-file">

*chunk.h*  
in enum *OpCode*

</div>

``` insert
  OP_SET_LOCAL,
```

``` insert-after
  OP_GET_GLOBAL,
```

</div>

<div class="source-file-narrow">

*chunk.h*, in enum *OpCode*

</div>

You can probably predict the implementation.

<div class="codehilite">

``` insert-before
      }
```

<div class="source-file">

*vm.c*  
in *run*()

</div>

``` insert
      case OP_SET_LOCAL: {
        uint8_t slot = READ_BYTE();
        vm.stack[slot] = peek(0);
        break;
      }
```

``` insert-after
      case OP_GET_GLOBAL: {
```

</div>

<div class="source-file-narrow">

*vm.c*, in *run*()

</div>

It takes the assigned value from the top of the stack and stores it in
the stack slot corresponding to the local variable. Note that it doesn’t
pop the value from the stack. Remember, assignment is an expression, and
every expression produces a value. The value of an assignment expression
is the assigned value itself, so the VM just leaves the value on the
stack.

Our disassembler is incomplete without support for these two new
instructions.

<div class="codehilite">

``` insert-before
      return simpleInstruction("OP_POP", offset);
```

<div class="source-file">

*debug.c*  
in *disassembleInstruction*()

</div>

``` insert
    case OP_GET_LOCAL:
      return byteInstruction("OP_GET_LOCAL", chunk, offset);
    case OP_SET_LOCAL:
      return byteInstruction("OP_SET_LOCAL", chunk, offset);
```

``` insert-after
    case OP_GET_GLOBAL:
```

</div>

<div class="source-file-narrow">

*debug.c*, in *disassembleInstruction*()

</div>

The compiler compiles local variables to direct slot access. The local
variable’s name never leaves the compiler to make it into the chunk at
all. That’s great for performance, but not so great for introspection.
When we disassemble these instructions, we can’t show the variable’s
name like we could with globals. Instead, we just show the slot number.

Erasing local variable names in the compiler is a real issue if we ever
want to implement a debugger for our VM. When users step through code,
they expect to see the values of local variables organized by their
names. To support that, we’d need to output some additional information
that tracks the name of each local variable at each stack slot.

<div class="codehilite">

<div class="source-file">

*debug.c*  
add after *simpleInstruction*()

</div>

    static int byteInstruction(const char* name, Chunk* chunk,
                               int offset) {
      uint8_t slot = chunk->code[offset + 1];
      printf("%-16s %4d\n", name, slot);
      return offset + 2; 
    }

</div>

<div class="source-file-narrow">

*debug.c*, add after *simpleInstruction*()

</div>

### <a href="#another-scope-edge-case" id="another-scope-edge-case"><span
class="small">22 . 4 . 2</span>Another scope edge case</a>

We already sunk some time into handling a couple of weird edge cases
around scopes. We made sure shadowing works correctly. We report an
error if two variables in the same local scope have the same name. For
reasons that aren’t entirely clear to me, variable scoping seems to have
a lot of these wrinkles. I’ve never seen a language where it feels
completely <span id="elegant">elegant</span>.

No, not even Scheme.

We’ve got one more edge case to deal with before we end this chapter.
Recall this strange beastie we first met in [jlox’s implementation of
variable
resolution](resolving-and-binding.html#resolving-variable-declarations):

<div class="codehilite">

    {
      var a = "outer";
      {
        var a = a;
      }
    }

</div>

We slayed it then by splitting a variable’s declaration into two phases,
and we’ll do that again here:

![An example variable declaration marked 'declared uninitialized' before
the variable name and 'ready for use' after the
initializer.](image/local-variables/phases.png)

As soon as the variable declaration begins<span class="em">—</span>in
other words, before its initializer<span class="em">—</span>the name is
declared in the current scope. The variable exists, but in a special
“uninitialized” state. Then we compile the initializer. If at any point
in that expression we resolve an identifier that points back to this
variable, we’ll see that it is not initialized yet and report an error.
After we finish compiling the initializer, we mark the variable as
initialized and ready for use.

To implement this, when we declare a local, we need to indicate the
“uninitialized” state somehow. We could add a new field to Local, but
let’s be a little more parsimonious with memory. Instead, we’ll set the
variable’s scope depth to a special sentinel value, `-1`.

<div class="codehilite">

``` insert-before
  local->name = name;
```

<div class="source-file">

*compiler.c*  
in *addLocal*()  
replace 1 line

</div>

``` insert
  local->depth = -1;
```

``` insert-after
}
```

</div>

<div class="source-file-narrow">

*compiler.c*, in *addLocal*(), replace 1 line

</div>

Later, once the variable’s initializer has been compiled, we mark it
initialized.

<div class="codehilite">

``` insert-before
  if (current->scopeDepth > 0) {
```

<div class="source-file">

*compiler.c*  
in *defineVariable*()

</div>

``` insert
    markInitialized();
```

``` insert-after
    return;
  }
```

</div>

<div class="source-file-narrow">

*compiler.c*, in *defineVariable*()

</div>

That is implemented like so:

<div class="codehilite">

<div class="source-file">

*compiler.c*  
add after *parseVariable*()

</div>

    static void markInitialized() {
      current->locals[current->localCount - 1].depth =
          current->scopeDepth;
    }

</div>

<div class="source-file-narrow">

*compiler.c*, add after *parseVariable*()

</div>

So this is *really* what “declaring” and “defining” a variable means in
the compiler. “Declaring” is when the variable is added to the scope,
and “defining” is when it becomes available for use.

When we resolve a reference to a local variable, we check the scope
depth to see if it’s fully defined.

<div class="codehilite">

``` insert-before
    if (identifiersEqual(name, &local->name)) {
```

<div class="source-file">

*compiler.c*  
in *resolveLocal*()

</div>

``` insert
      if (local->depth == -1) {
        error("Can't read local variable in its own initializer.");
      }
```

``` insert-after
      return i;
```

</div>

<div class="source-file-narrow">

*compiler.c*, in *resolveLocal*()

</div>

If the variable has the sentinel depth, it must be a reference to a
variable in its own initializer, and we report that as an error.

That’s it for this chapter! We added blocks, local variables, and real,
honest-to-God lexical scoping. Given that we introduced an entirely
different runtime representation for variables, we didn’t have to write
a lot of code. The implementation ended up being pretty clean and
efficient.

You’ll notice that almost all of the code we wrote is in the compiler.
Over in the runtime, it’s just two little instructions. You’ll see this
as a continuing <span id="static">trend</span> in clox compared to jlox.
One of the biggest hammers in the optimizer’s toolbox is pulling work
forward into the compiler so that you don’t have to do it at runtime. In
this chapter, that meant resolving exactly which stack slot every local
variable occupies. That way, at runtime, no lookup or resolution needs
to happen.

You can look at static types as an extreme example of this trend. A
statically typed language takes all of the type analysis and type error
handling and sorts it all out during compilation. Then the runtime
doesn’t have to waste any time checking that values have the proper type
for their operation. In fact, in some statically typed languages like C,
you don’t even *know* the type at runtime. The compiler completely
erases any representation of a value’s type leaving just the bare bits.

<div class="challenges">

## <a href="#challenges" id="challenges">Challenges</a>

1.  Our simple local array makes it easy to calculate the stack slot of
    each local variable. But it means that when the compiler resolves a
    reference to a variable, we have to do a linear scan through the
    array.

    Come up with something more efficient. Do you think the additional
    complexity is worth it?

2.  How do other languages handle code like this:

    <div class="codehilite">

        var a = a;

    </div>

    What would you do if it was your language? Why?

3.  Many languages make a distinction between variables that can be
    reassigned and those that can’t. In Java, the `final` modifier
    prevents you from assigning to a variable. In JavaScript, a variable
    declared with `let` can be assigned, but one declared using `const`
    can’t. Swift treats `let` as single-assignment and uses `var` for
    assignable variables. Scala and Kotlin use `val` and `var`.

    Pick a keyword for a single-assignment variable form to add to Lox.
    Justify your choice, then implement it. An attempt to assign to a
    variable declared using your new keyword should cause a compile
    error.

4.  Extend clox to allow more than 256 local variables to be in scope at
    a time.

</div>

<a href="jumping-back-and-forth.html" class="next">Next Chapter:
“Jumping Back and Forth” →</a> Handcrafted by Robert Nystrom — <a
href="https://github.com/munificent/craftinginterpreters/blob/master/LICENSE"
target="_blank">© 2015 – 2021</a>

</div>
