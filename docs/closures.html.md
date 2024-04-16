[![](image/logotype.png "Crafting Interpreters")](/)

<div class="contents">

### [Closures<span class="small">25</span>](#top)

- [<span class="small">25.1</span> Closure Objects](#closure-objects)
- [<span class="small">25.2</span> Upvalues](#upvalues)
- [<span class="small">25.3</span> Upvalue Objects](#upvalue-objects)
- [<span class="small">25.4</span> Closed Upvalues](#closed-upvalues)
- 
- [Challenges](#challenges)
- [<span class="small">note</span>Closing Over the Loop
  Variable](#design-note)

<div class="prev-next">

<a href="calls-and-functions.html" class="left"
title="Calls and Functions">← Previous</a>
[↑ Up](a-bytecode-virtual-machine.html "A Bytecode Virtual Machine")
<a href="garbage-collection.html" class="right"
title="Garbage Collection">Next →</a>

</div>

</div>

[![](image/logotype.png "Crafting Interpreters")](/)
<a href="calls-and-functions.html" class="prev"
title="Calls and Functions">←</a>
<a href="garbage-collection.html" class="next"
title="Garbage Collection">→</a>

<div class="page">

<div class="nav-wrapper">

[![](image/logotype.png "Crafting Interpreters")](/)

<div class="expandable">

### [Closures<span class="small">25</span>](#top)

- [<span class="small">25.1</span> Closure Objects](#closure-objects)
- [<span class="small">25.2</span> Upvalues](#upvalues)
- [<span class="small">25.3</span> Upvalue Objects](#upvalue-objects)
- [<span class="small">25.4</span> Closed Upvalues](#closed-upvalues)
- 
- [Challenges](#challenges)
- [<span class="small">note</span>Closing Over the Loop
  Variable](#design-note)

<div class="prev-next">

<a href="calls-and-functions.html" class="left"
title="Calls and Functions">← Previous</a>
[↑ Up](a-bytecode-virtual-machine.html "A Bytecode Virtual Machine")
<a href="garbage-collection.html" class="right"
title="Garbage Collection">Next →</a>

</div>

</div>

<span id="expand-nav">≡</span>

</div>

<div class="number">

25

</div>

# Closures

> As the man said, for every complex problem there’s a simple solution,
> and it’s wrong.
>
> Umberto Eco, *Foucault’s Pendulum*

Thanks to our diligent labor in [the last
chapter](calls-and-functions.html), we have a virtual machine with
working functions. What it lacks is closures. Aside from global
variables, which are their own breed of animal, a function has no way to
reference a variable declared outside of its own body.

<div class="codehilite">

    var x = "global";
    fun outer() {
      var x = "outer";
      fun inner() {
        print x;
      }
      inner();
    }
    outer();

</div>

Run this example now and it prints “global”. It’s supposed to print
“outer”. To fix this, we need to include the entire lexical scope of all
surrounding functions when resolving a variable.

This problem is harder in clox than it was in jlox because our bytecode
VM stores locals on a stack. We used a stack because I claimed locals
have stack semantics<span class="em">—</span>variables are discarded in
the reverse order that they are created. But with closures, that’s only
*mostly* true.

<div class="codehilite">

    fun makeClosure() {
      var local = "local";
      fun closure() {
        print local;
      }
      return closure;
    }

    var closure = makeClosure();
    closure();

</div>

The outer function `makeClosure()` declares a variable, `local`. It also
creates an inner function, `closure()` that captures that variable. Then
`makeClosure()` returns a reference to that function. Since the closure
<span id="flying">escapes</span> while holding on to the local variable,
`local` must outlive the function call where it was created.

<img src="image/closures/flying.png" class="above"
alt="A local variable flying away from the stack." />

Oh no, it’s escaping!

We could solve this problem by dynamically allocating memory for all
local variables. That’s what jlox does by putting everything in those
Environment objects that float around in Java’s heap. But we don’t want
to. Using a <span id="stack">stack</span> is *really* fast. Most local
variables are *not* captured by closures and do have stack semantics. It
would suck to make all of those slower for the benefit of the rare local
that is captured.

There is a reason that C and Java use the stack for their local
variables, after all.

This means a more complex approach than we used in our Java interpreter.
Because some locals have very different lifetimes, we will have two
implementation strategies. For locals that aren’t used in closures,
we’ll keep them just as they are on the stack. When a local is captured
by a closure, we’ll adopt another solution that lifts them onto the heap
where they can live as long as needed.

Closures have been around since the early Lisp days when bytes of memory
and CPU cycles were more precious than emeralds. Over the intervening
decades, hackers devised all <span id="lambda">manner</span> of ways to
compile closures to optimized runtime representations. Some are more
efficient but require a more complex compilation process than we could
easily retrofit into clox.

Search for “closure conversion” or “lambda lifting” to start exploring.

The technique I explain here comes from the design of the Lua VM. It is
fast, parsimonious with memory, and implemented with relatively little
code. Even more impressive, it fits naturally into the single-pass
compilers clox and Lua both use. It is somewhat intricate, though. It
might take a while before all the pieces click together in your mind.
We’ll build them one step at a time, and I’ll try to introduce the
concepts in stages.

## <a href="#closure-objects" id="closure-objects"><span
class="small">25 . 1</span>Closure Objects</a>

Our VM represents functions at runtime using ObjFunction. These objects
are created by the front end during compilation. At runtime, all the VM
does is load the function object from a constant table and bind it to a
name. There is no operation to “create” a function at runtime. Much like
string and number <span id="literal">literals</span>, they are constants
instantiated purely at compile time.

In other words, a function declaration in Lox *is* a kind of
literal<span class="em">—</span>a piece of syntax that defines a
constant value of a built-in type.

That made sense because all of the data that composes a function is
known at compile time: the chunk of bytecode compiled from the
function’s body, and the constants used in the body. Once we introduce
closures, though, that representation is no longer sufficient. Take a
gander at:

<div class="codehilite">

    fun makeClosure(value) {
      fun closure() {
        print value;
      }
      return closure;
    }

    var doughnut = makeClosure("doughnut");
    var bagel = makeClosure("bagel");
    doughnut();
    bagel();

</div>

The `makeClosure()` function defines and returns a function. We call it
twice and get two closures back. They are created by the same nested
function declaration, `closure`, but close over different values. When
we call the two closures, each prints a different string. That implies
we need some runtime representation for a closure that captures the
local variables surrounding the function as they exist when the function
declaration is *executed*, not just when it is compiled.

We’ll work our way up to capturing variables, but a good first step is
defining that object representation. Our existing ObjFunction type
represents the <span id="raw">“raw”</span> compile-time state of a
function declaration, since all closures created from a single
declaration share the same code and constants. At runtime, when we
execute a function declaration, we wrap the ObjFunction in a new
ObjClosure structure. The latter has a reference to the underlying bare
function along with runtime state for the variables the function closes
over.

The Lua implementation refers to the raw function object containing the
bytecode as a “prototype”, which is a great word to describe this,
except that word also gets overloaded to refer to [prototypal
inheritance](https://en.wikipedia.org/wiki/Prototype-based_programming).

![An ObjClosure with a reference to an
ObjFunction.](image/closures/obj-closure.png)

We’ll wrap every function in an ObjClosure, even if the function doesn’t
actually close over and capture any surrounding local variables. This is
a little wasteful, but it simplifies the VM because we can always assume
that the function we’re calling is an ObjClosure. That new struct starts
out like this:

<div class="codehilite">

<div class="source-file">

*object.h*  
add after struct *ObjString*

</div>

    typedef struct {
      Obj obj;
      ObjFunction* function;
    } ObjClosure;

</div>

<div class="source-file-narrow">

*object.h*, add after struct *ObjString*

</div>

Right now, it simply points to an ObjFunction and adds the necessary
object header stuff. Grinding through the usual ceremony for adding a
new object type to clox, we declare a C function to create a new
closure.

<div class="codehilite">

``` insert-before
} ObjClosure;
```

<div class="source-file">

*object.h*  
add after struct *ObjClosure*

</div>

``` insert
ObjClosure* newClosure(ObjFunction* function);
```

``` insert-after
ObjFunction* newFunction();
```

</div>

<div class="source-file-narrow">

*object.h*, add after struct *ObjClosure*

</div>

Then we implement it here:

<div class="codehilite">

<div class="source-file">

*object.c*  
add after *allocateObject*()

</div>

    ObjClosure* newClosure(ObjFunction* function) {
      ObjClosure* closure = ALLOCATE_OBJ(ObjClosure, OBJ_CLOSURE);
      closure->function = function;
      return closure;
    }

</div>

<div class="source-file-narrow">

*object.c*, add after *allocateObject*()

</div>

It takes a pointer to the ObjFunction it wraps. It also initializes the
type field to a new type.

<div class="codehilite">

``` insert-before
typedef enum {
```

<div class="source-file">

*object.h*  
in enum *ObjType*

</div>

``` insert
  OBJ_CLOSURE,
```

``` insert-after
  OBJ_FUNCTION,
```

</div>

<div class="source-file-narrow">

*object.h*, in enum *ObjType*

</div>

And when we’re done with a closure, we release its memory.

<div class="codehilite">

``` insert-before
  switch (object->type) {
```

<div class="source-file">

*memory.c*  
in *freeObject*()

</div>

``` insert
    case OBJ_CLOSURE: {
      FREE(ObjClosure, object);
      break;
    }
```

``` insert-after
    case OBJ_FUNCTION: {
```

</div>

<div class="source-file-narrow">

*memory.c*, in *freeObject*()

</div>

We free only the ObjClosure itself, not the ObjFunction. That’s because
the closure doesn’t *own* the function. There may be multiple closures
that all reference the same function, and none of them claims any
special privilege over it. We can’t free the ObjFunction until *all*
objects referencing it are gone<span class="em">—</span>including even
the surrounding function whose constant table contains it. Tracking that
sounds tricky, and it is! That’s why we’ll write a garbage collector
soon to manage it for us.

We also have the usual <span id="macro">macros</span> for checking a
value’s type.

Perhaps I should have defined a macro to make it easier to generate
these macros. Maybe that would be a little too meta.

<div class="codehilite">

``` insert-before
#define OBJ_TYPE(value)        (AS_OBJ(value)->type)
```

<div class="source-file">

*object.h*

</div>

``` insert
#define IS_CLOSURE(value)      isObjType(value, OBJ_CLOSURE)
```

``` insert-after
#define IS_FUNCTION(value)     isObjType(value, OBJ_FUNCTION)
```

</div>

<div class="source-file-narrow">

*object.h*

</div>

And to cast a value:

<div class="codehilite">

``` insert-before
#define IS_STRING(value)       isObjType(value, OBJ_STRING)
```

<div class="source-file">

*object.h*

</div>

``` insert
#define AS_CLOSURE(value)      ((ObjClosure*)AS_OBJ(value))
```

``` insert-after
#define AS_FUNCTION(value)     ((ObjFunction*)AS_OBJ(value))
```

</div>

<div class="source-file-narrow">

*object.h*

</div>

Closures are first-class objects, so you can print them.

<div class="codehilite">

``` insert-before
  switch (OBJ_TYPE(value)) {
```

<div class="source-file">

*object.c*  
in *printObject*()

</div>

``` insert
    case OBJ_CLOSURE:
      printFunction(AS_CLOSURE(value)->function);
      break;
```

``` insert-after
    case OBJ_FUNCTION:
```

</div>

<div class="source-file-narrow">

*object.c*, in *printObject*()

</div>

They display exactly as ObjFunction does. From the user’s perspective,
the difference between ObjFunction and ObjClosure is purely a hidden
implementation detail. With that out of the way, we have a working but
empty representation for closures.

### <a href="#compiling-to-closure-objects"
id="compiling-to-closure-objects"><span
class="small">25 . 1 . 1</span>Compiling to closure objects</a>

We have closure objects, but our VM never creates them. The next step is
getting the compiler to emit instructions to tell the runtime when to
create a new ObjClosure to wrap a given ObjFunction. This happens right
at the end of a function declaration.

<div class="codehilite">

``` insert-before
  ObjFunction* function = endCompiler();
```

<div class="source-file">

*compiler.c*  
in *function*()  
replace 1 line

</div>

``` insert
  emitBytes(OP_CLOSURE, makeConstant(OBJ_VAL(function)));
```

``` insert-after
}
```

</div>

<div class="source-file-narrow">

*compiler.c*, in *function*(), replace 1 line

</div>

Before, the final bytecode for a function declaration was a single
`OP_CONSTANT` instruction to load the compiled function from the
surrounding function’s constant table and push it onto the stack. Now we
have a new instruction.

<div class="codehilite">

``` insert-before
  OP_CALL,
```

<div class="source-file">

*chunk.h*  
in enum *OpCode*

</div>

``` insert
  OP_CLOSURE,
```

``` insert-after
  OP_RETURN,
```

</div>

<div class="source-file-narrow">

*chunk.h*, in enum *OpCode*

</div>

Like `OP_CONSTANT`, it takes a single operand that represents a constant
table index for the function. But when we get over to the runtime
implementation, we do something more interesting.

First, let’s be diligent VM hackers and slot in disassembler support for
the instruction.

<div class="codehilite">

``` insert-before
    case OP_CALL:
      return byteInstruction("OP_CALL", chunk, offset);
```

<div class="source-file">

*debug.c*  
in *disassembleInstruction*()

</div>

``` insert
    case OP_CLOSURE: {
      offset++;
      uint8_t constant = chunk->code[offset++];
      printf("%-16s %4d ", "OP_CLOSURE", constant);
      printValue(chunk->constants.values[constant]);
      printf("\n");
      return offset;
    }
```

``` insert-after
    case OP_RETURN:
```

</div>

<div class="source-file-narrow">

*debug.c*, in *disassembleInstruction*()

</div>

There’s more going on here than we usually have in the disassembler. By
the end of the chapter, you’ll discover that `OP_CLOSURE` is quite an
unusual instruction. It’s straightforward right
now<span class="em">—</span>just a single byte
operand<span class="em">—</span>but we’ll be adding to it. This code
here anticipates that future.

### <a href="#interpreting-function-declarations"
id="interpreting-function-declarations"><span
class="small">25 . 1 . 2</span>Interpreting function declarations</a>

Most of the work we need to do is in the runtime. We have to handle the
new instruction, naturally. But we also need to touch every piece of
code in the VM that works with ObjFunction and change it to use
ObjClosure instead<span class="em">—</span>function calls, call frames,
etc. We’ll start with the instruction, though.

<div class="codehilite">

``` insert-before
      }
```

<div class="source-file">

*vm.c*  
in *run*()

</div>

``` insert
      case OP_CLOSURE: {
        ObjFunction* function = AS_FUNCTION(READ_CONSTANT());
        ObjClosure* closure = newClosure(function);
        push(OBJ_VAL(closure));
        break;
      }
```

``` insert-after
      case OP_RETURN: {
```

</div>

<div class="source-file-narrow">

*vm.c*, in *run*()

</div>

Like the `OP_CONSTANT` instruction we used before, first we load the
compiled function from the constant table. The difference now is that we
wrap that function in a new ObjClosure and push the result onto the
stack.

Once you have a closure, you’ll eventually want to call it.

<div class="codehilite">

``` insert-before
    switch (OBJ_TYPE(callee)) {
```

<div class="source-file">

*vm.c*  
in *callValue*()  
replace 2 lines

</div>

``` insert
      case OBJ_CLOSURE:
        return call(AS_CLOSURE(callee), argCount);
```

``` insert-after
      case OBJ_NATIVE: {
```

</div>

<div class="source-file-narrow">

*vm.c*, in *callValue*(), replace 2 lines

</div>

We remove the code for calling objects whose type is `OBJ_FUNCTION`.
Since we wrap all functions in ObjClosures, the runtime will never try
to invoke a bare ObjFunction anymore. Those objects live only in
constant tables and get immediately <span id="naked">wrapped</span> in
closures before anything else sees them.

We don’t want any naked functions wandering around the VM! What would
the neighbors say?

We replace the old code with very similar code for calling a closure
instead. The only difference is the type of object we pass to `call()`.
The real changes are over in that function. First, we update its
signature.

<div class="codehilite">

<div class="source-file">

*vm.c*  
function *call*()  
replace 1 line

</div>

``` insert
static bool call(ObjClosure* closure, int argCount) {
```

``` insert-after
  if (argCount != function->arity) {
```

</div>

<div class="source-file-narrow">

*vm.c*, function *call*(), replace 1 line

</div>

Then, in the body, we need to fix everything that referenced the
function to handle the fact that we’ve introduced a layer of
indirection. We start with the arity checking:

<div class="codehilite">

``` insert-before
static bool call(ObjClosure* closure, int argCount) {
```

<div class="source-file">

*vm.c*  
in *call*()  
replace 3 lines

</div>

``` insert
  if (argCount != closure->function->arity) {
    runtimeError("Expected %d arguments but got %d.",
        closure->function->arity, argCount);
```

``` insert-after
    return false;
```

</div>

<div class="source-file-narrow">

*vm.c*, in *call*(), replace 3 lines

</div>

The only change is that we unwrap the closure to get to the underlying
function. The next thing `call()` does is create a new CallFrame. We
change that code to store the closure in the CallFrame and get the
bytecode pointer from the closure’s function.

<div class="codehilite">

``` insert-before
  CallFrame* frame = &vm.frames[vm.frameCount++];
```

<div class="source-file">

*vm.c*  
in *call*()  
replace 2 lines

</div>

``` insert
  frame->closure = closure;
  frame->ip = closure->function->chunk.code;
```

``` insert-after
  frame->slots = vm.stackTop - argCount - 1;
```

</div>

<div class="source-file-narrow">

*vm.c*, in *call*(), replace 2 lines

</div>

This necessitates changing the declaration of CallFrame too.

<div class="codehilite">

``` insert-before
typedef struct {
```

<div class="source-file">

*vm.h*  
in struct *CallFrame*  
replace 1 line

</div>

``` insert
  ObjClosure* closure;
```

``` insert-after
  uint8_t* ip;
```

</div>

<div class="source-file-narrow">

*vm.h*, in struct *CallFrame*, replace 1 line

</div>

That change triggers a few other cascading changes. Every place in the
VM that accessed CallFrame’s function needs to use a closure instead.
First, the macro for reading a constant from the current function’s
constant table:

<div class="codehilite">

``` insert-before
    (uint16_t)((frame->ip[-2] << 8) | frame->ip[-1]))
```

<div class="source-file">

*vm.c*  
in *run*()  
replace 2 lines

</div>

``` insert
#define READ_CONSTANT() \
    (frame->closure->function->chunk.constants.values[READ_BYTE()])
```

``` insert-after

#define READ_STRING() AS_STRING(READ_CONSTANT())
```

</div>

<div class="source-file-narrow">

*vm.c*, in *run*(), replace 2 lines

</div>

When `DEBUG_TRACE_EXECUTION` is enabled, it needs to get to the chunk
from the closure.

<div class="codehilite">

``` insert-before
    printf("\n");
```

<div class="source-file">

*vm.c*  
in *run*()  
replace 2 lines

</div>

``` insert
    disassembleInstruction(&frame->closure->function->chunk,
        (int)(frame->ip - frame->closure->function->chunk.code));
```

``` insert-after
#endif
```

</div>

<div class="source-file-narrow">

*vm.c*, in *run*(), replace 2 lines

</div>

Likewise when reporting a runtime error:

<div class="codehilite">

``` insert-before
    CallFrame* frame = &vm.frames[i];
```

<div class="source-file">

*vm.c*  
in *runtimeError*()  
replace 1 line

</div>

``` insert
    ObjFunction* function = frame->closure->function;
```

``` insert-after
    size_t instruction = frame->ip - function->chunk.code - 1;
```

</div>

<div class="source-file-narrow">

*vm.c*, in *runtimeError*(), replace 1 line

</div>

Almost there. The last piece is the blob of code that sets up the very
first CallFrame to begin executing the top-level code for a Lox script.

<div class="codehilite">

``` insert-before
  push(OBJ_VAL(function));
```

<div class="source-file">

*vm.c*  
in *interpret*()  
replace 1 line

</div>

``` insert
  ObjClosure* closure = newClosure(function);
  pop();
  push(OBJ_VAL(closure));
  call(closure, 0);
```

``` insert-after

  return run();
```

</div>

<div class="source-file-narrow">

*vm.c*, in *interpret*(), replace 1 line

</div>

<span id="pop">The</span> compiler still returns a raw ObjFunction when
compiling a script. That’s fine, but it means we need to wrap it in an
ObjClosure here, before the VM can execute it.

The code looks a little silly because we still push the original
ObjFunction onto the stack. Then we pop it after creating the closure,
only to then push the closure. Why put the ObjFunction on there at all?
As usual, when you see weird stack stuff going on, it’s to keep the
[forthcoming garbage collector](garbage-collection.html) aware of some
heap-allocated objects.

We are back to a working interpreter. The *user* can’t tell any
difference, but the compiler now generates code telling the VM to create
a closure for each function declaration. Every time the VM executes a
function declaration, it wraps the ObjFunction in a new ObjClosure. The
rest of the VM now handles those ObjClosures floating around. That’s the
boring stuff out of the way. Now we’re ready to make these closures
actually *do* something.

## <a href="#upvalues" id="upvalues"><span
class="small">25 . 2</span>Upvalues</a>

Our existing instructions for reading and writing local variables are
limited to a single function’s stack window. Locals from a surrounding
function are outside of the inner function’s window. We’re going to need
some new instructions.

The easiest approach might be an instruction that takes a relative stack
slot offset that can reach *before* the current function’s window. That
would work if closed-over variables were always on the stack. But as we
saw earlier, these variables sometimes outlive the function where they
are declared. That means they won’t always be on the stack.

The next easiest approach, then, would be to take any local variable
that gets closed over and have it always live on the heap. When the
local variable declaration in the surrounding function is executed, the
VM would allocate memory for it dynamically. That way it could live as
long as needed.

This would be a fine approach if clox didn’t have a single-pass
compiler. But that restriction we chose in our implementation makes
things harder. Take a look at this example:

<div class="codehilite">

    fun outer() {
      var x = 1;    // (1)
      x = 2;        // (2)
      fun inner() { // (3)
        print x;
      }
      inner();
    }

</div>

Here, the compiler compiles the declaration of `x` at `(1)` and emits
code for the assignment at `(2)`. It does that before reaching the
declaration of `inner()` at `(3)` and discovering that `x` is in fact
closed over. We don’t have an easy way to go back and fix that
already-emitted code to treat `x` specially. Instead, we want a solution
that allows a closed-over variable to live on the stack exactly like a
normal local variable *until the point that it is closed over*.

Fortunately, thanks to the Lua dev team, we have a solution. We use a
level of indirection that they call an **upvalue**. An upvalue refers to
a local variable in an enclosing function. Every closure maintains an
array of upvalues, one for each surrounding local variable that the
closure uses.

The upvalue points back into the stack to where the variable it captured
lives. When the closure needs to access a closed-over variable, it goes
through the corresponding upvalue to reach it. When a function
declaration is first executed and we create a closure for it, the VM
creates the array of upvalues and wires them up to “capture” the
surrounding local variables that the closure needs.

For example, if we throw this program at clox,

<div class="codehilite">

    {
      var a = 3;
      fun f() {
        print a;
      }
    }

</div>

the compiler and runtime will conspire together to build up a set of
objects in memory like this:

![The object graph of the stack, ObjClosure, ObjFunction, and upvalue
array.](image/closures/open-upvalue.png)

That might look overwhelming, but fear not. We’ll work our way through
it. The important part is that upvalues serve as the layer of
indirection needed to continue to find a captured local variable even
after it moves off the stack. But before we get to all that, let’s focus
on compiling captured variables.

### <a href="#compiling-upvalues" id="compiling-upvalues"><span
class="small">25 . 2 . 1</span>Compiling upvalues</a>

As usual, we want to do as much work as possible during compilation to
keep execution simple and fast. Since local variables are lexically
scoped in Lox, we have enough knowledge at compile time to resolve which
surrounding local variables a function accesses and where those locals
are declared. That, in turn, means we know *how many* upvalues a closure
needs, *which* variables they capture, and *which stack slots* contain
those variables in the declaring function’s stack window.

Currently, when the compiler resolves an identifier, it walks the block
scopes for the current function from innermost to outermost. If we don’t
find the variable in that function, we assume the variable must be a
global. We don’t consider the local scopes of enclosing
functions<span class="em">—</span>they get skipped right over. The first
change, then, is inserting a resolution step for those outer local
scopes.

<div class="codehilite">

``` insert-before
  if (arg != -1) {
    getOp = OP_GET_LOCAL;
    setOp = OP_SET_LOCAL;
```

<div class="source-file">

*compiler.c*  
in *namedVariable*()

</div>

``` insert
  } else if ((arg = resolveUpvalue(current, &name)) != -1) {
    getOp = OP_GET_UPVALUE;
    setOp = OP_SET_UPVALUE;
```

``` insert-after
  } else {
```

</div>

<div class="source-file-narrow">

*compiler.c*, in *namedVariable*()

</div>

This new `resolveUpvalue()` function looks for a local variable declared
in any of the surrounding functions. If it finds one, it returns an
“upvalue index” for that variable. (We’ll get into what that means
later.) Otherwise, it returns -1 to indicate the variable wasn’t found.
If it was found, we use these two new instructions for reading or
writing to the variable through its upvalue:

<div class="codehilite">

``` insert-before
  OP_SET_GLOBAL,
```

<div class="source-file">

*chunk.h*  
in enum *OpCode*

</div>

``` insert
  OP_GET_UPVALUE,
  OP_SET_UPVALUE,
```

``` insert-after
  OP_EQUAL,
```

</div>

<div class="source-file-narrow">

*chunk.h*, in enum *OpCode*

</div>

We’re implementing this sort of top-down, so I’ll show you how these
work at runtime soon. The part to focus on now is how the compiler
actually resolves the identifier.

<div class="codehilite">

<div class="source-file">

*compiler.c*  
add after *resolveLocal*()

</div>

    static int resolveUpvalue(Compiler* compiler, Token* name) {
      if (compiler->enclosing == NULL) return -1;

      int local = resolveLocal(compiler->enclosing, name);
      if (local != -1) {
        return addUpvalue(compiler, (uint8_t)local, true);
      }

      return -1;
    }

</div>

<div class="source-file-narrow">

*compiler.c*, add after *resolveLocal*()

</div>

We call this after failing to resolve a local variable in the current
function’s scope, so we know the variable isn’t in the current compiler.
Recall that Compiler stores a pointer to the Compiler for the enclosing
function, and these pointers form a linked chain that goes all the way
to the root Compiler for the top-level code. Thus, if the enclosing
Compiler is `NULL`, we know we’ve reached the outermost function without
finding a local variable. The variable must be
<span id="undefined">global</span>, so we return -1.

It might end up being an entirely undefined variable and not even
global. But in Lox, we don’t detect that error until runtime, so from
the compiler’s perspective, it’s “hopefully global”.

Otherwise, we try to resolve the identifier as a *local* variable in the
*enclosing* compiler. In other words, we look for it right outside the
current function. For example:

<div class="codehilite">

    fun outer() {
      var x = 1;
      fun inner() {
        print x; // (1)
      }
      inner();
    }

</div>

When compiling the identifier expression at `(1)`, `resolveUpvalue()`
looks for a local variable `x` declared in `outer()`. If
found<span class="em">—</span>like it is in this
example<span class="em">—</span>then we’ve successfully resolved the
variable. We create an upvalue so that the inner function can access the
variable through that. The upvalue is created here:

<div class="codehilite">

<div class="source-file">

*compiler.c*  
add after *resolveLocal*()

</div>

    static int addUpvalue(Compiler* compiler, uint8_t index,
                          bool isLocal) {
      int upvalueCount = compiler->function->upvalueCount;
      compiler->upvalues[upvalueCount].isLocal = isLocal;
      compiler->upvalues[upvalueCount].index = index;
      return compiler->function->upvalueCount++;
    }

</div>

<div class="source-file-narrow">

*compiler.c*, add after *resolveLocal*()

</div>

The compiler keeps an array of upvalue structures to track the
closed-over identifiers that it has resolved in the body of each
function. Remember how the compiler’s Local array mirrors the stack slot
indexes where locals live at runtime? This new upvalue array works the
same way. The indexes in the compiler’s array match the indexes where
upvalues will live in the ObjClosure at runtime.

This function adds a new upvalue to that array. It also keeps track of
the number of upvalues the function uses. It stores that count directly
in the ObjFunction itself because we’ll also
<span id="bridge">need</span> that number for use at runtime.

Like constants and function arity, the upvalue count is another one of
those little pieces of data that form the bridge between the compiler
and runtime.

The `index` field tracks the closed-over local variable’s slot index.
That way the compiler knows *which* variable in the enclosing function
needs to be captured. We’ll circle back to what that `isLocal` field is
for before too long. Finally, `addUpvalue()` returns the index of the
created upvalue in the function’s upvalue list. That index becomes the
operand to the `OP_GET_UPVALUE` and `OP_SET_UPVALUE` instructions.

That’s the basic idea for resolving upvalues, but the function isn’t
fully baked. A closure may reference the same variable in a surrounding
function multiple times. In that case, we don’t want to waste time and
memory creating a separate upvalue for each identifier expression. To
fix that, before we add a new upvalue, we first check to see if the
function already has an upvalue that closes over that variable.

<div class="codehilite">

``` insert-before
  int upvalueCount = compiler->function->upvalueCount;
```

<div class="source-file">

*compiler.c*  
in *addUpvalue*()

</div>

``` insert

  for (int i = 0; i < upvalueCount; i++) {
    Upvalue* upvalue = &compiler->upvalues[i];
    if (upvalue->index == index && upvalue->isLocal == isLocal) {
      return i;
    }
  }
```

``` insert-after
  compiler->upvalues[upvalueCount].isLocal = isLocal;
```

</div>

<div class="source-file-narrow">

*compiler.c*, in *addUpvalue*()

</div>

If we find an upvalue in the array whose slot index matches the one
we’re adding, we just return that *upvalue* index and reuse it.
Otherwise, we fall through and add the new upvalue.

These two functions access and modify a bunch of new state, so let’s
define that. First, we add the upvalue count to ObjFunction.

<div class="codehilite">

``` insert-before
  int arity;
```

<div class="source-file">

*object.h*  
in struct *ObjFunction*

</div>

``` insert
  int upvalueCount;
```

``` insert-after
  Chunk chunk;
```

</div>

<div class="source-file-narrow">

*object.h*, in struct *ObjFunction*

</div>

We’re conscientious C programmers, so we zero-initialize that when an
ObjFunction is first allocated.

<div class="codehilite">

``` insert-before
  function->arity = 0;
```

<div class="source-file">

*object.c*  
in *newFunction*()

</div>

``` insert
  function->upvalueCount = 0;
```

``` insert-after
  function->name = NULL;
```

</div>

<div class="source-file-narrow">

*object.c*, in *newFunction*()

</div>

In the compiler, we add a field for the upvalue array.

<div class="codehilite">

``` insert-before
  int localCount;
```

<div class="source-file">

*compiler.c*  
in struct *Compiler*

</div>

``` insert
  Upvalue upvalues[UINT8_COUNT];
```

``` insert-after
  int scopeDepth;
```

</div>

<div class="source-file-narrow">

*compiler.c*, in struct *Compiler*

</div>

For simplicity, I gave it a fixed size. The `OP_GET_UPVALUE` and
`OP_SET_UPVALUE` instructions encode an upvalue index using a single
byte operand, so there’s a restriction on how many upvalues a function
can have<span class="em">—</span>how many unique variables it can close
over. Given that, we can afford a static array that large. We also need
to make sure the compiler doesn’t overflow that limit.

<div class="codehilite">

``` insert-before
    if (upvalue->index == index && upvalue->isLocal == isLocal) {
      return i;
    }
  }
```

<div class="source-file">

*compiler.c*  
in *addUpvalue*()

</div>

``` insert
  if (upvalueCount == UINT8_COUNT) {
    error("Too many closure variables in function.");
    return 0;
  }
```

``` insert-after
  compiler->upvalues[upvalueCount].isLocal = isLocal;
```

</div>

<div class="source-file-narrow">

*compiler.c*, in *addUpvalue*()

</div>

Finally, the Upvalue struct type itself.

<div class="codehilite">

<div class="source-file">

*compiler.c*  
add after struct *Local*

</div>

    typedef struct {
      uint8_t index;
      bool isLocal;
    } Upvalue;

</div>

<div class="source-file-narrow">

*compiler.c*, add after struct *Local*

</div>

The `index` field stores which local slot the upvalue is capturing. The
`isLocal` field deserves its own section, which we’ll get to next.

### <a href="#flattening-upvalues" id="flattening-upvalues"><span
class="small">25 . 2 . 2</span>Flattening upvalues</a>

In the example I showed before, the closure is accessing a variable
declared in the immediately enclosing function. Lox also supports
accessing local variables declared in *any* enclosing scope, as in:

<div class="codehilite">

    fun outer() {
      var x = 1;
      fun middle() {
        fun inner() {
          print x;
        }
      }
    }

</div>

Here, we’re accessing `x` in `inner()`. That variable is defined not in
`middle()`, but all the way out in `outer()`. We need to handle cases
like this too. You *might* think that this isn’t much harder since the
variable will simply be somewhere farther down on the stack. But
consider this <span id="devious">devious</span> example:

If you work on programming languages long enough, you will develop a
finely honed skill at creating bizarre programs like this that are
technically valid but likely to trip up an implementation written by
someone with a less perverse imagination than you.

<div class="codehilite">

    fun outer() {
      var x = "value";
      fun middle() {
        fun inner() {
          print x;
        }

        print "create inner closure";
        return inner;
      }

      print "return from outer";
      return middle;
    }

    var mid = outer();
    var in = mid();
    in();

</div>

When you run this, it should print:

<div class="codehilite">

    return from outer
    create inner closure
    value

</div>

I know, it’s convoluted. The important part is that
`outer()`<span class="em">—</span>where `x` is
declared<span class="em">—</span>returns and pops all of its variables
off the stack before the *declaration* of `inner()` executes. So, at the
point in time that we create the closure for `inner()`, `x` is already
off the stack.

Here, I traced out the execution flow for you:

![Tracing through the previous example
program.](image/closures/execution-flow.png)

See how `x` is popped ① before it is captured ② and then later accessed
③? We really have two problems:

1.  We need to resolve local variables that are declared in surrounding
    functions beyond the immediately enclosing one.

2.  We need to be able to capture variables that have already left the
    stack.

Fortunately, we’re in the middle of adding upvalues to the VM, and
upvalues are explicitly designed for tracking variables that have
escaped the stack. So, in a clever bit of self-reference, we can use
upvalues to allow upvalues to capture variables declared outside of the
immediately surrounding function.

The solution is to allow a closure to capture either a local variable or
*an existing upvalue* in the immediately enclosing function. If a deeply
nested function references a local variable declared several hops away,
we’ll thread it through all of the intermediate functions by having each
function capture an upvalue for the next function to grab.

![An upvalue in inner() points to an upvalue in middle(), which points
to a local variable in outer().](image/closures/linked-upvalues.png)

In the above example, `middle()` captures the local variable `x` in the
immediately enclosing function `outer()` and stores it in its own
upvalue. It does this even though `middle()` itself doesn’t reference
`x`. Then, when the declaration of `inner()` executes, its closure grabs
the *upvalue* from the ObjClosure for `middle()` that captured `x`. A
function captures<span class="em">—</span>either a local or
upvalue<span class="em">—</span>*only* from the immediately surrounding
function, which is guaranteed to still be around at the point that the
inner function declaration executes.

In order to implement this, `resolveUpvalue()` becomes recursive.

<div class="codehilite">

``` insert-before
  if (local != -1) {
    return addUpvalue(compiler, (uint8_t)local, true);
  }
```

<div class="source-file">

*compiler.c*  
in *resolveUpvalue*()

</div>

``` insert
  int upvalue = resolveUpvalue(compiler->enclosing, name);
  if (upvalue != -1) {
    return addUpvalue(compiler, (uint8_t)upvalue, false);
  }
```

``` insert-after
  return -1;
```

</div>

<div class="source-file-narrow">

*compiler.c*, in *resolveUpvalue*()

</div>

It’s only another three lines of code, but I found this function really
challenging to get right the first time. This in spite of the fact that
I wasn’t inventing anything new, just porting the concept over from Lua.
Most recursive functions either do all their work before the recursive
call (a **pre-order traversal**, or “on the way down”), or they do all
the work after the recursive call (a **post-order traversal**, or “on
the way back up”). This function does both. The recursive call is right
in the middle.

We’ll walk through it slowly. First, we look for a matching local
variable in the enclosing function. If we find one, we capture that
local and return. That’s the <span id="base">base</span> case.

The other base case, of course, is if there is no enclosing function. In
that case, the variable can’t be resolved lexically and is treated as
global.

Otherwise, we look for a local variable beyond the immediately enclosing
function. We do that by recursively calling `resolveUpvalue()` on the
*enclosing* compiler, not the current one. This series of
`resolveUpvalue()` calls works its way along the chain of nested
compilers until it hits one of the base
cases<span class="em">—</span>either it finds an actual local variable
to capture or it runs out of compilers.

When a local variable is found, the most deeply
<span id="outer">nested</span> call to `resolveUpvalue()` captures it
and returns the upvalue index. That returns to the next call for the
inner function declaration. That call captures the *upvalue* from the
surrounding function, and so on. As each nested call to
`resolveUpvalue()` returns, we drill back down into the innermost
function declaration where the identifier we are resolving appears. At
each step along the way, we add an upvalue to the intervening function
and pass the resulting upvalue index down to the next call.

Each recursive call to `resolveUpvalue()` walks *out* one level of
function nesting. So an inner *recursive call* refers to an *outer*
nested declaration. The innermost recursive call to `resolveUpvalue()`
that finds the local variable will be for the *outermost* function, just
inside the enclosing function where that variable is actually declared.

It might help to walk through the original example when resolving `x`:

![Tracing through a recursive call to
resolveUpvalue().](image/closures/recursion.png)

Note that the new call to `addUpvalue()` passes `false` for the
`isLocal` parameter. Now you see that that flag controls whether the
closure captures a local variable or an upvalue from the surrounding
function.

By the time the compiler reaches the end of a function declaration,
every variable reference has been resolved as either a local, an
upvalue, or a global. Each upvalue may in turn capture a local variable
from the surrounding function, or an upvalue in the case of transitive
closures. We finally have enough data to emit bytecode which creates a
closure at runtime that captures all of the correct variables.

<div class="codehilite">

``` insert-before
  emitBytes(OP_CLOSURE, makeConstant(OBJ_VAL(function)));
```

<div class="source-file">

*compiler.c*  
in *function*()

</div>

``` insert

  for (int i = 0; i < function->upvalueCount; i++) {
    emitByte(compiler.upvalues[i].isLocal ? 1 : 0);
    emitByte(compiler.upvalues[i].index);
  }
```

``` insert-after
}
```

</div>

<div class="source-file-narrow">

*compiler.c*, in *function*()

</div>

The `OP_CLOSURE` instruction is unique in that it has a variably sized
encoding. For each upvalue the closure captures, there are two
single-byte operands. Each pair of operands specifies what that upvalue
captures. If the first byte is one, it captures a local variable in the
enclosing function. If zero, it captures one of the function’s upvalues.
The next byte is the local slot or upvalue index to capture.

This odd encoding means we need some bespoke support in the disassembly
code for `OP_CLOSURE`.

<div class="codehilite">

``` insert-before
      printf("\n");
```

<div class="source-file">

*debug.c*  
in *disassembleInstruction*()

</div>

``` insert

      ObjFunction* function = AS_FUNCTION(
          chunk->constants.values[constant]);
      for (int j = 0; j < function->upvalueCount; j++) {
        int isLocal = chunk->code[offset++];
        int index = chunk->code[offset++];
        printf("%04d      |                     %s %d\n",
               offset - 2, isLocal ? "local" : "upvalue", index);
      }
```

``` insert-after
      return offset;
```

</div>

<div class="source-file-narrow">

*debug.c*, in *disassembleInstruction*()

</div>

For example, take this script:

<div class="codehilite">

    fun outer() {
      var a = 1;
      var b = 2;
      fun middle() {
        var c = 3;
        var d = 4;
        fun inner() {
          print a + c + b + d;
        }
      }
    }

</div>

If we disassemble the instruction that creates the closure for
`inner()`, it prints this:

<div class="codehilite">

    0004    9 OP_CLOSURE          2 <fn inner>
    0006      |                     upvalue 0
    0008      |                     local 1
    0010      |                     upvalue 1
    0012      |                     local 2

</div>

We have two other, simpler instructions to add disassembler support for.

<div class="codehilite">

``` insert-before
    case OP_SET_GLOBAL:
      return constantInstruction("OP_SET_GLOBAL", chunk, offset);
```

<div class="source-file">

*debug.c*  
in *disassembleInstruction*()

</div>

``` insert
    case OP_GET_UPVALUE:
      return byteInstruction("OP_GET_UPVALUE", chunk, offset);
    case OP_SET_UPVALUE:
      return byteInstruction("OP_SET_UPVALUE", chunk, offset);
```

``` insert-after
    case OP_EQUAL:
```

</div>

<div class="source-file-narrow">

*debug.c*, in *disassembleInstruction*()

</div>

These both have a single-byte operand, so there’s nothing exciting going
on. We do need to add an include so the debug module can get to
`AS_FUNCTION()`.

<div class="codehilite">

``` insert-before
#include "debug.h"
```

<div class="source-file">

*debug.c*

</div>

``` insert
#include "object.h"
```

``` insert-after
#include "value.h"
```

</div>

<div class="source-file-narrow">

*debug.c*

</div>

With that, our compiler is where we want it. For each function
declaration, it outputs an `OP_CLOSURE` instruction followed by a series
of operand byte pairs for each upvalue it needs to capture at runtime.
It’s time to hop over to that side of the VM and get things running.

## <a href="#upvalue-objects" id="upvalue-objects"><span
class="small">25 . 3</span>Upvalue Objects</a>

Each `OP_CLOSURE` instruction is now followed by the series of bytes
that specify the upvalues the ObjClosure should own. Before we process
those operands, we need a runtime representation for upvalues.

<div class="codehilite">

<div class="source-file">

*object.h*  
add after struct *ObjString*

</div>

    typedef struct ObjUpvalue {
      Obj obj;
      Value* location;
    } ObjUpvalue;

</div>

<div class="source-file-narrow">

*object.h*, add after struct *ObjString*

</div>

We know upvalues must manage closed-over variables that no longer live
on the stack, which implies some amount of dynamic allocation. The
easiest way to do that in our VM is by building on the object system we
already have. That way, when we implement a garbage collector in [the
next chapter](garbage-collection.html), the GC can manage memory for
upvalues too.

Thus, our runtime upvalue structure is an ObjUpvalue with the typical
Obj header field. Following that is a `location` field that points to
the closed-over variable. Note that this is a *pointer* to a Value, not
a Value itself. It’s a reference to a *variable*, not a *value*. This is
important because it means that when we assign to the variable the
upvalue captures, we’re assigning to the actual variable, not a copy.
For example:

<div class="codehilite">

    fun outer() {
      var x = "before";
      fun inner() {
        x = "assigned";
      }
      inner();
      print x;
    }
    outer();

</div>

This program should print “assigned” even though the closure assigns to
`x` and the surrounding function accesses it.

Because upvalues are objects, we’ve got all the usual object machinery,
starting with a constructor-like function:

<div class="codehilite">

``` insert-before
ObjString* copyString(const char* chars, int length);
```

<div class="source-file">

*object.h*  
add after *copyString*()

</div>

``` insert
ObjUpvalue* newUpvalue(Value* slot);
```

``` insert-after
void printObject(Value value);
```

</div>

<div class="source-file-narrow">

*object.h*, add after *copyString*()

</div>

It takes the address of the slot where the closed-over variable lives.
Here is the implementation:

<div class="codehilite">

<div class="source-file">

*object.c*  
add after *copyString*()

</div>

    ObjUpvalue* newUpvalue(Value* slot) {
      ObjUpvalue* upvalue = ALLOCATE_OBJ(ObjUpvalue, OBJ_UPVALUE);
      upvalue->location = slot;
      return upvalue;
    }

</div>

<div class="source-file-narrow">

*object.c*, add after *copyString*()

</div>

We simply initialize the object and store the pointer. That requires a
new object type.

<div class="codehilite">

``` insert-before
  OBJ_STRING,
```

<div class="source-file">

*object.h*  
in enum *ObjType*

</div>

``` insert
  OBJ_UPVALUE
```

``` insert-after
} ObjType;
```

</div>

<div class="source-file-narrow">

*object.h*, in enum *ObjType*

</div>

And on the back side, a destructor-like function:

<div class="codehilite">

``` insert-before
      FREE(ObjString, object);
      break;
    }
```

<div class="source-file">

*memory.c*  
in *freeObject*()

</div>

``` insert
    case OBJ_UPVALUE:
      FREE(ObjUpvalue, object);
      break;
```

``` insert-after
  }
```

</div>

<div class="source-file-narrow">

*memory.c*, in *freeObject*()

</div>

Multiple closures can close over the same variable, so ObjUpvalue does
not own the variable it references. Thus, the only thing to free is the
ObjUpvalue itself.

And, finally, to print:

<div class="codehilite">

``` insert-before
    case OBJ_STRING:
      printf("%s", AS_CSTRING(value));
      break;
```

<div class="source-file">

*object.c*  
in *printObject*()

</div>

``` insert
    case OBJ_UPVALUE:
      printf("upvalue");
      break;
```

``` insert-after
  }
```

</div>

<div class="source-file-narrow">

*object.c*, in *printObject*()

</div>

Printing isn’t useful to end users. Upvalues are objects only so that we
can take advantage of the VM’s memory management. They aren’t
first-class values that a Lox user can directly access in a program. So
this code will never actually
execute<span class="ellipse"> . . . </span>but it keeps the compiler
from yelling at us about an unhandled switch case, so here we are.

### <a href="#upvalues-in-closures" id="upvalues-in-closures"><span
class="small">25 . 3 . 1</span>Upvalues in closures</a>

When I first introduced upvalues, I said each closure has an array of
them. We’ve finally worked our way back to implementing that.

<div class="codehilite">

``` insert-before
  ObjFunction* function;
```

<div class="source-file">

*object.h*  
in struct *ObjClosure*

</div>

``` insert
  ObjUpvalue** upvalues;
  int upvalueCount;
```

``` insert-after
} ObjClosure;
```

</div>

<div class="source-file-narrow">

*object.h*, in struct *ObjClosure*

</div>

<span id="count">Different</span> closures may have different numbers of
upvalues, so we need a dynamic array. The upvalues themselves are
dynamically allocated too, so we end up with a double
pointer<span class="em">—</span>a pointer to a dynamically allocated
array of pointers to upvalues. We also store the number of elements in
the array.

Storing the upvalue count in the closure is redundant because the
ObjFunction that the ObjClosure references also keeps that count. As
usual, this weird code is to appease the GC. The collector may need to
know an ObjClosure’s upvalue array size after the closure’s
corresponding ObjFunction has already been freed.

When we create an ObjClosure, we allocate an upvalue array of the proper
size, which we determined at compile time and stored in the ObjFunction.

<div class="codehilite">

``` insert-before
ObjClosure* newClosure(ObjFunction* function) {
```

<div class="source-file">

*object.c*  
in *newClosure*()

</div>

``` insert
  ObjUpvalue** upvalues = ALLOCATE(ObjUpvalue*,
                                   function->upvalueCount);
  for (int i = 0; i < function->upvalueCount; i++) {
    upvalues[i] = NULL;
  }
```

``` insert-after
  ObjClosure* closure = ALLOCATE_OBJ(ObjClosure, OBJ_CLOSURE);
```

</div>

<div class="source-file-narrow">

*object.c*, in *newClosure*()

</div>

Before creating the closure object itself, we allocate the array of
upvalues and initialize them all to `NULL`. This weird ceremony around
memory is a careful dance to please the (forthcoming) garbage collection
deities. It ensures the memory manager never sees uninitialized memory.

Then we store the array in the new closure, as well as copy the count
over from the ObjFunction.

<div class="codehilite">

``` insert-before
  closure->function = function;
```

<div class="source-file">

*object.c*  
in *newClosure*()

</div>

``` insert
  closure->upvalues = upvalues;
  closure->upvalueCount = function->upvalueCount;
```

``` insert-after
  return closure;
```

</div>

<div class="source-file-narrow">

*object.c*, in *newClosure*()

</div>

When we free an ObjClosure, we also free the upvalue array.

<div class="codehilite">

``` insert-before
    case OBJ_CLOSURE: {
```

<div class="source-file">

*memory.c*  
in *freeObject*()

</div>

``` insert
      ObjClosure* closure = (ObjClosure*)object;
      FREE_ARRAY(ObjUpvalue*, closure->upvalues,
                 closure->upvalueCount);
```

``` insert-after
      FREE(ObjClosure, object);
```

</div>

<div class="source-file-narrow">

*memory.c*, in *freeObject*()

</div>

ObjClosure does not own the ObjUpvalue objects themselves, but it does
own *the array* containing pointers to those upvalues.

We fill the upvalue array over in the interpreter when it creates a
closure. This is where we walk through all of the operands after
`OP_CLOSURE` to see what kind of upvalue each slot captures.

<div class="codehilite">

``` insert-before
        push(OBJ_VAL(closure));
```

<div class="source-file">

*vm.c*  
in *run*()

</div>

``` insert
        for (int i = 0; i < closure->upvalueCount; i++) {
          uint8_t isLocal = READ_BYTE();
          uint8_t index = READ_BYTE();
          if (isLocal) {
            closure->upvalues[i] =
                captureUpvalue(frame->slots + index);
          } else {
            closure->upvalues[i] = frame->closure->upvalues[index];
          }
        }
```

``` insert-after
        break;
```

</div>

<div class="source-file-narrow">

*vm.c*, in *run*()

</div>

This code is the magic moment when a closure comes to life. We iterate
over each upvalue the closure expects. For each one, we read a pair of
operand bytes. If the upvalue closes over a local variable in the
enclosing function, we let `captureUpvalue()` do the work.

Otherwise, we capture an upvalue from the surrounding function. An
`OP_CLOSURE` instruction is emitted at the end of a function
declaration. At the moment that we are executing that declaration, the
*current* function is the surrounding one. That means the current
function’s closure is stored in the CallFrame at the top of the
callstack. So, to grab an upvalue from the enclosing function, we can
read it right from the `frame` local variable, which caches a reference
to that CallFrame.

Closing over a local variable is more interesting. Most of the work
happens in a separate function, but first we calculate the argument to
pass to it. We need to grab a pointer to the captured local’s slot in
the surrounding function’s stack window. That window begins at
`frame->slots`, which points to slot zero. Adding `index` offsets that
to the local slot we want to capture. We pass that pointer here:

<div class="codehilite">

<div class="source-file">

*vm.c*  
add after *callValue*()

</div>

    static ObjUpvalue* captureUpvalue(Value* local) {
      ObjUpvalue* createdUpvalue = newUpvalue(local);
      return createdUpvalue;
    }

</div>

<div class="source-file-narrow">

*vm.c*, add after *callValue*()

</div>

This seems a little silly. All it does is create a new ObjUpvalue that
captures the given stack slot and returns it. Did we need a separate
function for this? Well, no, not *yet*. But you know we are going to end
up sticking more code in here.

First, let’s wrap up what we’re working on. Back in the interpreter code
for handling `OP_CLOSURE`, we eventually finish iterating through the
upvalue array and initialize each one. When that completes, we have a
new closure with an array full of upvalues pointing to variables.

With that in hand, we can implement the instructions that work with
those upvalues.

<div class="codehilite">

``` insert-before
      }
```

<div class="source-file">

*vm.c*  
in *run*()

</div>

``` insert
      case OP_GET_UPVALUE: {
        uint8_t slot = READ_BYTE();
        push(*frame->closure->upvalues[slot]->location);
        break;
      }
```

``` insert-after
      case OP_EQUAL: {
```

</div>

<div class="source-file-narrow">

*vm.c*, in *run*()

</div>

The operand is the index into the current function’s upvalue array. So
we simply look up the corresponding upvalue and dereference its location
pointer to read the value in that slot. Setting a variable is similar.

<div class="codehilite">

``` insert-before
      }
```

<div class="source-file">

*vm.c*  
in *run*()

</div>

``` insert
      case OP_SET_UPVALUE: {
        uint8_t slot = READ_BYTE();
        *frame->closure->upvalues[slot]->location = peek(0);
        break;
      }
```

``` insert-after
      case OP_EQUAL: {
```

</div>

<div class="source-file-narrow">

*vm.c*, in *run*()

</div>

We <span id="assign">take</span> the value on top of the stack and store
it into the slot pointed to by the chosen upvalue. Just as with the
instructions for local variables, it’s important that these instructions
are fast. User programs are constantly reading and writing variables, so
if that’s slow, everything is slow. And, as usual, the way we make them
fast is by keeping them simple. These two new instructions are pretty
good: no control flow, no complex arithmetic, just a couple of pointer
indirections and a `push()`.

The set instruction doesn’t *pop* the value from the stack because,
remember, assignment is an expression in Lox. So the result of the
assignment<span class="em">—</span>the assigned
value<span class="em">—</span>needs to remain on the stack for the
surrounding expression.

This is a milestone. As long as all of the variables remain on the
stack, we have working closures. Try this:

<div class="codehilite">

    fun outer() {
      var x = "outside";
      fun inner() {
        print x;
      }
      inner();
    }
    outer();

</div>

Run this, and it correctly prints “outside”.

## <a href="#closed-upvalues" id="closed-upvalues"><span
class="small">25 . 4</span>Closed Upvalues</a>

Of course, a key feature of closures is that they hold on to the
variable as long as needed, even after the function that declares the
variable has returned. Here’s another example that *should* work:

<div class="codehilite">

    fun outer() {
      var x = "outside";
      fun inner() {
        print x;
      }

      return inner;
    }

    var closure = outer();
    closure();

</div>

But if you run it right now<span class="ellipse"> . . . </span>who knows
what it does? At runtime, it will end up reading from a stack slot that
no longer contains the closed-over variable. Like I’ve mentioned a few
times, the crux of the issue is that variables in closures don’t have
stack semantics. That means we’ve got to hoist them off the stack when
the function where they were declared returns. This final section of the
chapter does that.

### <a href="#values-and-variables" id="values-and-variables"><span
class="small">25 . 4 . 1</span>Values and variables</a>

Before we get to writing code, I want to dig into an important semantic
point. Does a closure close over a *value* or a *variable?* This isn’t
purely an <span id="academic">academic</span> question. I’m not just
splitting hairs. Consider:

If Lox didn’t allow assignment, it *would* be an academic question.

<div class="codehilite">

    var globalSet;
    var globalGet;

    fun main() {
      var a = "initial";

      fun set() { a = "updated"; }
      fun get() { print a; }

      globalSet = set;
      globalGet = get;
    }

    main();
    globalSet();
    globalGet();

</div>

The outer `main()` function creates two closures and stores them in
<span id="global">global</span> variables so that they outlive the
execution of `main()` itself. Both of those closures capture the same
variable. The first closure assigns a new value to it and the second
closure reads the variable.

The fact that I’m using a couple of global variables isn’t significant.
I needed some way to return two values from a function, and without any
kind of collection type in Lox, my options were limited.

What does the call to `globalGet()` print? If closures capture *values*
then each closure gets its own copy of `a` with the value that `a` had
at the point in time that the closure’s function declaration executed.
The call to `globalSet()` will modify `set()`’s copy of `a`, but
`get()`’s copy will be unaffected. Thus, the call to `globalGet()` will
print “initial”.

If closures close over variables, then `get()` and `set()` will both
capture<span class="em">—</span>reference<span class="em">—</span>the
*same mutable variable*. When `set()` changes `a`, it changes the same
`a` that `get()` reads from. There is only one `a`. That, in turn,
implies the call to `globalGet()` will print “updated”.

Which is it? The answer for Lox and most other languages I know with
closures is the latter. Closures capture variables. You can think of
them as capturing *the place the value lives*. This is important to keep
in mind as we deal with closed-over variables that are no longer on the
stack. When a variable moves to the heap, we need to ensure that all
closures capturing that variable retain a reference to its *one* new
location. That way, when the variable is mutated, all closures see the
change.

### <a href="#closing-upvalues" id="closing-upvalues"><span
class="small">25 . 4 . 2</span>Closing upvalues</a>

We know that local variables always start out on the stack. This is
faster, and lets our single-pass compiler emit code before it discovers
the variable has been captured. We also know that closed-over variables
need to move to the heap if the closure outlives the function where the
captured variable is declared.

Following Lua, we’ll use **open upvalue** to refer to an upvalue that
points to a local variable still on the stack. When a variable moves to
the heap, we are *closing* the upvalue and the result is, naturally, a
**closed upvalue**. The two questions we need to answer are:

1.  Where on the heap does the closed-over variable go?

2.  When do we close the upvalue?

The answer to the first question is easy. We already have a convenient
object on the heap that represents a reference to a
variable<span class="em">—</span>ObjUpvalue itself. The closed-over
variable will move into a new field right inside the ObjUpvalue struct.
That way we don’t need to do any additional heap allocation to close an
upvalue.

The second question is straightforward too. As long as the variable is
on the stack, there may be code that refers to it there, and that code
must work correctly. So the logical time to hoist the variable to the
heap is as late as possible. If we move the local variable right when it
goes out of scope, we are certain that no code after that point will try
to access it from the stack. <span id="after">After</span> the variable
is out of scope, the compiler will have reported an error if any code
tried to use it.

By “after” here, I mean in the lexical or textual
sense<span class="em">—</span>code past the `}` for the block containing
the declaration of the closed-over variable.

The compiler already emits an `OP_POP` instruction when a local variable
goes out of scope. If a variable is captured by a closure, we will
instead emit a different instruction to hoist that variable out of the
stack and into its corresponding upvalue. To do that, the compiler needs
to know which <span id="param">locals</span> are closed over.

The compiler doesn’t pop parameters and locals declared immediately
inside the body of a function. We’ll handle those too, in the runtime.

The compiler already maintains an array of Upvalue structs for each
local variable in the function to track exactly that state. That array
is good for answering “Which variables does this closure use?” But it’s
poorly suited for answering, “Does *any* function capture this local
variable?” In particular, once the Compiler for some closure has
finished, the Compiler for the enclosing function whose variable has
been captured no longer has access to any of the upvalue state.

In other words, the compiler maintains pointers from upvalues to the
locals they capture, but not in the other direction. So we first need to
add some extra tracking inside the existing Local struct so that we can
tell if a given local is captured by a closure.

<div class="codehilite">

``` insert-before
  int depth;
```

<div class="source-file">

*compiler.c*  
in struct *Local*

</div>

``` insert
  bool isCaptured;
```

``` insert-after
} Local;
```

</div>

<div class="source-file-narrow">

*compiler.c*, in struct *Local*

</div>

This field is `true` if the local is captured by any later nested
function declaration. Initially, all locals are not captured.

<div class="codehilite">

``` insert-before
  local->depth = -1;
```

<div class="source-file">

*compiler.c*  
in *addLocal*()

</div>

``` insert
  local->isCaptured = false;
```

``` insert-after
}
```

</div>

<div class="source-file-narrow">

*compiler.c*, in *addLocal*()

</div>

<span id="zero">Likewise</span>, the special “slot zero local” that the
compiler implicitly declares is not captured.

Later in the book, it *will* become possible for a user to capture this
variable. Just building some anticipation here.

<div class="codehilite">

``` insert-before
  local->depth = 0;
```

<div class="source-file">

*compiler.c*  
in *initCompiler*()

</div>

``` insert
  local->isCaptured = false;
```

``` insert-after
  local->name.start = "";
```

</div>

<div class="source-file-narrow">

*compiler.c*, in *initCompiler*()

</div>

When resolving an identifier, if we end up creating an upvalue for a
local variable, we mark it as captured.

<div class="codehilite">

``` insert-before
  if (local != -1) {
```

<div class="source-file">

*compiler.c*  
in *resolveUpvalue*()

</div>

``` insert
    compiler->enclosing->locals[local].isCaptured = true;
```

``` insert-after
    return addUpvalue(compiler, (uint8_t)local, true);
```

</div>

<div class="source-file-narrow">

*compiler.c*, in *resolveUpvalue*()

</div>

Now, at the end of a block scope when the compiler emits code to free
the stack slots for the locals, we can tell which ones need to get
hoisted onto the heap. We’ll use a new instruction for that.

<div class="codehilite">

``` insert-before
  while (current->localCount > 0 &&
         current->locals[current->localCount - 1].depth >
            current->scopeDepth) {
```

<div class="source-file">

*compiler.c*  
in *endScope*()  
replace 1 line

</div>

``` insert
    if (current->locals[current->localCount - 1].isCaptured) {
      emitByte(OP_CLOSE_UPVALUE);
    } else {
      emitByte(OP_POP);
    }
```

``` insert-after
    current->localCount--;
  }
```

</div>

<div class="source-file-narrow">

*compiler.c*, in *endScope*(), replace 1 line

</div>

The instruction requires no operand. We know that the variable will
always be right on top of the stack at the point that this instruction
executes. We declare the instruction.

<div class="codehilite">

``` insert-before
  OP_CLOSURE,
```

<div class="source-file">

*chunk.h*  
in enum *OpCode*

</div>

``` insert
  OP_CLOSE_UPVALUE,
```

``` insert-after
  OP_RETURN,
```

</div>

<div class="source-file-narrow">

*chunk.h*, in enum *OpCode*

</div>

And add trivial disassembler support for it:

<div class="codehilite">

``` insert-before
    }
```

<div class="source-file">

*debug.c*  
in *disassembleInstruction*()

</div>

``` insert
    case OP_CLOSE_UPVALUE:
      return simpleInstruction("OP_CLOSE_UPVALUE", offset);
```

``` insert-after
    case OP_RETURN:
```

</div>

<div class="source-file-narrow">

*debug.c*, in *disassembleInstruction*()

</div>

Excellent. Now the generated bytecode tells the runtime exactly when
each captured local variable must move to the heap. Better, it does so
only for the locals that *are* used by a closure and need this special
treatment. This aligns with our general performance goal that we want
users to pay only for functionality that they use. Variables that aren’t
used by closures live and die entirely on the stack just as they did
before.

### <a href="#tracking-open-upvalues" id="tracking-open-upvalues"><span
class="small">25 . 4 . 3</span>Tracking open upvalues</a>

Let’s move over to the runtime side. Before we can interpret
`OP_CLOSE_UPVALUE` instructions, we have an issue to resolve. Earlier,
when I talked about whether closures capture variables or values, I said
it was important that if multiple closures access the same variable that
they end up with a reference to the exact same storage location in
memory. That way if one closure writes to the variable, the other
closure sees the change.

Right now, if two closures capture the same
<span id="indirect">local</span> variable, the VM creates a separate
Upvalue for each one. The necessary sharing is missing. When we move the
variable off the stack, if we move it into only one of the upvalues, the
other upvalue will have an orphaned value.

The VM *does* share upvalues if one closure captures an *upvalue* from a
surrounding function. The nested case works correctly. But if two
*sibling* closures capture the same local variable, they each create a
separate ObjUpvalue.

To fix that, whenever the VM needs an upvalue that captures a particular
local variable slot, we will first search for an existing upvalue
pointing to that slot. If found, we reuse that. The challenge is that
all of the previously created upvalues are squirreled away inside the
upvalue arrays of the various closures. Those closures could be anywhere
in the VM’s memory.

The first step is to give the VM its own list of all open upvalues that
point to variables still on the stack. Searching a list each time the VM
needs an upvalue sounds like it might be slow, but in practice, it’s not
bad. The number of variables on the stack that actually get closed over
tends to be small. And function declarations that
<span id="create">create</span> closures are rarely on performance
critical execution paths in the user’s program.

Closures are frequently *invoked* inside hot loops. Think about the
closures passed to typical higher-order functions on collections like
[`map()`](https://en.wikipedia.org/wiki/Map_(higher-order_function)) and
[`filter()`](https://en.wikipedia.org/wiki/Filter_(higher-order_function)).
That should be fast. But the function declaration that *creates* the
closure happens only once and is usually outside of the loop.

Even better, we can order the list of open upvalues by the stack slot
index they point to. The common case is that a slot has *not* already
been captured<span class="em">—</span>sharing variables between closures
is uncommon<span class="em">—</span>and closures tend to capture locals
near the top of the stack. If we store the open upvalue array in stack
slot order, as soon as we step past the slot where the local we’re
capturing lives, we know it won’t be found. When that local is near the
top of the stack, we can exit the loop pretty early.

Maintaining a sorted list requires inserting elements in the middle
efficiently. That suggests using a linked list instead of a dynamic
array. Since we defined the ObjUpvalue struct ourselves, the easiest
implementation is an intrusive list that puts the next pointer right
inside the ObjUpvalue struct itself.

<div class="codehilite">

``` insert-before
  Value* location;
```

<div class="source-file">

*object.h*  
in struct *ObjUpvalue*

</div>

``` insert
  struct ObjUpvalue* next;
```

``` insert-after
} ObjUpvalue;
```

</div>

<div class="source-file-narrow">

*object.h*, in struct *ObjUpvalue*

</div>

When we allocate an upvalue, it is not attached to any list yet so the
link is `NULL`.

<div class="codehilite">

``` insert-before
  upvalue->location = slot;
```

<div class="source-file">

*object.c*  
in *newUpvalue*()

</div>

``` insert
  upvalue->next = NULL;
```

``` insert-after
  return upvalue;
```

</div>

<div class="source-file-narrow">

*object.c*, in *newUpvalue*()

</div>

The VM owns the list, so the head pointer goes right inside the main VM
struct.

<div class="codehilite">

``` insert-before
  Table strings;
```

<div class="source-file">

*vm.h*  
in struct *VM*

</div>

``` insert
  ObjUpvalue* openUpvalues;
```

``` insert-after
  Obj* objects;
```

</div>

<div class="source-file-narrow">

*vm.h*, in struct *VM*

</div>

The list starts out empty.

<div class="codehilite">

``` insert-before
  vm.frameCount = 0;
```

<div class="source-file">

*vm.c*  
in *resetStack*()

</div>

``` insert
  vm.openUpvalues = NULL;
```

``` insert-after
}
```

</div>

<div class="source-file-narrow">

*vm.c*, in *resetStack*()

</div>

Starting with the first upvalue pointed to by the VM, each open upvalue
points to the next open upvalue that references a local variable farther
down the stack. This script, for example,

<div class="codehilite">

    {
      var a = 1;
      fun f() {
        print a;
      }
      var b = 2;
      fun g() {
        print b;
      }
      var c = 3;
      fun h() {
        print c;
      }
    }

</div>

should produce a series of linked upvalues like so:

![Three upvalues in a linked list.](image/closures/linked-list.png)

Whenever we close over a local variable, before creating a new upvalue,
we look for an existing one in the list.

<div class="codehilite">

``` insert-before
static ObjUpvalue* captureUpvalue(Value* local) {
```

<div class="source-file">

*vm.c*  
in *captureUpvalue*()

</div>

``` insert
  ObjUpvalue* prevUpvalue = NULL;
  ObjUpvalue* upvalue = vm.openUpvalues;
  while (upvalue != NULL && upvalue->location > local) {
    prevUpvalue = upvalue;
    upvalue = upvalue->next;
  }

  if (upvalue != NULL && upvalue->location == local) {
    return upvalue;
  }
```

``` insert-after
  ObjUpvalue* createdUpvalue = newUpvalue(local);
```

</div>

<div class="source-file-narrow">

*vm.c*, in *captureUpvalue*()

</div>

We start at the <span id="head">head</span> of the list, which is the
upvalue closest to the top of the stack. We walk through the list, using
a little pointer comparison to iterate past every upvalue pointing to
slots above the one we’re looking for. While we do that, we keep track
of the preceding upvalue on the list. We’ll need to update that node’s
`next` pointer if we end up inserting a node after it.

It’s a singly linked list. It’s not like we have any other choice than
to start at the head and go forward from there.

There are three reasons we can exit the loop:

1.  **The local slot we stopped at *is* the slot we’re looking for.** We
    found an existing upvalue capturing the variable, so we reuse that
    upvalue.

2.  **We ran out of upvalues to search.** When `upvalue` is `NULL`, it
    means every open upvalue in the list points to locals above the slot
    we’re looking for, or (more likely) the upvalue list is empty.
    Either way, we didn’t find an upvalue for our slot.

3.  **We found an upvalue whose local slot is *below* the one we’re
    looking for.** Since the list is sorted, that means we’ve gone past
    the slot we are closing over, and thus there must not be an existing
    upvalue for it.

In the first case, we’re done and we’ve returned. Otherwise, we create a
new upvalue for our local slot and insert it into the list at the right
location.

<div class="codehilite">

``` insert-before
  ObjUpvalue* createdUpvalue = newUpvalue(local);
```

<div class="source-file">

*vm.c*  
in *captureUpvalue*()

</div>

``` insert
  createdUpvalue->next = upvalue;

  if (prevUpvalue == NULL) {
    vm.openUpvalues = createdUpvalue;
  } else {
    prevUpvalue->next = createdUpvalue;
  }
```

``` insert-after
  return createdUpvalue;
```

</div>

<div class="source-file-narrow">

*vm.c*, in *captureUpvalue*()

</div>

The current incarnation of this function already creates the upvalue, so
we only need to add code to insert the upvalue into the list. We exited
the list traversal by either going past the end of the list, or by
stopping on the first upvalue whose stack slot is below the one we’re
looking for. In either case, that means we need to insert the new
upvalue *before* the object pointed at by `upvalue` (which may be `NULL`
if we hit the end of the list).

As you may have learned in Data Structures 101, to insert a node into a
linked list, you set the `next` pointer of the previous node to point to
your new one. We have been conveniently keeping track of that preceding
node as we walked the list. We also need to handle the
<span id="double">special</span> case where we are inserting a new
upvalue at the head of the list, in which case the “next” pointer is the
VM’s head pointer.

There is a shorter implementation that handles updating either the head
pointer or the previous upvalue’s `next` pointer uniformly by using a
pointer to a pointer, but that kind of code confuses almost everyone who
hasn’t reached some Zen master level of pointer expertise. I went with
the basic `if` statement approach.

With this updated function, the VM now ensures that there is only ever a
single ObjUpvalue for any given local slot. If two closures capture the
same variable, they will get the same upvalue. We’re ready to move those
upvalues off the stack now.

### <a href="#closing-upvalues-at-runtime"
id="closing-upvalues-at-runtime"><span
class="small">25 . 4 . 4</span>Closing upvalues at runtime</a>

The compiler helpfully emits an `OP_CLOSE_UPVALUE` instruction to tell
the VM exactly when a local variable should be hoisted onto the heap.
Executing that instruction is the interpreter’s responsibility.

<div class="codehilite">

``` insert-before
      }
```

<div class="source-file">

*vm.c*  
in *run*()

</div>

``` insert
      case OP_CLOSE_UPVALUE:
        closeUpvalues(vm.stackTop - 1);
        pop();
        break;
```

``` insert-after
      case OP_RETURN: {
```

</div>

<div class="source-file-narrow">

*vm.c*, in *run*()

</div>

When we reach the instruction, the variable we are hoisting is right on
top of the stack. We call a helper function, passing the address of that
stack slot. That function is responsible for closing the upvalue and
moving the local from the stack to the heap. After that, the VM is free
to discard the stack slot, which it does by calling `pop()`.

The fun stuff happens here:

<div class="codehilite">

<div class="source-file">

*vm.c*  
add after *captureUpvalue*()

</div>

    static void closeUpvalues(Value* last) {
      while (vm.openUpvalues != NULL &&
             vm.openUpvalues->location >= last) {
        ObjUpvalue* upvalue = vm.openUpvalues;
        upvalue->closed = *upvalue->location;
        upvalue->location = &upvalue->closed;
        vm.openUpvalues = upvalue->next;
      }
    }

</div>

<div class="source-file-narrow">

*vm.c*, add after *captureUpvalue*()

</div>

This function takes a pointer to a stack slot. It closes every open
upvalue it can find that points to that slot or any slot above it on the
stack. Right now, we pass a pointer only to the top slot on the stack,
so the “or above it” part doesn’t come into play, but it will soon.

To do this, we walk the VM’s list of open upvalues, again from top to
bottom. If an upvalue’s location points into the range of slots we’re
closing, we close the upvalue. Otherwise, once we reach an upvalue
outside of the range, we know the rest will be too, so we stop
iterating.

The way an upvalue gets closed is pretty <span id="cool">cool</span>.
First, we copy the variable’s value into the `closed` field in the
ObjUpvalue. That’s where closed-over variables live on the heap. The
`OP_GET_UPVALUE` and `OP_SET_UPVALUE` instructions need to look for the
variable there after it’s been moved. We could add some conditional
logic in the interpreter code for those instructions to check some flag
for whether the upvalue is open or closed.

But there is already a level of indirection in
play<span class="em">—</span>those instructions dereference the
`location` pointer to get to the variable’s value. When the variable
moves from the stack to the `closed` field, we simply update that
`location` to the address of the ObjUpvalue’s *own* `closed` field.

I’m not praising myself here. This is all the Lua dev team’s innovation.

![Moving a value from the stack to the upvalue's 'closed' field and then
pointing the 'value' field to it.](image/closures/closing.png)

We don’t need to change how `OP_GET_UPVALUE` and `OP_SET_UPVALUE` are
interpreted at all. That keeps them simple, which in turn keeps them
fast. We do need to add the new field to ObjUpvalue, though.

<div class="codehilite">

``` insert-before
  Value* location;
```

<div class="source-file">

*object.h*  
in struct *ObjUpvalue*

</div>

``` insert
  Value closed;
```

``` insert-after
  struct ObjUpvalue* next;
```

</div>

<div class="source-file-narrow">

*object.h*, in struct *ObjUpvalue*

</div>

And we should zero it out when we create an ObjUpvalue so there’s no
uninitialized memory floating around.

<div class="codehilite">

``` insert-before
  ObjUpvalue* upvalue = ALLOCATE_OBJ(ObjUpvalue, OBJ_UPVALUE);
```

<div class="source-file">

*object.c*  
in *newUpvalue*()

</div>

``` insert
  upvalue->closed = NIL_VAL;
```

``` insert-after
  upvalue->location = slot;
```

</div>

<div class="source-file-narrow">

*object.c*, in *newUpvalue*()

</div>

Whenever the compiler reaches the end of a block, it discards all local
variables in that block and emits an `OP_CLOSE_UPVALUE` for each local
variable that was closed over. The compiler <span id="close">does</span>
*not* emit any instructions at the end of the outermost block scope that
defines a function body. That scope contains the function’s parameters
and any locals declared immediately inside the function. Those need to
get closed too.

There’s nothing *preventing* us from closing the outermost function
scope in the compiler and emitting `OP_POP` and `OP_CLOSE_UPVALUE`
instructions. Doing so is just unnecessary because the runtime discards
all of the stack slots used by the function implicitly when it pops the
call frame.

This is the reason `closeUpvalues()` accepts a pointer to a stack slot.
When a function returns, we call that same helper and pass in the first
stack slot owned by the function.

<div class="codehilite">

``` insert-before
        Value result = pop();
```

<div class="source-file">

*vm.c*  
in *run*()

</div>

``` insert
        closeUpvalues(frame->slots);
```

``` insert-after
        vm.frameCount--;
```

</div>

<div class="source-file-narrow">

*vm.c*, in *run*()

</div>

By passing the first slot in the function’s stack window, we close every
remaining open upvalue owned by the returning function. And with that,
we now have a fully functioning closure implementation. Closed-over
variables live as long as they are needed by the functions that capture
them.

This was a lot of work! In jlox, closures fell out naturally from our
environment representation. In clox, we had to add a lot of
code<span class="em">—</span>new bytecode instructions, more data
structures in the compiler, and new runtime objects. The VM very much
treats variables in closures as different from other variables.

There is a rationale for that. In terms of implementation complexity,
jlox gave us closures “for free”. But in terms of *performance*, jlox’s
closures are anything but. By allocating *all* environments on the heap,
jlox pays a significant performance price for *all* local variables,
even the majority which are never captured by closures.

With clox, we have a more complex system, but that allows us to tailor
the implementation to fit the two use patterns we observe for local
variables. For most variables which do have stack semantics, we allocate
them entirely on the stack which is simple and fast. Then, for the few
local variables where that doesn’t work, we have a second slower path we
can opt in to as needed.

Fortunately, users don’t perceive the complexity. From their
perspective, local variables in Lox are simple and uniform. The
*language itself* is as simple as jlox’s implementation. But under the
hood, clox is watching what the user does and optimizing for their
specific uses. As your language implementations grow in sophistication,
you’ll find yourself doing this more. A large fraction of “optimization”
is about adding special case code that detects certain uses and provides
a custom-built, faster path for code that fits that pattern.

We have lexical scoping fully working in clox now, which is a major
milestone. And, now that we have functions and variables with complex
lifetimes, we also have a *lot* of objects floating around in clox’s
heap, with a web of pointers stringing them together. The [next
step](garbage-collection.html) is figuring out how to manage that memory
so that we can free some of those objects when they’re no longer needed.

<div class="challenges">

## <a href="#challenges" id="challenges">Challenges</a>

1.  Wrapping every ObjFunction in an ObjClosure introduces a level of
    indirection that has a performance cost. That cost isn’t necessary
    for functions that do not close over any variables, but it does let
    the runtime treat all calls uniformly.

    Change clox to only wrap functions in ObjClosures that need
    upvalues. How does the code complexity and performance compare to
    always wrapping functions? Take care to benchmark programs that do
    and do not use closures. How should you weight the importance of
    each benchmark? If one gets slower and one faster, how do you decide
    what trade-off to make to choose an implementation strategy?

2.  Read the design note below. I’ll wait. Now, how do you think Lox
    *should* behave? Change the implementation to create a new variable
    for each loop iteration.

3.  A [famous koan](http://wiki.c2.com/?ClosuresAndObjectsAreEquivalent)
    teaches us that “objects are a poor man’s closure” (and vice versa).
    Our VM doesn’t support objects yet, but now that we have closures we
    can approximate them. Using closures, write a Lox program that
    models two-dimensional vector “objects”. It should:

    - Define a “constructor” function to create a new vector with the
      given *x* and *y* coordinates.

    - Provide “methods” to access the *x* and *y* coordinates of values
      returned from that constructor.

    - Define an addition “method” that adds two vectors and produces a
      third.

</div>

<div class="design-note">

## <a href="#design-note" id="design-note">Design Note: Closing Over the
Loop Variable</a>

Closures capture variables. When two closures capture the same variable,
they share a reference to the same underlying storage location. This
fact is visible when new values are assigned to the variable. Obviously,
if two closures capture *different* variables, there is no sharing.

<div class="codehilite">

    var globalOne;
    var globalTwo;

    fun main() {
      {
        var a = "one";
        fun one() {
          print a;
        }
        globalOne = one;
      }

      {
        var a = "two";
        fun two() {
          print a;
        }
        globalTwo = two;
      }
    }

    main();
    globalOne();
    globalTwo();

</div>

This prints “one” then “two”. In this example, it’s pretty clear that
the two `a` variables are different. But it’s not always so obvious.
Consider:

<div class="codehilite">

    var globalOne;
    var globalTwo;

    fun main() {
      for (var a = 1; a <= 2; a = a + 1) {
        fun closure() {
          print a;
        }
        if (globalOne == nil) {
          globalOne = closure;
        } else {
          globalTwo = closure;
        }
      }
    }

    main();
    globalOne();
    globalTwo();

</div>

The code is convoluted because Lox has no collection types. The
important part is that the `main()` function does two iterations of a
`for` loop. Each time through the loop, it creates a closure that
captures the loop variable. It stores the first closure in `globalOne`
and the second in `globalTwo`.

There are definitely two different closures. Do they close over two
different variables? Is there only one `a` for the entire duration of
the loop, or does each iteration get its own distinct `a` variable?

The script here is strange and contrived, but this does show up in real
code in languages that aren’t as minimal as clox. Here’s a JavaScript
example:

<div class="codehilite">

    var closures = [];
    for (var i = 1; i <= 2; i++) {
      closures.push(function () { console.log(i); });
    }

    closures[0]();
    closures[1]();

</div>

Does this print “1” then “2”, or does it print
<span id="three">“3”</span> twice? You may be surprised to hear that it
prints “3” twice. In this JavaScript program, there is only a single `i`
variable whose lifetime includes all iterations of the loop, including
the final exit.

You’re wondering how *three* enters the picture? After the second
iteration, `i++` is executed, which increments `i` to three. That’s what
causes `i <= 2` to evaluate to false and end the loop. If `i` never
reached three, the loop would run forever.

If you’re familiar with JavaScript, you probably know that variables
declared using `var` are implicitly *hoisted* to the surrounding
function or top-level scope. It’s as if you really wrote this:

<div class="codehilite">

    var closures = [];
    var i;
    for (i = 1; i <= 2; i++) {
      closures.push(function () { console.log(i); });
    }

    closures[0]();
    closures[1]();

</div>

At that point, it’s clearer that there is only a single `i`. Now
consider if you change the program to use the newer `let` keyword:

<div class="codehilite">

    var closures = [];
    for (let i = 1; i <= 2; i++) {
      closures.push(function () { console.log(i); });
    }

    closures[0]();
    closures[1]();

</div>

Does this new program behave the same? Nope. In this case, it prints “1”
then “2”. Each closure gets its own `i`. That’s sort of strange when you
think about it. The increment clause is `i++`. That looks very much like
it is assigning to and mutating an existing variable, not creating a new
one.

Let’s try some other languages. Here’s Python:

<div class="codehilite">

    closures = []
    for i in range(1, 3):
      closures.append(lambda: print(i))

    closures[0]()
    closures[1]()

</div>

Python doesn’t really have block scope. Variables are implicitly
declared and are automatically scoped to the surrounding function. Kind
of like hoisting in JS, now that I think about it. So both closures
capture the same variable. Unlike C, though, we don’t exit the loop by
incrementing `i` *past* the last value, so this prints “2” twice.

What about Ruby? Ruby has two typical ways to iterate numerically.
Here’s the classic imperative style:

<div class="codehilite">

    closures = []
    for i in 1..2 do
      closures << lambda { puts i }
    end

    closures[0].call
    closures[1].call

</div>

This, like Python, prints “2” twice. But the more idiomatic Ruby style
is using a higher-order `each()` method on range objects:

<div class="codehilite">

    closures = []
    (1..2).each do |i|
      closures << lambda { puts i }
    end

    closures[0].call
    closures[1].call

</div>

If you’re not familiar with Ruby, the `do |i| ... end` part is basically
a closure that gets created and passed to the `each()` method. The `|i|`
is the parameter signature for the closure. The `each()` method invokes
that closure twice, passing in 1 for `i` the first time and 2 the second
time.

In this case, the “loop variable” is really a function parameter. And,
since each iteration of the loop is a separate invocation of the
function, those are definitely separate variables for each call. So this
prints “1” then “2”.

If a language has a higher-level iterator-based looping structure like
`foreach` in C#, Java’s “enhanced for”, `for-of` in JavaScript, `for-in`
in Dart, etc., then I think it’s natural to the reader to have each
iteration create a new variable. The code *looks* like a new variable
because the loop header looks like a variable declaration. And there’s
no increment expression that looks like it’s mutating that variable to
advance to the next step.

If you dig around StackOverflow and other places, you find evidence that
this is what users expect, because they are very surprised when they
*don’t* get it. In particular, C# originally did *not* create a new loop
variable for each iteration of a `foreach` loop. This was such a
frequent source of user confusion that they took the very rare step of
shipping a breaking change to the language. In C# 5, each iteration
creates a fresh variable.

Old C-style `for` loops are harder. The increment clause really does
look like mutation. That implies there is a single variable that’s
getting updated each step. But it’s almost never *useful* for each
iteration to share a loop variable. The only time you can even detect
this is when closures capture it. And it’s rarely helpful to have a
closure that references a variable whose value is whatever value caused
you to exit the loop.

The pragmatically useful answer is probably to do what JavaScript does
with `let` in `for` loops. Make it look like mutation but actually
create a new variable each time, because that’s what users want. It is
kind of weird when you think about it, though.

</div>

<a href="garbage-collection.html" class="next">Next Chapter: “Garbage
Collection” →</a> Handcrafted by Robert Nystrom — <a
href="https://github.com/munificent/craftinginterpreters/blob/master/LICENSE"
target="_blank">© 2015 – 2021</a>

</div>
