[![](image/logotype.png "Crafting Interpreters")](/)

<div class="contents">

### [Chunks of Bytecode<span class="small">14</span>](#top)

- [<span class="small">14.1</span> Bytecode?](#bytecode)
- [<span class="small">14.2</span> Getting Started](#getting-started)
- [<span class="small">14.3</span> Chunks of
  Instructions](#chunks-of-instructions)
- [<span class="small">14.4</span> Disassembling
  Chunks](#disassembling-chunks)
- [<span class="small">14.5</span> Constants](#constants)
- [<span class="small">14.6</span> Line Information](#line-information)
- 
- [Challenges](#challenges)
- [<span class="small">note</span>Test Your Language](#design-note)

<div class="prev-next">

<a href="a-bytecode-virtual-machine.html" class="left"
title="A Bytecode Virtual Machine">← Previous</a>
[↑ Up](a-bytecode-virtual-machine.html "A Bytecode Virtual Machine")
<a href="a-virtual-machine.html" class="right"
title="A Virtual Machine">Next →</a>

</div>

</div>

[![](image/logotype.png "Crafting Interpreters")](/)
<a href="a-bytecode-virtual-machine.html" class="prev"
title="A Bytecode Virtual Machine">←</a>
<a href="a-virtual-machine.html" class="next"
title="A Virtual Machine">→</a>

<div class="page">

<div class="nav-wrapper">

[![](image/logotype.png "Crafting Interpreters")](/)

<div class="expandable">

### [Chunks of Bytecode<span class="small">14</span>](#top)

- [<span class="small">14.1</span> Bytecode?](#bytecode)
- [<span class="small">14.2</span> Getting Started](#getting-started)
- [<span class="small">14.3</span> Chunks of
  Instructions](#chunks-of-instructions)
- [<span class="small">14.4</span> Disassembling
  Chunks](#disassembling-chunks)
- [<span class="small">14.5</span> Constants](#constants)
- [<span class="small">14.6</span> Line Information](#line-information)
- 
- [Challenges](#challenges)
- [<span class="small">note</span>Test Your Language](#design-note)

<div class="prev-next">

<a href="a-bytecode-virtual-machine.html" class="left"
title="A Bytecode Virtual Machine">← Previous</a>
[↑ Up](a-bytecode-virtual-machine.html "A Bytecode Virtual Machine")
<a href="a-virtual-machine.html" class="right"
title="A Virtual Machine">Next →</a>

</div>

</div>

<span id="expand-nav">≡</span>

</div>

<div class="number">

14

</div>

# Chunks of Bytecode

> If you find that you’re spending almost all your time on theory, start
> turning some attention to practical things; it will improve your
> theories. If you find that you’re spending almost all your time on
> practice, start turning some attention to theoretical things; it will
> improve your practice.
>
> Donald Knuth

We already have ourselves a complete implementation of Lox with jlox, so
why isn’t the book over yet? Part of this is because jlox relies on the
<span id="metal">JVM</span> to do lots of things for us. If we want to
understand how an interpreter works all the way down to the metal, we
need to build those bits and pieces ourselves.

Of course, our second interpreter relies on the C standard library for
basics like memory allocation, and the C compiler frees us from details
of the underlying machine code we’re running it on. Heck, that machine
code is probably implemented in terms of microcode on the chip. And the
C runtime relies on the operating system to hand out pages of memory.
But we have to stop *somewhere* if this book is going to fit on your
bookshelf.

An even more fundamental reason that jlox isn’t sufficient is that it’s
too damn slow. A tree-walk interpreter is fine for some kinds of
high-level, declarative languages. But for a general-purpose, imperative
language<span class="em">—</span>even a “scripting” language like
Lox<span class="em">—</span>it won’t fly. Take this little script:

<div class="codehilite">

    fun fib(n) {
      if (n < 2) return n;
      return fib(n - 1) + fib(n - 2); 
    }

    var before = clock();
    print fib(40);
    var after = clock();
    print after - before;

</div>

This is a comically inefficient way to actually calculate Fibonacci
numbers. Our goal is to see how fast the *interpreter* runs, not to see
how fast of a program we can write. A slow program that does a lot of
work<span class="em">—</span>pointless or not<span class="em">—</span>is
a good test case for that.

On my laptop, that takes jlox about 72 seconds to execute. An equivalent
C program finishes in half a second. Our dynamically typed scripting
language is never going to be as fast as a statically typed language
with manual memory management, but we don’t need to settle for more than
*two orders of magnitude* slower.

We could take jlox and run it in a profiler and start tuning and
tweaking hotspots, but that will only get us so far. The execution
model<span class="em">—</span>walking the AST<span class="em">—</span>is
fundamentally the wrong design. We can’t micro-optimize that to the
performance we want any more than you can polish an AMC Gremlin into an
SR-71 Blackbird.

We need to rethink the core model. This chapter introduces that model,
bytecode, and begins our new interpreter, clox.

## <a href="#bytecode" id="bytecode"><span
class="small">14 . 1</span>Bytecode?</a>

In engineering, few choices are without trade-offs. To best understand
why we’re going with bytecode, let’s stack it up against a couple of
alternatives.

### <a href="#why-not-walk-the-ast" id="why-not-walk-the-ast"><span
class="small">14 . 1 . 1</span>Why not walk the AST?</a>

Our existing interpreter has a couple of things going for it:

- Well, first, we already wrote it. It’s done. And the main reason it’s
  done is because this style of interpreter is *really simple to
  implement*. The runtime representation of the code directly maps to
  the syntax. It’s virtually effortless to get from the parser to the
  data structures we need at runtime.

- It’s *portable*. Our current interpreter is written in Java and runs
  on any platform Java supports. We could write a new implementation in
  C using the same approach and compile and run our language on
  basically every platform under the sun.

Those are real advantages. But, on the other hand, it’s *not
memory-efficient*. Each piece of syntax becomes an AST node. A tiny Lox
expression like `1 + 2` turns into a slew of objects with lots of
pointers between them, something like:

<span id="header"></span>

The “(header)” parts are the bookkeeping information the Java virtual
machine uses to support memory management and store the object’s type.
Those take up space too!

![The tree of Java objects created to represent '1 +
2'.](image/chunks-of-bytecode/ast.png)

Each of those pointers adds an extra 32 or 64 bits of overhead to the
object. Worse, sprinkling our data across the heap in a loosely
connected web of objects does bad things for
<span id="locality">*spatial locality*</span>.

I wrote [an entire
chapter](http://gameprogrammingpatterns.com/data-locality.html) about
this exact problem in my first book, *Game Programming Patterns*, if you
want to really dig in.

Modern CPUs process data way faster than they can pull it from RAM. To
compensate for that, chips have multiple layers of caching. If a piece
of memory it needs is already in the cache, it can be loaded more
quickly. We’re talking upwards of 100 *times* faster.

How does data get into that cache? The machine speculatively stuffs
things in there for you. Its heuristic is pretty simple. Whenever the
CPU reads a bit of data from RAM, it pulls in a whole little bundle of
adjacent bytes and stuffs them in the cache.

If our program next requests some data close enough to be inside that
cache line, our CPU runs like a well-oiled conveyor belt in a factory.
We *really* want to take advantage of this. To use the cache
effectively, the way we represent code in memory should be dense and
ordered like it’s read.

Now look up at that tree. Those sub-objects could be
<span id="anywhere">*anywhere*</span>. Every step the tree-walker takes
where it follows a reference to a child node may step outside the bounds
of the cache and force the CPU to stall until a new lump of data can be
slurped in from RAM. Just the *overhead* of those tree nodes with all of
their pointer fields and object headers tends to push objects away from
each other and out of the cache.

Even if the objects happened to be allocated in sequential memory when
the parser first produced them, after a couple of rounds of garbage
collection<span class="em">—</span>which may move objects around in
memory<span class="em">—</span>there’s no telling where they’ll be.

Our AST walker has other overhead too around interface dispatch and the
Visitor pattern, but the locality issues alone are enough to justify a
better code representation.

### <a href="#why-not-compile-to-native-code"
id="why-not-compile-to-native-code"><span
class="small">14 . 1 . 2</span>Why not compile to native code?</a>

If you want to go *real* fast, you want to get all of those layers of
indirection out of the way. Right down to the metal. Machine code. It
even *sounds* fast. *Machine code.*

Compiling directly to the native instruction set the chip supports is
what the fastest languages do. Targeting native code has been the most
efficient option since way back in the early days when engineers
actually <span id="hand">handwrote</span> programs in machine code.

Yes, they actually wrote machine code by hand. On punched cards. Which,
presumably, they punched *with their fists*.

If you’ve never written any machine code, or its slightly more
human-palatable cousin assembly code before, I’ll give you the gentlest
of introductions. Native code is a dense series of operations, encoded
directly in binary. Each instruction is between one and a few bytes
long, and is almost mind-numbingly low level. “Move a value from this
address to this register.” “Add the integers in these two registers.”
Stuff like that.

The CPU cranks through the instructions, decoding and executing each one
in order. There is no tree structure like our AST, and control flow is
handled by jumping from one point in the code directly to another. No
indirection, no overhead, no unnecessary skipping around or chasing
pointers.

Lightning fast, but that performance comes at a cost. First of all,
compiling to native code ain’t easy. Most chips in wide use today have
sprawling Byzantine architectures with heaps of instructions that
accreted over decades. They require sophisticated register allocation,
pipelining, and instruction scheduling.

And, of course, you’ve thrown <span id="back">portability</span> out.
Spend a few years mastering some architecture and that still only gets
you onto *one* of the several popular instruction sets out there. To get
your language on all of them, you need to learn all of their instruction
sets and write a separate back end for each one.

The situation isn’t entirely dire. A well-architected compiler lets you
share the front end and most of the middle layer optimization passes
across the different architectures you support. It’s mainly the code
generation and some of the details around instruction selection that
you’ll need to write afresh each time.

The [LLVM](https://llvm.org/) project gives you some of this out of the
box. If your compiler outputs LLVM’s own special intermediate language,
LLVM in turn compiles that to native code for a plethora of
architectures.

### <a href="#what-is-bytecode" id="what-is-bytecode"><span
class="small">14 . 1 . 3</span>What is bytecode?</a>

Fix those two points in your mind. On one end, a tree-walk interpreter
is simple, portable, and slow. On the other, native code is complex and
platform-specific but fast. Bytecode sits in the middle. It retains the
portability of a tree-walker<span class="em">—</span>we won’t be getting
our hands dirty with assembly code in this book. It sacrifices *some*
simplicity to get a performance boost in return, though not as fast as
going fully native.

Structurally, bytecode resembles machine code. It’s a dense, linear
sequence of binary instructions. That keeps overhead low and plays nice
with the cache. However, it’s a much simpler, higher-level instruction
set than any real chip out there. (In many bytecode formats, each
instruction is only a single byte long, hence “bytecode”.)

Imagine you’re writing a native compiler from some source language and
you’re given carte blanche to define the easiest possible architecture
to target. Bytecode is kind of like that. It’s an idealized fantasy
instruction set that makes your life as the compiler writer easier.

The problem with a fantasy architecture, of course, is that it doesn’t
exist. We solve that by writing an *emulator*<span class="em">—</span>a
simulated chip written in software that interprets the bytecode one
instruction at a time. A *virtual machine (VM)*, if you will.

That emulation layer adds <span id="p-code">overhead</span>, which is a
key reason bytecode is slower than native code. But in return, it gives
us portability. Write our VM in a language like C that is already
supported on all the machines we care about, and we can run our emulator
on top of any hardware we like.

One of the first bytecode formats was
[p-code](https://en.wikipedia.org/wiki/P-code_machine), developed for
Niklaus Wirth’s Pascal language. You might think a PDP-11 running at
15MHz couldn’t afford the overhead of emulating a virtual machine. But
back then, computers were in their Cambrian explosion and new
architectures appeared every day. Keeping up with the latest chips was
worth more than squeezing the maximum performance from each one. That’s
why the “p” in p-code doesn’t stand for “Pascal”, but “portable”.

This is the path we’ll take with our new interpreter, clox. We’ll follow
in the footsteps of the main implementations of Python, Ruby, Lua,
OCaml, Erlang, and others. In many ways, our VM’s design will parallel
the structure of our previous interpreter:

![Phases of the two implementations. jlox is Parser to Syntax Trees to
Interpreter. clox is Compiler to Bytecode to Virtual
Machine.](image/chunks-of-bytecode/phases.png)

Of course, we won’t implement the phases strictly in order. Like our
previous interpreter, we’ll bounce around, building up the
implementation one language feature at a time. In this chapter, we’ll
get the skeleton of the application in place and create the data
structures needed to store and represent a chunk of bytecode.

## <a href="#getting-started" id="getting-started"><span
class="small">14 . 2</span>Getting Started</a>

Where else to begin, but at `main()`? <span id="ready">Fire</span> up
your trusty text editor and start typing.

Now is a good time to stretch, maybe crack your knuckles. A little
montage music wouldn’t hurt either.

<div class="codehilite">

<div class="source-file">

*main.c*  
create new file

</div>

    #include "common.h"

    int main(int argc, const char* argv[]) {
      return 0;
    }

</div>

<div class="source-file-narrow">

*main.c*, create new file

</div>

From this tiny seed, we will grow our entire VM. Since C provides us
with so little, we first need to spend some time amending the soil. Some
of that goes into this header:

<div class="codehilite">

<div class="source-file">

*common.h*  
create new file

</div>

    #ifndef clox_common_h
    #define clox_common_h

    #include <stdbool.h>
    #include <stddef.h>
    #include <stdint.h>

    #endif

</div>

<div class="source-file-narrow">

*common.h*, create new file

</div>

There are a handful of types and constants we’ll use throughout the
interpreter, and this is a convenient place to put them. For now, it’s
the venerable `NULL`, `size_t`, the nice C99 Boolean `bool`, and
explicit-sized integer types<span class="em">—</span>`uint8_t` and
friends.

## <a href="#chunks-of-instructions" id="chunks-of-instructions"><span
class="small">14 . 3</span>Chunks of Instructions</a>

Next, we need a module to define our code representation. I’ve been
using “chunk” to refer to sequences of bytecode, so let’s make that the
official name for that module.

<div class="codehilite">

<div class="source-file">

*chunk.h*  
create new file

</div>

    #ifndef clox_chunk_h
    #define clox_chunk_h

    #include "common.h"

    #endif

</div>

<div class="source-file-narrow">

*chunk.h*, create new file

</div>

In our bytecode format, each instruction has a one-byte **operation
code** (universally shortened to **opcode**). That number controls what
kind of instruction we’re dealing with<span class="em">—</span>add,
subtract, look up variable, etc. We define those here:

<div class="codehilite">

``` insert-before
#include "common.h"
```

<div class="source-file">

*chunk.h*

</div>

``` insert

typedef enum {
  OP_RETURN,
} OpCode;
```

``` insert-after

#endif
```

</div>

<div class="source-file-narrow">

*chunk.h*

</div>

For now, we start with a single instruction, `OP_RETURN`. When we have a
full-featured VM, this instruction will mean “return from the current
function”. I admit this isn’t exactly useful yet, but we have to start
somewhere, and this is a particularly simple instruction, for reasons
we’ll get to later.

### <a href="#a-dynamic-array-of-instructions"
id="a-dynamic-array-of-instructions"><span
class="small">14 . 3 . 1</span>A dynamic array of instructions</a>

Bytecode is a series of instructions. Eventually, we’ll store some other
data along with the instructions, so let’s go ahead and create a struct
to hold it all.

<div class="codehilite">

``` insert-before
} OpCode;
```

<div class="source-file">

*chunk.h*  
add after enum *OpCode*

</div>

``` insert

typedef struct {
  uint8_t* code;
} Chunk;
```

``` insert-after

#endif
```

</div>

<div class="source-file-narrow">

*chunk.h*, add after enum *OpCode*

</div>

At the moment, this is simply a wrapper around an array of bytes. Since
we don’t know how big the array needs to be before we start compiling a
chunk, it must be dynamic. Dynamic arrays are one of my favorite data
structures. That sounds like claiming vanilla is my favorite ice cream
<span id="flavor">flavor</span>, but hear me out. Dynamic arrays
provide:

Butter pecan is actually my favorite.

- Cache-friendly, dense storage

- Constant-time indexed element lookup

- Constant-time appending to the end of the array

Those features are exactly why we used dynamic arrays all the time in
jlox under the guise of Java’s ArrayList class. Now that we’re in C, we
get to roll our own. If you’re rusty on dynamic arrays, the idea is
pretty simple. In addition to the array itself, we keep two numbers: the
number of elements in the array we have allocated (“capacity”) and how
many of those allocated entries are actually in use (“count”).

<div class="codehilite">

``` insert-before
typedef struct {
```

<div class="source-file">

*chunk.h*  
in struct *Chunk*

</div>

``` insert
  int count;
  int capacity;
```

``` insert-after
  uint8_t* code;
} Chunk;
```

</div>

<div class="source-file-narrow">

*chunk.h*, in struct *Chunk*

</div>

When we add an element, if the count is less than the capacity, then
there is already available space in the array. We store the new element
right in there and bump the count.

![Storing an element in an array that has enough
capacity.](image/chunks-of-bytecode/insert.png)

If we have no spare capacity, then the process is a little more
involved.

<img src="image/chunks-of-bytecode/grow.png" class="wide"
alt="Growing the dynamic array before storing an element." />

1.  <span id="amortized">Allocate</span> a new array with more capacity.
2.  Copy the existing elements from the old array to the new one.
3.  Store the new `capacity`.
4.  Delete the old array.
5.  Update `code` to point to the new array.
6.  Store the element in the new array now that there is room.
7.  Update the `count`.

Copying the existing elements when you grow the array makes it seem like
appending an element is *O(n)*, not *O(1)* like I said above. However,
you need to do this copy step only on *some* of the appends. Most of the
time, there is already extra capacity, so you don’t need to copy.

To understand how this works, we need [**amortized
analysis**](https://en.wikipedia.org/wiki/Amortized_analysis). That
shows us that as long as we grow the array by a multiple of its current
size, when we average out the cost of a *sequence* of appends, each
append is *O(1)*.

We have our struct ready, so let’s implement the functions to work with
it. C doesn’t have constructors, so we declare a function to initialize
a new chunk.

<div class="codehilite">

``` insert-before
} Chunk;
```

<div class="source-file">

*chunk.h*  
add after struct *Chunk*

</div>

``` insert

void initChunk(Chunk* chunk);
```

``` insert-after

#endif
```

</div>

<div class="source-file-narrow">

*chunk.h*, add after struct *Chunk*

</div>

And implement it thusly:

<div class="codehilite">

<div class="source-file">

*chunk.c*  
create new file

</div>

    #include <stdlib.h>

    #include "chunk.h"

    void initChunk(Chunk* chunk) {
      chunk->count = 0;
      chunk->capacity = 0;
      chunk->code = NULL;
    }

</div>

<div class="source-file-narrow">

*chunk.c*, create new file

</div>

The dynamic array starts off completely empty. We don’t even allocate a
raw array yet. To append a byte to the end of the chunk, we use a new
function.

<div class="codehilite">

``` insert-before
void initChunk(Chunk* chunk);
```

<div class="source-file">

*chunk.h*  
add after *initChunk*()

</div>

``` insert
void writeChunk(Chunk* chunk, uint8_t byte);
```

``` insert-after

#endif
```

</div>

<div class="source-file-narrow">

*chunk.h*, add after *initChunk*()

</div>

This is where the interesting work happens.

<div class="codehilite">

<div class="source-file">

*chunk.c*  
add after *initChunk*()

</div>

    void writeChunk(Chunk* chunk, uint8_t byte) {
      if (chunk->capacity < chunk->count + 1) {
        int oldCapacity = chunk->capacity;
        chunk->capacity = GROW_CAPACITY(oldCapacity);
        chunk->code = GROW_ARRAY(uint8_t, chunk->code,
            oldCapacity, chunk->capacity);
      }

      chunk->code[chunk->count] = byte;
      chunk->count++;
    }

</div>

<div class="source-file-narrow">

*chunk.c*, add after *initChunk*()

</div>

The first thing we need to do is see if the current array already has
capacity for the new byte. If it doesn’t, then we first need to grow the
array to make room. (We also hit this case on the very first write when
the array is `NULL` and `capacity` is 0.)

To grow the array, first we figure out the new capacity and grow the
array to that size. Both of those lower-level memory operations are
defined in a new module.

<div class="codehilite">

``` insert-before
#include "chunk.h"
```

<div class="source-file">

*chunk.c*

</div>

``` insert
#include "memory.h"
```

``` insert-after

void initChunk(Chunk* chunk) {
```

</div>

<div class="source-file-narrow">

*chunk.c*

</div>

This is enough to get us started.

<div class="codehilite">

<div class="source-file">

*memory.h*  
create new file

</div>

    #ifndef clox_memory_h
    #define clox_memory_h

    #include "common.h"

    #define GROW_CAPACITY(capacity) \
        ((capacity) < 8 ? 8 : (capacity) * 2)

    #endif

</div>

<div class="source-file-narrow">

*memory.h*, create new file

</div>

This macro calculates a new capacity based on a given current capacity.
In order to get the performance we want, the important part is that it
*scales* based on the old size. We grow by a factor of two, which is
pretty typical. 1.5× is another common choice.

We also handle when the current capacity is zero. In that case, we jump
straight to eight elements instead of starting at one. That
<span id="profile">avoids</span> a little extra memory churn when the
array is very small, at the expense of wasting a few bytes on very small
chunks.

I picked the number eight somewhat arbitrarily for the book. Most
dynamic array implementations have a minimum threshold like this. The
right way to pick a value for this is to profile against real-world
usage and see which constant makes the best performance trade-off
between extra grows versus wasted space.

Once we know the desired capacity, we create or grow the array to that
size using `GROW_ARRAY()`.

<div class="codehilite">

``` insert-before
#define GROW_CAPACITY(capacity) \
    ((capacity) < 8 ? 8 : (capacity) * 2)
```

<div class="source-file">

*memory.h*

</div>

``` insert

#define GROW_ARRAY(type, pointer, oldCount, newCount) \
    (type*)reallocate(pointer, sizeof(type) * (oldCount), \
        sizeof(type) * (newCount))

void* reallocate(void* pointer, size_t oldSize, size_t newSize);
```

``` insert-after

#endif
```

</div>

<div class="source-file-narrow">

*memory.h*

</div>

This macro pretties up a function call to `reallocate()` where the real
work happens. The macro itself takes care of getting the size of the
array’s element type and casting the resulting `void*` back to a pointer
of the right type.

This `reallocate()` function is the single function we’ll use for all
dynamic memory management in clox<span class="em">—</span>allocating
memory, freeing it, and changing the size of an existing allocation.
Routing all of those operations through a single function will be
important later when we add a garbage collector that needs to keep track
of how much memory is in use.

The two size arguments passed to `reallocate()` control which operation
to perform:

| oldSize  | newSize                | Operation                   |
|----------|------------------------|-----------------------------|
| 0        | Non‑zero               | Allocate new block.         |
| Non‑zero | 0                      | Free allocation.            |
| Non‑zero | Smaller than `oldSize` | Shrink existing allocation. |
| Non‑zero | Larger than `oldSize`  | Grow existing allocation.   |

That sounds like a lot of cases to handle, but here’s the
implementation:

<div class="codehilite">

<div class="source-file">

*memory.c*  
create new file

</div>

    #include <stdlib.h>

    #include "memory.h"

    void* reallocate(void* pointer, size_t oldSize, size_t newSize) {
      if (newSize == 0) {
        free(pointer);
        return NULL;
      }

      void* result = realloc(pointer, newSize);
      return result;
    }

</div>

<div class="source-file-narrow">

*memory.c*, create new file

</div>

When `newSize` is zero, we handle the deallocation case ourselves by
calling `free()`. Otherwise, we rely on the C standard library’s
`realloc()` function. That function conveniently supports the other
three aspects of our policy. When `oldSize` is zero, `realloc()` is
equivalent to calling `malloc()`.

The interesting cases are when both `oldSize` and `newSize` are not
zero. Those tell `realloc()` to resize the previously allocated block.
If the new size is smaller than the existing block of memory, it simply
<span id="shrink">updates</span> the size of the block and returns the
same pointer you gave it. If the new size is larger, it attempts to grow
the existing block of memory.

It can do that only if the memory after that block isn’t already in use.
If there isn’t room to grow the block, `realloc()` instead allocates a
*new* block of memory of the desired size, copies over the old bytes,
frees the old block, and then returns a pointer to the new block.
Remember, that’s exactly the behavior we want for our dynamic array.

Because computers are finite lumps of matter and not the perfect
mathematical abstractions computer science theory would have us believe,
allocation can fail if there isn’t enough memory and `realloc()` will
return `NULL`. We should handle that.

<div class="codehilite">

``` insert-before
  void* result = realloc(pointer, newSize);
```

<div class="source-file">

*memory.c*  
in *reallocate*()

</div>

``` insert
  if (result == NULL) exit(1);
```

``` insert-after
  return result;
```

</div>

<div class="source-file-narrow">

*memory.c*, in *reallocate*()

</div>

There’s not really anything *useful* that our VM can do if it can’t get
the memory it needs, but we at least detect that and abort the process
immediately instead of returning a `NULL` pointer and letting it go off
the rails later.

Since all we passed in was a bare pointer to the first byte of memory,
what does it mean to “update” the block’s size? Under the hood, the
memory allocator maintains additional bookkeeping information for each
block of heap-allocated memory, including its size.

Given a pointer to some previously allocated memory, it can find this
bookkeeping information, which is necessary to be able to cleanly free
it. It’s this size metadata that `realloc()` updates.

Many implementations of `malloc()` store the allocated size in memory
right *before* the returned address.

OK, we can create new chunks and write instructions to them. Are we
done? Nope! We’re in C now, remember, we have to manage memory
ourselves, like in Ye Olden Times, and that means *freeing* it too.

<div class="codehilite">

``` insert-before
void initChunk(Chunk* chunk);
```

<div class="source-file">

*chunk.h*  
add after *initChunk*()

</div>

``` insert
void freeChunk(Chunk* chunk);
```

``` insert-after
void writeChunk(Chunk* chunk, uint8_t byte);
```

</div>

<div class="source-file-narrow">

*chunk.h*, add after *initChunk*()

</div>

The implementation is:

<div class="codehilite">

<div class="source-file">

*chunk.c*  
add after *initChunk*()

</div>

    void freeChunk(Chunk* chunk) {
      FREE_ARRAY(uint8_t, chunk->code, chunk->capacity);
      initChunk(chunk);
    }

</div>

<div class="source-file-narrow">

*chunk.c*, add after *initChunk*()

</div>

We deallocate all of the memory and then call `initChunk()` to zero out
the fields leaving the chunk in a well-defined empty state. To free the
memory, we add one more macro.

<div class="codehilite">

``` insert-before
#define GROW_ARRAY(type, pointer, oldCount, newCount) \
    (type*)reallocate(pointer, sizeof(type) * (oldCount), \
        sizeof(type) * (newCount))
```

<div class="source-file">

*memory.h*

</div>

``` insert

#define FREE_ARRAY(type, pointer, oldCount) \
    reallocate(pointer, sizeof(type) * (oldCount), 0)
```

``` insert-after

void* reallocate(void* pointer, size_t oldSize, size_t newSize);
```

</div>

<div class="source-file-narrow">

*memory.h*

</div>

Like `GROW_ARRAY()`, this is a wrapper around a call to `reallocate()`.
This one frees the memory by passing in zero for the new size. I know,
this is a lot of boring low-level stuff. Don’t worry, we’ll get a lot of
use out of these in later chapters and will get to program at a higher
level. Before we can do that, though, we gotta lay our own foundation.

## <a href="#disassembling-chunks" id="disassembling-chunks"><span
class="small">14 . 4</span>Disassembling Chunks</a>

Now we have a little module for creating chunks of bytecode. Let’s try
it out by hand-building a sample chunk.

<div class="codehilite">

``` insert-before
int main(int argc, const char* argv[]) {
```

<div class="source-file">

*main.c*  
in *main*()

</div>

``` insert
  Chunk chunk;
  initChunk(&chunk);
  writeChunk(&chunk, OP_RETURN);
  freeChunk(&chunk);
```

``` insert-after
  return 0;
```

</div>

<div class="source-file-narrow">

*main.c*, in *main*()

</div>

Don’t forget the include.

<div class="codehilite">

``` insert-before
#include "common.h"
```

<div class="source-file">

*main.c*

</div>

``` insert
#include "chunk.h"
```

``` insert-after

int main(int argc, const char* argv[]) {
```

</div>

<div class="source-file-narrow">

*main.c*

</div>

Run that and give it a try. Did it work?
Uh<span class="ellipse"> . . . </span>who knows? All we’ve done is push
some bytes around in memory. We have no human-friendly way to see what’s
actually inside that chunk we made.

To fix this, we’re going to create a **disassembler**. An **assembler**
is an old-school program that takes a file containing human-readable
mnemonic names for CPU instructions like “ADD” and “MULT” and translates
them to their binary machine code equivalent. A *dis*assembler goes in
the other direction<span class="em">—</span>given a blob of machine
code, it spits out a textual listing of the instructions.

We’ll implement something <span id="printer">similar</span>. Given a
chunk, it will print out all of the instructions in it. A Lox *user*
won’t use this, but we Lox *maintainers* will certainly benefit since it
gives us a window into the interpreter’s internal representation of
code.

In jlox, our analogous tool was the [AstPrinter
class](representing-code.html#a-not-very-pretty-printer).

In `main()`, after we create the chunk, we pass it to the disassembler.

<div class="codehilite">

``` insert-before
  initChunk(&chunk);
  writeChunk(&chunk, OP_RETURN);
```

<div class="source-file">

*main.c*  
in *main*()

</div>

``` insert

  disassembleChunk(&chunk, "test chunk");
```

``` insert-after
  freeChunk(&chunk);
```

</div>

<div class="source-file-narrow">

*main.c*, in *main*()

</div>

Again, we whip up <span id="module">yet another</span> module.

I promise you we won’t be creating this many new files in later
chapters.

<div class="codehilite">

``` insert-before
#include "chunk.h"
```

<div class="source-file">

*main.c*

</div>

``` insert
#include "debug.h"
```

``` insert-after

int main(int argc, const char* argv[]) {
```

</div>

<div class="source-file-narrow">

*main.c*

</div>

Here’s that header:

<div class="codehilite">

<div class="source-file">

*debug.h*  
create new file

</div>

    #ifndef clox_debug_h
    #define clox_debug_h

    #include "chunk.h"

    void disassembleChunk(Chunk* chunk, const char* name);
    int disassembleInstruction(Chunk* chunk, int offset);

    #endif

</div>

<div class="source-file-narrow">

*debug.h*, create new file

</div>

In `main()`, we call `disassembleChunk()` to disassemble all of the
instructions in the entire chunk. That’s implemented in terms of the
other function, which just disassembles a single instruction. It shows
up here in the header because we’ll call it from the VM in later
chapters.

Here’s a start at the implementation file:

<div class="codehilite">

<div class="source-file">

*debug.c*  
create new file

</div>

    #include <stdio.h>

    #include "debug.h"

    void disassembleChunk(Chunk* chunk, const char* name) {
      printf("== %s ==\n", name);

      for (int offset = 0; offset < chunk->count;) {
        offset = disassembleInstruction(chunk, offset);
      }
    }

</div>

<div class="source-file-narrow">

*debug.c*, create new file

</div>

To disassemble a chunk, we print a little header (so we can tell *which*
chunk we’re looking at) and then crank through the bytecode,
disassembling each instruction. The way we iterate through the code is a
little odd. Instead of incrementing `offset` in the loop, we let
`disassembleInstruction()` do it for us. When we call that function,
after disassembling the instruction at the given offset, it returns the
offset of the *next* instruction. This is because, as we’ll see later,
instructions can have different sizes.

The core of the “debug” module is this function:

<div class="codehilite">

<div class="source-file">

*debug.c*  
add after *disassembleChunk*()

</div>

    int disassembleInstruction(Chunk* chunk, int offset) {
      printf("%04d ", offset);

      uint8_t instruction = chunk->code[offset];
      switch (instruction) {
        case OP_RETURN:
          return simpleInstruction("OP_RETURN", offset);
        default:
          printf("Unknown opcode %d\n", instruction);
          return offset + 1;
      }
    }

</div>

<div class="source-file-narrow">

*debug.c*, add after *disassembleChunk*()

</div>

First, it prints the byte offset of the given
instruction<span class="em">—</span>that tells us where in the chunk
this instruction is. This will be a helpful signpost when we start doing
control flow and jumping around in the bytecode.

Next, it reads a single byte from the bytecode at the given offset.
That’s our opcode. We <span id="switch">switch</span> on that. For each
kind of instruction, we dispatch to a little utility function for
displaying it. On the off chance that the given byte doesn’t look like
an instruction at all<span class="em">—</span>a bug in our
compiler<span class="em">—</span>we print that too. For the one
instruction we do have, `OP_RETURN`, the display function is:

We have only one instruction right now, but this switch will grow
throughout the rest of the book.

<div class="codehilite">

<div class="source-file">

*debug.c*  
add after *disassembleChunk*()

</div>

    static int simpleInstruction(const char* name, int offset) {
      printf("%s\n", name);
      return offset + 1;
    }

</div>

<div class="source-file-narrow">

*debug.c*, add after *disassembleChunk*()

</div>

There isn’t much to a return instruction, so all it does is print the
name of the opcode, then return the next byte offset past this
instruction. Other instructions will have more going on.

If we run our nascent interpreter now, it actually prints something:

<div class="codehilite">

    == test chunk ==
    0000 OP_RETURN

</div>

It worked! This is sort of the “Hello, world!” of our code
representation. We can create a chunk, write an instruction to it, and
then extract that instruction back out. Our encoding and decoding of the
binary bytecode is working.

## <a href="#constants" id="constants"><span
class="small">14 . 5</span>Constants</a>

Now that we have a rudimentary chunk structure working, let’s start
making it more useful. We can store *code* in chunks, but what about
*data*? Many values the interpreter works with are created at runtime as
the result of operations.

<div class="codehilite">

    1 + 2;

</div>

The value 3 appears nowhere in the code here. However, the literals `1`
and `2` do. To compile that statement to bytecode, we need some sort of
instruction that means “produce a constant” and those literal values
need to get stored in the chunk somewhere. In jlox, the Expr.Literal AST
node held the value. We need a different solution now that we don’t have
a syntax tree.

### <a href="#representing-values" id="representing-values"><span
class="small">14 . 5 . 1</span>Representing values</a>

We won’t be *running* any code in this chapter, but since constants have
a foot in both the static and dynamic worlds of our interpreter, they
force us to start thinking at least a little bit about how our VM should
represent values.

For now, we’re going to start as simple as
possible<span class="em">—</span>we’ll support only double-precision,
floating-point numbers. This will obviously expand over time, so we’ll
set up a new module to give ourselves room to grow.

<div class="codehilite">

<div class="source-file">

*value.h*  
create new file

</div>

    #ifndef clox_value_h
    #define clox_value_h

    #include "common.h"

    typedef double Value;

    #endif

</div>

<div class="source-file-narrow">

*value.h*, create new file

</div>

This typedef abstracts how Lox values are concretely represented in C.
That way, we can change that representation without needing to go back
and fix existing code that passes around values.

Back to the question of where to store constants in a chunk. For small
fixed-size values like integers, many instruction sets store the value
directly in the code stream right after the opcode. These are called
**immediate instructions** because the bits for the value are
immediately after the opcode.

That doesn’t work well for large or variable-sized constants like
strings. In a native compiler to machine code, those bigger constants
get stored in a separate “constant data” region in the binary
executable. Then, the instruction to load a constant has an address or
offset pointing to where the value is stored in that section.

Most virtual machines do something similar. For example, the Java
Virtual Machine [associates a **constant
pool**](https://docs.oracle.com/javase/specs/jvms/se7/html/jvms-4.html#jvms-4.4)
with each compiled class. That sounds good enough for clox to me. Each
chunk will carry with it a list of the values that appear as literals in
the program. To keep things <span id="immediate">simpler</span>, we’ll
put *all* constants in there, even simple integers.

In addition to needing two kinds of constant
instructions<span class="em">—</span>one for immediate values and one
for constants in the constant table<span class="em">—</span>immediates
also force us to worry about alignment, padding, and endianness. Some
architectures aren’t happy if you try to say, stuff a 4-byte integer at
an odd address.

### <a href="#value-arrays" id="value-arrays"><span
class="small">14 . 5 . 2</span>Value arrays</a>

The constant pool is an array of values. The instruction to load a
constant looks up the value by index in that array. As with our
<span id="generic">bytecode</span> array, the compiler doesn’t know how
big the array needs to be ahead of time. So, again, we need a dynamic
one. Since C doesn’t have generic data structures, we’ll write another
dynamic array data structure, this time for Value.

Defining a new struct and manipulation functions each time we need a
dynamic array of a different type is a chore. We could cobble together
some preprocessor macros to fake generics, but that’s overkill for clox.
We won’t need many more of these.

<div class="codehilite">

``` insert-before
typedef double Value;
```

<div class="source-file">

*value.h*

</div>

``` insert

typedef struct {
  int capacity;
  int count;
  Value* values;
} ValueArray;
```

``` insert-after

#endif
```

</div>

<div class="source-file-narrow">

*value.h*

</div>

As with the bytecode array in Chunk, this struct wraps a pointer to an
array along with its allocated capacity and the number of elements in
use. We also need the same three functions to work with value arrays.

<div class="codehilite">

``` insert-before
} ValueArray;
```

<div class="source-file">

*value.h*  
add after struct *ValueArray*

</div>

``` insert

void initValueArray(ValueArray* array);
void writeValueArray(ValueArray* array, Value value);
void freeValueArray(ValueArray* array);
```

``` insert-after

#endif
```

</div>

<div class="source-file-narrow">

*value.h*, add after struct *ValueArray*

</div>

The implementations will probably give you déjà vu. First, to create a
new one:

<div class="codehilite">

<div class="source-file">

*value.c*  
create new file

</div>

    #include <stdio.h>

    #include "memory.h"
    #include "value.h"

    void initValueArray(ValueArray* array) {
      array->values = NULL;
      array->capacity = 0;
      array->count = 0;
    }

</div>

<div class="source-file-narrow">

*value.c*, create new file

</div>

Once we have an initialized array, we can start
<span id="add">adding</span> values to it.

Fortunately, we don’t need other operations like insertion and removal.

<div class="codehilite">

<div class="source-file">

*value.c*  
add after *initValueArray*()

</div>

    void writeValueArray(ValueArray* array, Value value) {
      if (array->capacity < array->count + 1) {
        int oldCapacity = array->capacity;
        array->capacity = GROW_CAPACITY(oldCapacity);
        array->values = GROW_ARRAY(Value, array->values,
                                   oldCapacity, array->capacity);
      }

      array->values[array->count] = value;
      array->count++;
    }

</div>

<div class="source-file-narrow">

*value.c*, add after *initValueArray*()

</div>

The memory-management macros we wrote earlier do let us reuse some of
the logic from the code array, so this isn’t too bad. Finally, to
release all memory used by the array:

<div class="codehilite">

<div class="source-file">

*value.c*  
add after *writeValueArray*()

</div>

    void freeValueArray(ValueArray* array) {
      FREE_ARRAY(Value, array->values, array->capacity);
      initValueArray(array);
    }

</div>

<div class="source-file-narrow">

*value.c*, add after *writeValueArray*()

</div>

Now that we have growable arrays of values, we can add one to Chunk to
store the chunk’s constants.

<div class="codehilite">

``` insert-before
  uint8_t* code;
```

<div class="source-file">

*chunk.h*  
in struct *Chunk*

</div>

``` insert
  ValueArray constants;
```

``` insert-after
} Chunk;
```

</div>

<div class="source-file-narrow">

*chunk.h*, in struct *Chunk*

</div>

Don’t forget the include.

<div class="codehilite">

``` insert-before
#include "common.h"
```

<div class="source-file">

*chunk.h*

</div>

``` insert
#include "value.h"
```

``` insert-after

typedef enum {
```

</div>

<div class="source-file-narrow">

*chunk.h*

</div>

Ah, C, and its Stone Age modularity story. Where were we? Right. When we
initialize a new chunk, we initialize its constant list too.

<div class="codehilite">

``` insert-before
  chunk->code = NULL;
```

<div class="source-file">

*chunk.c*  
in *initChunk*()

</div>

``` insert
  initValueArray(&chunk->constants);
```

``` insert-after
}
```

</div>

<div class="source-file-narrow">

*chunk.c*, in *initChunk*()

</div>

Likewise, we free the constants when we free the chunk.

<div class="codehilite">

``` insert-before
  FREE_ARRAY(uint8_t, chunk->code, chunk->capacity);
```

<div class="source-file">

*chunk.c*  
in *freeChunk*()

</div>

``` insert
  freeValueArray(&chunk->constants);
```

``` insert-after
  initChunk(chunk);
```

</div>

<div class="source-file-narrow">

*chunk.c*, in *freeChunk*()

</div>

Next, we define a convenience method to add a new constant to the chunk.
Our yet-to-be-written compiler could write to the constant array inside
Chunk directly<span class="em">—</span>it’s not like C has private
fields or anything<span class="em">—</span>but it’s a little nicer to
add an explicit function.

<div class="codehilite">

``` insert-before
void writeChunk(Chunk* chunk, uint8_t byte);
```

<div class="source-file">

*chunk.h*  
add after *writeChunk*()

</div>

``` insert
int addConstant(Chunk* chunk, Value value);
```

``` insert-after

#endif
```

</div>

<div class="source-file-narrow">

*chunk.h*, add after *writeChunk*()

</div>

Then we implement it.

<div class="codehilite">

<div class="source-file">

*chunk.c*  
add after *writeChunk*()

</div>

    int addConstant(Chunk* chunk, Value value) {
      writeValueArray(&chunk->constants, value);
      return chunk->constants.count - 1;
    }

</div>

<div class="source-file-narrow">

*chunk.c*, add after *writeChunk*()

</div>

After we add the constant, we return the index where the constant was
appended so that we can locate that same constant later.

### <a href="#constant-instructions" id="constant-instructions"><span
class="small">14 . 5 . 3</span>Constant instructions</a>

We can *store* constants in chunks, but we also need to *execute* them.
In a piece of code like:

<div class="codehilite">

    print 1;
    print 2;

</div>

The compiled chunk needs to not only contain the values 1 and 2, but
know *when* to produce them so that they are printed in the right order.
Thus, we need an instruction that produces a particular constant.

<div class="codehilite">

``` insert-before
typedef enum {
```

<div class="source-file">

*chunk.h*  
in enum *OpCode*

</div>

``` insert
  OP_CONSTANT,
```

``` insert-after
  OP_RETURN,
```

</div>

<div class="source-file-narrow">

*chunk.h*, in enum *OpCode*

</div>

When the VM executes a constant instruction, it
<span id="load">“loads”</span> the constant for use. This new
instruction is a little more complex than `OP_RETURN`. In the above
example, we load two different constants. A single bare opcode isn’t
enough to know *which* constant to load.

I’m being vague about what it means to “load” or “produce” a constant
because we haven’t learned how the virtual machine actually executes
code at runtime yet. For that, you’ll have to wait until you get to (or
skip ahead to, I suppose) the [next chapter](a-virtual-machine.html).

To handle cases like this, our bytecode<span class="em">—</span>like
most others<span class="em">—</span>allows instructions to have
<span id="operand">**operands**</span>. These are stored as binary data
immediately after the opcode in the instruction stream and let us
parameterize what the instruction does.

![OP_CONSTANT is a byte for the opcode followed by a byte for the
constant index.](image/chunks-of-bytecode/format.png)

Each opcode determines how many operand bytes it has and what they mean.
For example, a simple operation like “return” may have no operands,
where an instruction for “load local variable” needs an operand to
identify which variable to load. Each time we add a new opcode to clox,
we specify what its operands look like<span class="em">—</span>its
**instruction format**.

Bytecode instruction operands are *not* the same as the operands passed
to an arithmetic operator. You’ll see when we get to expressions that
arithmetic operand values are tracked separately. Instruction operands
are a lower-level notion that modify how the bytecode instruction itself
behaves.

In this case, `OP_CONSTANT` takes a single byte operand that specifies
which constant to load from the chunk’s constant array. Since we don’t
have a compiler yet, we “hand-compile” an instruction in our test chunk.

<div class="codehilite">

``` insert-before
  initChunk(&chunk);
```

<div class="source-file">

*main.c*  
in *main*()

</div>

``` insert

  int constant = addConstant(&chunk, 1.2);
  writeChunk(&chunk, OP_CONSTANT);
  writeChunk(&chunk, constant);
```

``` insert-after
  writeChunk(&chunk, OP_RETURN);
```

</div>

<div class="source-file-narrow">

*main.c*, in *main*()

</div>

We add the constant value itself to the chunk’s constant pool. That
returns the index of the constant in the array. Then we write the
constant instruction, starting with its opcode. After that, we write the
one-byte constant index operand. Note that `writeChunk()` can write
opcodes or operands. It’s all raw bytes as far as that function is
concerned.

If we try to run this now, the disassembler is going to yell at us
because it doesn’t know how to decode the new instruction. Let’s fix
that.

<div class="codehilite">

``` insert-before
  switch (instruction) {
```

<div class="source-file">

*debug.c*  
in *disassembleInstruction*()

</div>

``` insert
    case OP_CONSTANT:
      return constantInstruction("OP_CONSTANT", chunk, offset);
```

``` insert-after
    case OP_RETURN:
```

</div>

<div class="source-file-narrow">

*debug.c*, in *disassembleInstruction*()

</div>

This instruction has a different instruction format, so we write a new
helper function to disassemble it.

<div class="codehilite">

<div class="source-file">

*debug.c*  
add after *disassembleChunk*()

</div>

    static int constantInstruction(const char* name, Chunk* chunk,
                                   int offset) {
      uint8_t constant = chunk->code[offset + 1];
      printf("%-16s %4d '", name, constant);
      printValue(chunk->constants.values[constant]);
      printf("'\n");
    }

</div>

<div class="source-file-narrow">

*debug.c*, add after *disassembleChunk*()

</div>

There’s more going on here. As with `OP_RETURN`, we print out the name
of the opcode. Then we pull out the constant index from the subsequent
byte in the chunk. We print that index, but that isn’t super useful to
us human readers. So we also look up the actual constant
value<span class="em">—</span>since constants *are* known at compile
time after all<span class="em">—</span>and display the value itself too.

This requires some way to print a clox Value. That function will live in
the “value” module, so we include that.

<div class="codehilite">

``` insert-before
#include "debug.h"
```

<div class="source-file">

*debug.c*

</div>

``` insert
#include "value.h"
```

``` insert-after

void disassembleChunk(Chunk* chunk, const char* name) {
```

</div>

<div class="source-file-narrow">

*debug.c*

</div>

Over in that header, we declare:

<div class="codehilite">

``` insert-before
void freeValueArray(ValueArray* array);
```

<div class="source-file">

*value.h*  
add after *freeValueArray*()

</div>

``` insert
void printValue(Value value);
```

``` insert-after

#endif
```

</div>

<div class="source-file-narrow">

*value.h*, add after *freeValueArray*()

</div>

And here’s an implementation:

<div class="codehilite">

<div class="source-file">

*value.c*  
add after *freeValueArray*()

</div>

    void printValue(Value value) {
      printf("%g", value);
    }

</div>

<div class="source-file-narrow">

*value.c*, add after *freeValueArray*()

</div>

Magnificent, right? As you can imagine, this is going to get more
complex once we add dynamic typing to Lox and have values of different
types.

Back in `constantInstruction()`, the only remaining piece is the return
value.

<div class="codehilite">

``` insert-before
  printf("'\n");
```

<div class="source-file">

*debug.c*  
in *constantInstruction*()

</div>

``` insert
  return offset + 2;
```

``` insert-after
}
```

</div>

<div class="source-file-narrow">

*debug.c*, in *constantInstruction*()

</div>

Remember that `disassembleInstruction()` also returns a number to tell
the caller the offset of the beginning of the *next* instruction. Where
`OP_RETURN` was only a single byte, `OP_CONSTANT` is
two<span class="em">—</span>one for the opcode and one for the operand.

## <a href="#line-information" id="line-information"><span
class="small">14 . 6</span>Line Information</a>

Chunks contain almost all of the information that the runtime needs from
the user’s source code. It’s kind of crazy to think that we can reduce
all of the different AST classes that we created in jlox down to an
array of bytes and an array of constants. There’s only one piece of data
we’re missing. We need it, even though the user hopes to never see it.

When a runtime error occurs, we show the user the line number of the
offending source code. In jlox, those numbers live in tokens, which we
in turn store in the AST nodes. We need a different solution for clox
now that we’ve ditched syntax trees in favor of bytecode. Given any
bytecode instruction, we need to be able to determine the line of the
user’s source program that it was compiled from.

There are a lot of clever ways we could encode this. I took the absolute
<span id="side">simplest</span> approach I could come up with, even
though it’s embarrassingly inefficient with memory. In the chunk, we
store a separate array of integers that parallels the bytecode. Each
number in the array is the line number for the corresponding byte in the
bytecode. When a runtime error occurs, we look up the line number at the
same index as the current instruction’s offset in the code array.

This braindead encoding does do one thing right: it keeps the line
information in a *separate* array instead of interleaving it in the
bytecode itself. Since line information is only used when a runtime
error occurs, we don’t want it between the instructions, taking up
precious space in the CPU cache and causing more cache misses as the
interpreter skips past it to get to the opcodes and operands it cares
about.

To implement this, we add another array to Chunk.

<div class="codehilite">

``` insert-before
  uint8_t* code;
```

<div class="source-file">

*chunk.h*  
in struct *Chunk*

</div>

``` insert
  int* lines;
```

``` insert-after
  ValueArray constants;
```

</div>

<div class="source-file-narrow">

*chunk.h*, in struct *Chunk*

</div>

Since it exactly parallels the bytecode array, we don’t need a separate
count or capacity. Every time we touch the code array, we make a
corresponding change to the line number array, starting with
initialization.

<div class="codehilite">

``` insert-before
  chunk->code = NULL;
```

<div class="source-file">

*chunk.c*  
in *initChunk*()

</div>

``` insert
  chunk->lines = NULL;
```

``` insert-after
  initValueArray(&chunk->constants);
```

</div>

<div class="source-file-narrow">

*chunk.c*, in *initChunk*()

</div>

And likewise deallocation:

<div class="codehilite">

``` insert-before
  FREE_ARRAY(uint8_t, chunk->code, chunk->capacity);
```

<div class="source-file">

*chunk.c*  
in *freeChunk*()

</div>

``` insert
  FREE_ARRAY(int, chunk->lines, chunk->capacity);
```

``` insert-after
  freeValueArray(&chunk->constants);
```

</div>

<div class="source-file-narrow">

*chunk.c*, in *freeChunk*()

</div>

When we write a byte of code to the chunk, we need to know what source
line it came from, so we add an extra parameter in the declaration of
`writeChunk()`.

<div class="codehilite">

``` insert-before
void freeChunk(Chunk* chunk);
```

<div class="source-file">

*chunk.h*  
function *writeChunk*()  
replace 1 line

</div>

``` insert
void writeChunk(Chunk* chunk, uint8_t byte, int line);
```

``` insert-after
int addConstant(Chunk* chunk, Value value);
```

</div>

<div class="source-file-narrow">

*chunk.h*, function *writeChunk*(), replace 1 line

</div>

And in the implementation:

<div class="codehilite">

<div class="source-file">

*chunk.c*  
function *writeChunk*()  
replace 1 line

</div>

``` insert
void writeChunk(Chunk* chunk, uint8_t byte, int line) {
```

``` insert-after
  if (chunk->capacity < chunk->count + 1) {
```

</div>

<div class="source-file-narrow">

*chunk.c*, function *writeChunk*(), replace 1 line

</div>

When we allocate or grow the code array, we do the same for the line
info too.

<div class="codehilite">

``` insert-before
    chunk->code = GROW_ARRAY(uint8_t, chunk->code,
        oldCapacity, chunk->capacity);
```

<div class="source-file">

*chunk.c*  
in *writeChunk*()

</div>

``` insert
    chunk->lines = GROW_ARRAY(int, chunk->lines,
        oldCapacity, chunk->capacity);
```

``` insert-after
  }
```

</div>

<div class="source-file-narrow">

*chunk.c*, in *writeChunk*()

</div>

Finally, we store the line number in the array.

<div class="codehilite">

``` insert-before
  chunk->code[chunk->count] = byte;
```

<div class="source-file">

*chunk.c*  
in *writeChunk*()

</div>

``` insert
  chunk->lines[chunk->count] = line;
```

``` insert-after
  chunk->count++;
```

</div>

<div class="source-file-narrow">

*chunk.c*, in *writeChunk*()

</div>

### <a href="#disassembling-line-information"
id="disassembling-line-information"><span
class="small">14 . 6 . 1</span>Disassembling line information</a>

Alright, let’s try this out with our little, uh, artisanal chunk. First,
since we added a new parameter to `writeChunk()`, we need to fix those
calls to pass in some<span class="em">—</span>arbitrary at this
point<span class="em">—</span>line number.

<div class="codehilite">

``` insert-before
  int constant = addConstant(&chunk, 1.2);
```

<div class="source-file">

*main.c*  
in *main*()  
replace 4 lines

</div>

``` insert
  writeChunk(&chunk, OP_CONSTANT, 123);
  writeChunk(&chunk, constant, 123);

  writeChunk(&chunk, OP_RETURN, 123);
```

``` insert-after

  disassembleChunk(&chunk, "test chunk");
```

</div>

<div class="source-file-narrow">

*main.c*, in *main*(), replace 4 lines

</div>

Once we have a real front end, of course, the compiler will track the
current line as it parses and pass that in.

Now that we have line information for every instruction, let’s put it to
good use. In our disassembler, it’s helpful to show which source line
each instruction was compiled from. That gives us a way to map back to
the original code when we’re trying to figure out what some blob of
bytecode is supposed to do. After printing the offset of the
instruction<span class="em">—</span>the number of bytes from the
beginning of the chunk<span class="em">—</span>we show its source line.

<div class="codehilite">

``` insert-before
int disassembleInstruction(Chunk* chunk, int offset) {
  printf("%04d ", offset);
```

<div class="source-file">

*debug.c*  
in *disassembleInstruction*()

</div>

``` insert
  if (offset > 0 &&
      chunk->lines[offset] == chunk->lines[offset - 1]) {
    printf("   | ");
  } else {
    printf("%4d ", chunk->lines[offset]);
  }
```

``` insert-after

  uint8_t instruction = chunk->code[offset];
```

</div>

<div class="source-file-narrow">

*debug.c*, in *disassembleInstruction*()

</div>

Bytecode instructions tend to be pretty fine-grained. A single line of
source code often compiles to a whole sequence of instructions. To make
that more visually clear, we show a `|` for any instruction that comes
from the same source line as the preceding one. The resulting output for
our handwritten chunk looks like:

<div class="codehilite">

    == test chunk ==
    0000  123 OP_CONSTANT         0 '1.2'
    0002    | OP_RETURN

</div>

We have a three-byte chunk. The first two bytes are a constant
instruction that loads 1.2 from the chunk’s constant pool. The first
byte is the `OP_CONSTANT` opcode and the second is the index in the
constant pool. The third byte (at offset 2) is a single-byte return
instruction.

In the remaining chapters, we will flesh this out with lots more kinds
of instructions. But the basic structure is here, and we have everything
we need now to completely represent an executable piece of code at
runtime in our virtual machine. Remember that whole family of AST
classes we defined in jlox? In clox, we’ve reduced that down to three
arrays: bytes of code, constant values, and line information for
debugging.

This reduction is a key reason why our new interpreter will be faster
than jlox. You can think of bytecode as a sort of compact serialization
of the AST, highly optimized for how the interpreter will deserialize it
in the order it needs as it executes. In the [next
chapter](a-virtual-machine.html), we will see how the virtual machine
does exactly that.

<div class="challenges">

## <a href="#challenges" id="challenges">Challenges</a>

1.  Our encoding of line information is hilariously wasteful of memory.
    Given that a series of instructions often correspond to the same
    source line, a natural solution is something akin to [run-length
    encoding](https://en.wikipedia.org/wiki/Run-length_encoding) of the
    line numbers.

    Devise an encoding that compresses the line information for a series
    of instructions on the same line. Change `writeChunk()` to write
    this compressed form, and implement a `getLine()` function that,
    given the index of an instruction, determines the line where the
    instruction occurs.

    *Hint: It’s not necessary for `getLine()` to be particularly
    efficient. Since it is called only when a runtime error occurs, it
    is well off the critical path where performance matters.*

2.  Because `OP_CONSTANT` uses only a single byte for its operand, a
    chunk may only contain up to 256 different constants. That’s small
    enough that people writing real-world code will hit that limit. We
    could use two or more bytes to store the operand, but that makes
    *every* constant instruction take up more space. Most chunks won’t
    need that many unique constants, so that wastes space and sacrifices
    some locality in the common case to support the rare case.

    To balance those two competing aims, many instruction sets feature
    multiple instructions that perform the same operation but with
    operands of different sizes. Leave our existing one-byte
    `OP_CONSTANT` instruction alone, and define a second
    `OP_CONSTANT_LONG` instruction. It stores the operand as a 24-bit
    number, which should be plenty.

    Implement this function:

    <div class="codehilite">

        void writeConstant(Chunk* chunk, Value value, int line) {
          // Implement me...
        }

    </div>

    It adds `value` to `chunk`’s constant array and then writes an
    appropriate instruction to load the constant. Also add support to
    the disassembler for `OP_CONSTANT_LONG` instructions.

    Defining two instructions seems to be the best of both worlds. What
    sacrifices, if any, does it force on us?

3.  Our `reallocate()` function relies on the C standard library for
    dynamic memory allocation and freeing. `malloc()` and `free()`
    aren’t magic. Find a couple of open source implementations of them
    and explain how they work. How do they keep track of which bytes are
    allocated and which are free? What is required to allocate a block
    of memory? Free it? How do they make that efficient? What do they do
    about fragmentation?

    *Hardcore mode:* Implement `reallocate()` without calling
    `realloc()`, `malloc()`, or `free()`. You are allowed to call
    `malloc()` *once*, at the beginning of the interpreter’s execution,
    to allocate a single big block of memory, which your `reallocate()`
    function has access to. It parcels out blobs of memory from that
    single region, your own personal heap. It’s your job to define how
    it does that.

</div>

<div class="design-note">

## <a href="#design-note" id="design-note">Design Note: Test Your
Language</a>

We’re almost halfway through the book and one thing we haven’t talked
about is *testing* your language implementation. That’s not because
testing isn’t important. I can’t possibly stress enough how vital it is
to have a good, comprehensive test suite for your language.

I wrote a [test suite for
Lox](https://github.com/munificent/craftinginterpreters/tree/master/test)
(which you are welcome to use on your own Lox implementation) before I
wrote a single word of this book. Those tests found countless bugs in my
implementations.

Tests are important in all software, but they’re even more important for
a programming language for at least a couple of reasons:

- **Users expect their programming languages to be rock solid.** We are
  so used to mature, stable compilers and interpreters that “It’s your
  code, not the compiler” is [an ingrained part of software
  culture](https://blog.codinghorror.com/the-first-rule-of-programming-its-always-your-fault/).
  If there are bugs in your language implementation, users will go
  through the full five stages of grief before they can figure out
  what’s going on, and you don’t want to put them through all that.

- **A language implementation is a deeply interconnected piece of
  software.** Some codebases are broad and shallow. If the file loading
  code is broken in your text editor,
  it<span class="em">—</span>hopefully\!<span class="em">—</span>won’t
  cause failures in the text rendering on screen. Language
  implementations are narrower and deeper, especially the core of the
  interpreter that handles the language’s actual semantics. That makes
  it easy for subtle bugs to creep in caused by weird interactions
  between various parts of the system. It takes good tests to flush
  those out.

- **The input to a language implementation is, by design,
  combinatorial.** There are an infinite number of possible programs a
  user could write, and your implementation needs to run them all
  correctly. You obviously can’t test that exhaustively, but you need to
  work hard to cover as much of the input space as you can.

- **Language implementations are often complex, constantly changing, and
  full of optimizations.** That leads to gnarly code with lots of dark
  corners where bugs can hide.

All of that means you’re gonna want a lot of tests. But *what* tests?
Projects I’ve seen focus mostly on end-to-end “language tests”. Each
test is a program written in the language along with the output or
errors it is expected to produce. Then you have a test runner that
pushes the test program through your language implementation and
validates that it does what it’s supposed to. Writing your tests in the
language itself has a few nice advantages:

- The tests aren’t coupled to any particular API or internal
  architecture decisions of the implementation. This frees you to
  reorganize or rewrite parts of your interpreter or compiler without
  needing to update a slew of tests.

- You can use the same tests for multiple implementations of the
  language.

- Tests can often be terse and easy to read and maintain since they are
  simply scripts in your language.

It’s not all rosy, though:

- End-to-end tests help you determine *if* there is a bug, but not
  *where* the bug is. It can be harder to figure out where the erroneous
  code in the implementation is because all the test tells you is that
  the right output didn’t appear.

- It can be a chore to craft a valid program that tickles some obscure
  corner of the implementation. This is particularly true for highly
  optimized compilers where you may need to write convoluted code to
  ensure that you end up on just the right optimization path where a bug
  may be hiding.

- The overhead can be high to fire up the interpreter, parse, compile,
  and run each test script. With a big suite of
  tests<span class="em">—</span>which you *do* want,
  remember<span class="em">—</span>that can mean a lot of time spent
  waiting for the tests to finish running.

I could go on, but I don’t want this to turn into a sermon. Also, I
don’t pretend to be an expert on *how* to test languages. I just want
you to internalize how important it is *that* you test yours. Seriously.
Test your language. You’ll thank me for it.

</div>

<a href="a-virtual-machine.html" class="next">Next Chapter: “A Virtual
Machine” →</a> Handcrafted by Robert Nystrom — <a
href="https://github.com/munificent/craftinginterpreters/blob/master/LICENSE"
target="_blank">© 2015 – 2021</a>

</div>

[![](image/logotype.png "Crafting Interpreters")](/)

<div class="contents">

### [A Virtual Machine<span class="small">15</span>](#top)

- [<span class="small">15.1</span> An Instruction Execution
  Machine](#an-instruction-execution-machine)
- [<span class="small">15.2</span> A Value Stack
  Manipulator](#a-value-stack-manipulator)
- [<span class="small">15.3</span> An Arithmetic
  Calculator](#an-arithmetic-calculator)
- 
- [Challenges](#challenges)
- [<span class="small">note</span>Register-Based Bytecode](#design-note)

<div class="prev-next">

<a href="chunks-of-bytecode.html" class="left"
title="Chunks of Bytecode">← Previous</a>
[↑ Up](a-bytecode-virtual-machine.html "A Bytecode Virtual Machine")
<a href="scanning-on-demand.html" class="right"
title="Scanning on Demand">Next →</a>

</div>

</div>

[![](image/logotype.png "Crafting Interpreters")](/)
<a href="chunks-of-bytecode.html" class="prev"
title="Chunks of Bytecode">←</a>
<a href="scanning-on-demand.html" class="next"
title="Scanning on Demand">→</a>

<div class="page">

<div class="nav-wrapper">

[![](image/logotype.png "Crafting Interpreters")](/)

<div class="expandable">

### [A Virtual Machine<span class="small">15</span>](#top)

- [<span class="small">15.1</span> An Instruction Execution
  Machine](#an-instruction-execution-machine)
- [<span class="small">15.2</span> A Value Stack
  Manipulator](#a-value-stack-manipulator)
- [<span class="small">15.3</span> An Arithmetic
  Calculator](#an-arithmetic-calculator)
- 
- [Challenges](#challenges)
- [<span class="small">note</span>Register-Based Bytecode](#design-note)

<div class="prev-next">

<a href="chunks-of-bytecode.html" class="left"
title="Chunks of Bytecode">← Previous</a>
[↑ Up](a-bytecode-virtual-machine.html "A Bytecode Virtual Machine")
<a href="scanning-on-demand.html" class="right"
title="Scanning on Demand">Next →</a>

</div>

</div>

<span id="expand-nav">≡</span>

</div>

<div class="number">

15

</div>

# A Virtual Machine

> Magicians protect their secrets not because the secrets are large and
> important, but because they are so small and trivial. The wonderful
> effects created on stage are often the result of a secret so absurd
> that the magician would be embarrassed to admit that that was how it
> was done.
>
> Christopher Priest, *The Prestige*

We’ve spent a lot of time talking about how to represent a program as a
sequence of bytecode instructions, but it feels like learning biology
using only stuffed, dead animals. We know what instructions are in
theory, but we’ve never seen them in action, so it’s hard to really
understand what they *do*. It would be hard to write a compiler that
outputs bytecode when we don’t have a good understanding of how that
bytecode behaves.

So, before we go and build the front end of our new interpreter, we will
begin with the back end<span class="em">—</span>the virtual machine that
executes instructions. It breathes life into the bytecode. Watching the
instructions prance around gives us a clearer picture of how a compiler
might translate the user’s source code into a series of them.

## <a href="#an-instruction-execution-machine"
id="an-instruction-execution-machine"><span
class="small">15 . 1</span>An Instruction Execution Machine</a>

The virtual machine is one part of our interpreter’s internal
architecture. You hand it a chunk of
code<span class="em">—</span>literally a
Chunk<span class="em">—</span>and it runs it. The code and data
structures for the VM reside in a new module.

<div class="codehilite">

<div class="source-file">

*vm.h*  
create new file

</div>

    #ifndef clox_vm_h
    #define clox_vm_h

    #include "chunk.h"

    typedef struct {
      Chunk* chunk;
    } VM;

    void initVM();
    void freeVM();

    #endif

</div>

<div class="source-file-narrow">

*vm.h*, create new file

</div>

As usual, we start simple. The VM will gradually acquire a whole pile of
state it needs to keep track of, so we define a struct now to stuff that
all in. Currently, all we store is the chunk that it executes.

Like we do with most of the data structures we create, we also define
functions to create and tear down a VM. Here’s the implementation:

<div class="codehilite">

<div class="source-file">

*vm.c*  
create new file

</div>

    #include "common.h"
    #include "vm.h"

    VM vm; 

    void initVM() {
    }

    void freeVM() {
    }

</div>

<div class="source-file-narrow">

*vm.c*, create new file

</div>

OK, calling those functions “implementations” is a stretch. We don’t
have any interesting state to initialize or free yet, so the functions
are empty. Trust me, we’ll get there.

The slightly more interesting line here is that declaration of `vm`.
This module is eventually going to have a slew of functions and it would
be a chore to pass around a pointer to the VM to all of them. Instead,
we declare a single global VM object. We need only one anyway, and this
keeps the code in the book a little lighter on the page.

The choice to have a static VM instance is a concession for the book,
but not necessarily a sound engineering choice for a real language
implementation. If you’re building a VM that’s designed to be embedded
in other host applications, it gives the host more flexibility if you
*do* explicitly take a VM pointer and pass it around.

That way, the host app can control when and where memory for the VM is
allocated, run multiple VMs in parallel, etc.

What I’m doing here is a global variable, and [everything bad you’ve
heard about global
variables](http://gameprogrammingpatterns.com/singleton.html) is still
true when programming in the large. But when keeping things small for a
book<span class="ellipse"> . . . </span>

Before we start pumping fun code into our VM, let’s go ahead and wire it
up to the interpreter’s main entrypoint.

<div class="codehilite">

``` insert-before
int main(int argc, const char* argv[]) {
```

<div class="source-file">

*main.c*  
in *main*()

</div>

``` insert
  initVM();
```

``` insert-after
  Chunk chunk;
```

</div>

<div class="source-file-narrow">

*main.c*, in *main*()

</div>

We spin up the VM when the interpreter first starts. Then when we’re
about to exit, we wind it down.

<div class="codehilite">

``` insert-before
  disassembleChunk(&chunk, "test chunk");
```

<div class="source-file">

*main.c*  
in *main*()

</div>

``` insert
  freeVM();
```

``` insert-after
  freeChunk(&chunk);
```

</div>

<div class="source-file-narrow">

*main.c*, in *main*()

</div>

One last ceremonial obligation:

<div class="codehilite">

``` insert-before
#include "debug.h"
```

<div class="source-file">

*main.c*

</div>

``` insert
#include "vm.h"
```

``` insert-after

int main(int argc, const char* argv[]) {
```

</div>

<div class="source-file-narrow">

*main.c*

</div>

Now when you run clox, it starts up the VM before it creates that
hand-authored chunk from the [last
chapter](chunks-of-bytecode.html#disassembling-chunks). The VM is ready
and waiting, so let’s teach it to do something.

### <a href="#executing-instructions" id="executing-instructions"><span
class="small">15 . 1 . 1</span>Executing instructions</a>

The VM springs into action when we command it to interpret a chunk of
bytecode.

<div class="codehilite">

``` insert-before
  disassembleChunk(&chunk, "test chunk");
```

<div class="source-file">

*main.c*  
in *main*()

</div>

``` insert
  interpret(&chunk);
```

``` insert-after
  freeVM();
```

</div>

<div class="source-file-narrow">

*main.c*, in *main*()

</div>

This function is the main entrypoint into the VM. It’s declared like so:

<div class="codehilite">

``` insert-before
void freeVM();
```

<div class="source-file">

*vm.h*  
add after *freeVM*()

</div>

``` insert
InterpretResult interpret(Chunk* chunk);
```

``` insert-after

#endif
```

</div>

<div class="source-file-narrow">

*vm.h*, add after *freeVM*()

</div>

The VM runs the chunk and then responds with a value from this enum:

<div class="codehilite">

``` insert-before
} VM;
```

<div class="source-file">

*vm.h*  
add after struct *VM*

</div>

``` insert
typedef enum {
  INTERPRET_OK,
  INTERPRET_COMPILE_ERROR,
  INTERPRET_RUNTIME_ERROR
} InterpretResult;
```

``` insert-after
void initVM();
void freeVM();
```

</div>

<div class="source-file-narrow">

*vm.h*, add after struct *VM*

</div>

We aren’t using the result yet, but when we have a compiler that reports
static errors and a VM that detects runtime errors, the interpreter will
use this to know how to set the exit code of the process.

We’re inching towards some actual implementation.

<div class="codehilite">

<div class="source-file">

*vm.c*  
add after *freeVM*()

</div>

    InterpretResult interpret(Chunk* chunk) {
      vm.chunk = chunk;
      vm.ip = vm.chunk->code;
      return run();
    }

</div>

<div class="source-file-narrow">

*vm.c*, add after *freeVM*()

</div>

First, we store the chunk being executed in the VM. Then we call
`run()`, an internal helper function that actually runs the bytecode
instructions. Between those two parts is an intriguing line. What is
this `ip` business?

As the VM works its way through the bytecode, it keeps track of where it
is<span class="em">—</span>the location of the instruction currently
being executed. We don’t use a <span id="local">local</span> variable
inside `run()` for this because eventually other functions will need to
access it. Instead, we store it as a field in VM.

If we were trying to squeeze every ounce of speed out of our bytecode
interpreter, we would store `ip` in a local variable. It gets modified
so often during execution that we want the C compiler to keep it in a
register.

<div class="codehilite">

``` insert-before
typedef struct {
  Chunk* chunk;
```

<div class="source-file">

*vm.h*  
in struct *VM*

</div>

``` insert
  uint8_t* ip;
```

``` insert-after
} VM;
```

</div>

<div class="source-file-narrow">

*vm.h*, in struct *VM*

</div>

Its type is a byte pointer. We use an actual real C pointer pointing
right into the middle of the bytecode array instead of something like an
integer index because it’s faster to dereference a pointer than look up
an element in an array by index.

The name “IP” is traditional, and<span class="em">—</span>unlike many
traditional names in CS<span class="em">—</span>actually makes sense:
it’s an **[instruction
pointer](https://en.wikipedia.org/wiki/Program_counter)**. Almost every
instruction set in the <span id="ip">world</span>, real and virtual, has
a register or variable like this.

x86, x64, and the CLR call it “IP”. 68k, PowerPC, ARM, p-code, and the
JVM call it “PC”, for **program counter**.

We initialize `ip` by pointing it at the first byte of code in the
chunk. We haven’t executed that instruction yet, so `ip` points to the
instruction *about to be executed*. This will be true during the entire
time the VM is running: the IP always points to the next instruction,
not the one currently being handled.

The real fun happens in `run`().

<div class="codehilite">

<div class="source-file">

*vm.c*  
add after *freeVM*()

</div>

    static InterpretResult run() {
    #define READ_BYTE() (*vm.ip++)

      for (;;) {
        uint8_t instruction;
        switch (instruction = READ_BYTE()) {
          case OP_RETURN: {
            return INTERPRET_OK;
          }
        }
      }

    #undef READ_BYTE
    }

</div>

<div class="source-file-narrow">

*vm.c*, add after *freeVM*()

</div>

This is the single most <span id="important">important</span> function
in all of clox, by far. When the interpreter executes a user’s program,
it will spend something like 90% of its time inside `run()`. It is the
beating heart of the VM.

Or, at least, it *will* be in a few chapters when it has enough content
to be useful. Right now, it’s not exactly a wonder of software wizardry.

Despite that dramatic intro, it’s conceptually pretty simple. We have an
outer loop that goes and goes. Each turn through that loop, we read and
execute a single bytecode instruction.

To process an instruction, we first figure out what kind of instruction
we’re dealing with. The `READ_BYTE` macro reads the byte currently
pointed at by `ip` and then <span id="next">advances</span> the
instruction pointer. The first byte of any instruction is the opcode.
Given a numeric opcode, we need to get to the right C code that
implements that instruction’s semantics. This process is called
**decoding** or **dispatching** the instruction.

Note that `ip` advances as soon as we read the opcode, before we’ve
actually started executing the instruction. So, again, `ip` points to
the *next* byte of code to be used.

We do that process for every single instruction, every single time one
is executed, so this is the most performance critical part of the entire
virtual machine. Programming language lore is filled with
<span id="dispatch">clever</span> techniques to do bytecode dispatch
efficiently, going all the way back to the early days of computers.

If you want to learn some of these techniques, look up “direct threaded
code”, “jump table”, and “computed goto”.

Alas, the fastest solutions require either non-standard extensions to C,
or handwritten assembly code. For clox, we’ll keep it simple. Just like
our disassembler, we have a single giant `switch` statement with a case
for each opcode. The body of each case implements that opcode’s
behavior.

So far, we handle only a single instruction, `OP_RETURN`, and the only
thing it does is exit the loop entirely. Eventually, that instruction
will be used to return from the current Lox function, but we don’t have
functions yet, so we’ll repurpose it temporarily to end the execution.

Let’s go ahead and support our one other instruction.

<div class="codehilite">

``` insert-before
    switch (instruction = READ_BYTE()) {
```

<div class="source-file">

*vm.c*  
in *run*()

</div>

``` insert
      case OP_CONSTANT: {
        Value constant = READ_CONSTANT();
        printValue(constant);
        printf("\n");
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

We don’t have enough machinery in place yet to do anything useful with a
constant. For now, we’ll just print it out so we interpreter hackers can
see what’s going on inside our VM. That call to `printf()` necessitates
an include.

<div class="codehilite">

<div class="source-file">

*vm.c*  
add to top of file

</div>

``` insert
#include <stdio.h>
```

``` insert-after
#include "common.h"
```

</div>

<div class="source-file-narrow">

*vm.c*, add to top of file

</div>

We also have a new macro to define.

<div class="codehilite">

``` insert-before
#define READ_BYTE() (*vm.ip++)
```

<div class="source-file">

*vm.c*  
in *run*()

</div>

``` insert
#define READ_CONSTANT() (vm.chunk->constants.values[READ_BYTE()])
```

``` insert-after

  for (;;) {
```

</div>

<div class="source-file-narrow">

*vm.c*, in *run*()

</div>

`READ_CONSTANT()` reads the next byte from the bytecode, treats the
resulting number as an index, and looks up the corresponding Value in
the chunk’s constant table. In later chapters, we’ll add a few more
instructions with operands that refer to constants, so we’re setting up
this helper macro now.

Like the previous `READ_BYTE` macro, `READ_CONSTANT` is only used inside
`run()`. To make that scoping more explicit, the macro definitions
themselves are confined to that function. We
<span id="macro">define</span> them at the beginning
and<span class="em">—</span>because we
care<span class="em">—</span>undefine them at the end.

<div class="codehilite">

``` insert-before
#undef READ_BYTE
```

<div class="source-file">

*vm.c*  
in *run*()

</div>

``` insert
#undef READ_CONSTANT
```

``` insert-after
}
```

</div>

<div class="source-file-narrow">

*vm.c*, in *run*()

</div>

Undefining these macros explicitly might seem needlessly fastidious, but
C tends to punish sloppy users, and the C preprocessor doubly so.

### <a href="#execution-tracing" id="execution-tracing"><span
class="small">15 . 1 . 2</span>Execution tracing</a>

If you run clox now, it executes the chunk we hand-authored in the last
chapter and spits out `1.2` to your terminal. We can see that it’s
working, but that’s only because our implementation of `OP_CONSTANT` has
temporary code to log the value. Once that instruction is doing what
it’s supposed to do and plumbing that constant along to other operations
that want to consume it, the VM will become a black box. That makes our
lives as VM implementers harder.

To help ourselves out, now is a good time to add some diagnostic logging
to the VM like we did with chunks themselves. In fact, we’ll even reuse
the same code. We don’t want this logging enabled all the
time<span class="em">—</span>it’s just for us VM hackers, not Lox
users<span class="em">—</span>so first we create a flag to hide it
behind.

<div class="codehilite">

``` insert-before
#include <stdint.h>
```

<div class="source-file">

*common.h*

</div>

``` insert

#define DEBUG_TRACE_EXECUTION
```

``` insert-after

#endif
```

</div>

<div class="source-file-narrow">

*common.h*

</div>

When this flag is defined, the VM disassembles and prints each
instruction right before executing it. Where our previous disassembler
walked an entire chunk once, statically, this disassembles instructions
dynamically, on the fly.

<div class="codehilite">

``` insert-before
  for (;;) {
```

<div class="source-file">

*vm.c*  
in *run*()

</div>

``` insert
#ifdef DEBUG_TRACE_EXECUTION
    disassembleInstruction(vm.chunk,
                           (int)(vm.ip - vm.chunk->code));
#endif
```

``` insert-after
    uint8_t instruction;
```

</div>

<div class="source-file-narrow">

*vm.c*, in *run*()

</div>

Since `disassembleInstruction()` takes an integer byte *offset* and we
store the current instruction reference as a direct pointer, we first do
a little pointer math to convert `ip` back to a relative offset from the
beginning of the bytecode. Then we disassemble the instruction that
begins at that byte.

As ever, we need to bring in the declaration of the function before we
can call it.

<div class="codehilite">

``` insert-before
#include "common.h"
```

<div class="source-file">

*vm.c*

</div>

``` insert
#include "debug.h"
```

``` insert-after
#include "vm.h"
```

</div>

<div class="source-file-narrow">

*vm.c*

</div>

I know this code isn’t super impressive so
far<span class="em">—</span>it’s literally a switch statement wrapped in
a `for` loop but, believe it or not, this is one of the two major
components of our VM. With this, we can imperatively execute
instructions. Its simplicity is a virtue<span class="em">—</span>the
less work it does, the faster it can do it. Contrast this with all of
the complexity and overhead we had in jlox with the Visitor pattern for
walking the AST.

## <a href="#a-value-stack-manipulator"
id="a-value-stack-manipulator"><span class="small">15 . 2</span>A Value
Stack Manipulator</a>

In addition to imperative side effects, Lox has expressions that
produce, modify, and consume values. Thus, our compiled bytecode needs a
way to shuttle values around between the different instructions that
need them. For example:

<div class="codehilite">

    print 3 - 2;

</div>

We obviously need instructions for the constants 3 and 2, the `print`
statement, and the subtraction. But how does the subtraction instruction
know that 3 is the <span id="word">minuend</span> and 2 is the
subtrahend? How does the print instruction know to print the result of
that?

Yes, I did have to look up “subtrahend” and “minuend” in a dictionary.
But aren’t they delightful words? “Minuend” sounds like a kind of
Elizabethan dance and “subtrahend” might be some sort of underground
Paleolithic monument.

To put a finer point on it, look at this thing right here:

<div class="codehilite">

    fun echo(n) {
      print n;
      return n;
    }

    print echo(echo(1) + echo(2)) + echo(echo(4) + echo(5));

</div>

I wrapped each subexpression in a call to `echo()` that prints and
returns its argument. That side effect means we can see the exact order
of operations.

Don’t worry about the VM for a minute. Think about just the semantics of
Lox itself. The operands to an arithmetic operator obviously need to be
evaluated before we can perform the operation itself. (It’s pretty hard
to add `a + b` if you don’t know what `a` and `b` are.) Also, when we
implemented expressions in jlox, we <span id="undefined">decided</span>
that the left operand must be evaluated before the right.

We could have left evaluation order unspecified and let each
implementation decide. That leaves the door open for optimizing
compilers to reorder arithmetic expressions for efficiency, even in
cases where the operands have visible side effects. C and Scheme leave
evaluation order unspecified. Java specifies left-to-right evaluation
like we do for Lox.

I think nailing down stuff like this is generally better for users. When
expressions are not evaluated in the order users
intuit<span class="em">—</span>possibly in different orders across
different implementations\!<span class="em">—</span>it can be a burning
hellscape of pain to figure out what’s going on.

Here is the syntax tree for the `print` statement:

![The AST for the example statement, with numbers marking the order that
the nodes are evaluated.](image/a-virtual-machine/ast.png)

Given left-to-right evaluation, and the way the expressions are nested,
any correct Lox implementation *must* print these numbers in this order:

<div class="codehilite">

    1  // from echo(1)
    2  // from echo(2)
    3  // from echo(1 + 2)
    4  // from echo(4)
    5  // from echo(5)
    9  // from echo(4 + 5)
    12 // from print 3 + 9

</div>

Our old jlox interpreter accomplishes this by recursively traversing the
AST. It does a postorder traversal. First it recurses down the left
operand branch, then the right operand, then finally it evaluates the
node itself.

After evaluating the left operand, jlox needs to store that result
somewhere temporarily while it’s busy traversing down through the right
operand tree. We use a local variable in Java for that. Our recursive
tree-walk interpreter creates a unique Java call frame for each node
being evaluated, so we could have as many of these local variables as we
needed.

In clox, our `run()` function is not
recursive<span class="em">—</span>the nested expression tree is
flattened out into a linear series of instructions. We don’t have the
luxury of using C local variables, so how and where should we store
these temporary values? You can probably <span id="guess">guess</span>
already, but I want to really drill into this because it’s an aspect of
programming that we take for granted, but we rarely learn *why*
computers are architected this way.

Hint: it’s in the name of this section, and it’s how Java and C manage
recursive calls to functions.

Let’s do a weird exercise. We’ll walk through the execution of the above
program a step at a time:

![The series of instructions with bars showing which numbers need to be
preserved across which instructions.](image/a-virtual-machine/bars.png)

On the left are the steps of code. On the right are the values we’re
tracking. Each bar represents a number. It starts when the value is
first produced<span class="em">—</span>either a constant or the result
of an addition. The length of the bar tracks when a previously produced
value needs to be kept around, and it ends when that value finally gets
consumed by an operation.

As you step through, you see values appear and then later get eaten. The
longest-lived ones are the values produced from the left-hand side of an
addition. Those stick around while we work through the right-hand
operand expression.

In the above diagram, I gave each unique number its own visual column.
Let’s be a little more parsimonious. Once a number is consumed, we allow
its column to be reused for another later value. In other words, we take
all of those gaps up there and fill them in, pushing in numbers from the
right:

![Like the previous diagram, but with number bars pushed to the left,
forming a stack.](image/a-virtual-machine/bars-stacked.png)

There’s some interesting stuff going on here. When we shift everything
over, each number still manages to stay in a single column for its
entire life. Also, there are no gaps left. In other words, whenever a
number appears earlier than another, then it will live at least as long
as that second one. The first number to appear is the last to be
consumed. Hmm<span class="ellipse"> . . . </span>last-in,
first-out<span class="ellipse"> . . . </span>why, that’s a
<span id="pancakes">stack</span>!

This is also a stack:

![A stack... of pancakes.](image/a-virtual-machine/pancakes.png)

In the second diagram, each time we introduce a number, we push it onto
the stack from the right. When numbers are consumed, they are always
popped off from rightmost to left.

Since the temporary values we need to track naturally have stack-like
behavior, our VM will use a stack to manage them. When an instruction
“produces” a value, it pushes it onto the stack. When it needs to
consume one or more values, it gets them by popping them off the stack.

### <a href="#the-vms-stack" id="the-vms-stack"><span
class="small">15 . 2 . 1</span>The VM’s Stack</a>

Maybe this doesn’t seem like a revelation, but I *love* stack-based VMs.
When you first see a magic trick, it feels like something actually
magical. But then you learn how it works<span class="em">—</span>usually
some mechanical gimmick or misdirection<span class="em">—</span>and the
sense of wonder evaporates. There are a <span id="wonder">couple</span>
of ideas in computer science where even after I pulled them apart and
learned all the ins and outs, some of the initial sparkle remained.
Stack-based VMs are one of those.

Heaps<span class="em">—</span>[the data
structure](https://en.wikipedia.org/wiki/Heap_(data_structure)), not
[the memory management
thing](https://en.wikipedia.org/wiki/Memory_management#HEAP)<span class="em">—</span>are
another. And Vaughan Pratt’s top-down operator precedence parsing
scheme, which we’ll learn about [in due
time](compiling-expressions.html).

As you’ll see in this chapter, executing instructions in a stack-based
VM is dead <span id="cheat">simple</span>. In later chapters, you’ll
also discover that compiling a source language to a stack-based
instruction set is a piece of cake. And yet, this architecture is fast
enough to be used by production language implementations. It almost
feels like cheating at the programming language game.

To take a bit of the sheen off: stack-based interpreters aren’t a silver
bullet. They’re often *adequate*, but modern implementations of the JVM,
the CLR, and JavaScript all use sophisticated [just-in-time
compilation](https://en.wikipedia.org/wiki/Just-in-time_compilation)
pipelines to generate *much* faster native code on the fly.

Alrighty, it’s codin’ time! Here’s the stack:

<div class="codehilite">

``` insert-before
typedef struct {
  Chunk* chunk;
  uint8_t* ip;
```

<div class="source-file">

*vm.h*  
in struct *VM*

</div>

``` insert
  Value stack[STACK_MAX];
  Value* stackTop;
```

``` insert-after
} VM;
```

</div>

<div class="source-file-narrow">

*vm.h*, in struct *VM*

</div>

We implement the stack semantics ourselves on top of a raw C array. The
bottom of the stack<span class="em">—</span>the first value pushed and
the last to be popped<span class="em">—</span>is at element zero in the
array, and later pushed values follow it. If we push the letters of
“crepe”<span class="em">—</span>my favorite stackable breakfast
item<span class="em">—</span>onto the stack, in order, the resulting C
array looks like this:

![An array containing the letters in 'crepe' in order starting at
element 0.](image/a-virtual-machine/array.png)

Since the stack grows and shrinks as values are pushed and popped, we
need to track where the top of the stack is in the array. As with `ip`,
we use a direct pointer instead of an integer index since it’s faster to
dereference the pointer than calculate the offset from the index each
time we need it.

The pointer points at the array element just *past* the element
containing the top value on the stack. That seems a little odd, but
almost every implementation does this. It means we can indicate that the
stack is empty by pointing at element zero in the array.

![An empty array with stackTop pointing at the first
element.](image/a-virtual-machine/stack-empty.png)

If we pointed to the top element, then for an empty stack we’d need to
point at element -1. That’s <span id="defined">undefined</span> in C. As
we push values onto the stack<span class="ellipse"> . . . </span>

What about when the stack is *full*, you ask, Clever Reader? The C
standard is one step ahead of you. It *is* allowed and well-specified to
have an array pointer that points just past the end of an array.

![An array with 'c' at element
zero.](image/a-virtual-machine/stack-c.png)

<span class="ellipse"> . . . </span>`stackTop` always points just past
the last item.

![An array with 'c', 'r', 'e', 'p', and 'e' in the first five
elements.](image/a-virtual-machine/stack-crepe.png)

I remember it like this: `stackTop` points to where the next value to be
pushed will go. The maximum number of values we can store on the stack
(for now, at least) is:

<div class="codehilite">

``` insert-before
#include "chunk.h"
```

<div class="source-file">

*vm.h*

</div>

``` insert

#define STACK_MAX 256
```

``` insert-after

typedef struct {
```

</div>

<div class="source-file-narrow">

*vm.h*

</div>

Giving our VM a fixed stack size means it’s possible for some sequence
of instructions to push too many values and run out of stack
space<span class="em">—</span>the classic “stack overflow”. We could
grow the stack dynamically as needed, but for now we’ll keep it simple.
Since VM uses Value, we need to include its declaration.

<div class="codehilite">

``` insert-before
#include "chunk.h"
```

<div class="source-file">

*vm.h*

</div>

``` insert
#include "value.h"
```

``` insert-after

#define STACK_MAX 256
```

</div>

<div class="source-file-narrow">

*vm.h*

</div>

Now that VM has some interesting state, we get to initialize it.

<div class="codehilite">

``` insert-before
void initVM() {
```

<div class="source-file">

*vm.c*  
in *initVM*()

</div>

``` insert
  resetStack();
```

``` insert-after
}
```

</div>

<div class="source-file-narrow">

*vm.c*, in *initVM*()

</div>

That uses this helper function:

<div class="codehilite">

<div class="source-file">

*vm.c*  
add after variable *vm*

</div>

    static void resetStack() {
      vm.stackTop = vm.stack;
    }

</div>

<div class="source-file-narrow">

*vm.c*, add after variable *vm*

</div>

Since the stack array is declared directly inline in the VM struct, we
don’t need to allocate it. We don’t even need to clear the unused cells
in the array<span class="em">—</span>we simply won’t access them until
after values have been stored in them. The only initialization we need
is to set `stackTop` to point to the beginning of the array to indicate
that the stack is empty.

The stack protocol supports two operations:

<div class="codehilite">

``` insert-before
InterpretResult interpret(Chunk* chunk);
```

<div class="source-file">

*vm.h*  
add after *interpret*()

</div>

``` insert
void push(Value value);
Value pop();
```

``` insert-after

#endif
```

</div>

<div class="source-file-narrow">

*vm.h*, add after *interpret*()

</div>

You can push a new value onto the top of the stack, and you can pop the
most recently pushed value back off. Here’s the first function:

<div class="codehilite">

<div class="source-file">

*vm.c*  
add after *freeVM*()

</div>

    void push(Value value) {
      *vm.stackTop = value;
      vm.stackTop++;
    }

</div>

<div class="source-file-narrow">

*vm.c*, add after *freeVM*()

</div>

If you’re rusty on your C pointer syntax and operations, this is a good
warm-up. The first line stores `value` in the array element at the top
of the stack. Remember, `stackTop` points just *past* the last used
element, at the next available one. This stores the value in that slot.
Then we increment the pointer itself to point to the next unused slot in
the array now that the previous slot is occupied.

Popping is the mirror image.

<div class="codehilite">

<div class="source-file">

*vm.c*  
add after *push*()

</div>

    Value pop() {
      vm.stackTop--;
      return *vm.stackTop;
    }

</div>

<div class="source-file-narrow">

*vm.c*, add after *push*()

</div>

First, we move the stack pointer *back* to get to the most recent used
slot in the array. Then we look up the value at that index and return
it. We don’t need to explicitly “remove” it from the
array<span class="em">—</span>moving `stackTop` down is enough to mark
that slot as no longer in use.

### <a href="#stack-tracing" id="stack-tracing"><span
class="small">15 . 2 . 2</span>Stack tracing</a>

We have a working stack, but it’s hard to *see* that it’s working. When
we start implementing more complex instructions and compiling and
running larger pieces of code, we’ll end up with a lot of values crammed
into that array. It would make our lives as VM hackers easier if we had
some visibility into the stack.

To that end, whenever we’re tracing execution, we’ll also show the
current contents of the stack before we interpret each instruction.

<div class="codehilite">

``` insert-before
#ifdef DEBUG_TRACE_EXECUTION
```

<div class="source-file">

*vm.c*  
in *run*()

</div>

``` insert
    printf("          ");
    for (Value* slot = vm.stack; slot < vm.stackTop; slot++) {
      printf("[ ");
      printValue(*slot);
      printf(" ]");
    }
    printf("\n");
```

``` insert-after
    disassembleInstruction(vm.chunk,
```

</div>

<div class="source-file-narrow">

*vm.c*, in *run*()

</div>

We loop, printing each value in the array, starting at the first (bottom
of the stack) and ending when we reach the top. This lets us observe the
effect of each instruction on the stack. The output is pretty verbose,
but it’s useful when we’re surgically extracting a nasty bug from the
bowels of the interpreter.

Stack in hand, let’s revisit our two instructions. First up:

<div class="codehilite">

``` insert-before
      case OP_CONSTANT: {
        Value constant = READ_CONSTANT();
```

<div class="source-file">

*vm.c*  
in *run*()  
replace 2 lines

</div>

``` insert
        push(constant);
```

``` insert-after
        break;
```

</div>

<div class="source-file-narrow">

*vm.c*, in *run*(), replace 2 lines

</div>

In the last chapter, I was hand-wavey about how the `OP_CONSTANT`
instruction “loads” a constant. Now that we have a stack you know what
it means to actually produce a value: it gets pushed onto the stack.

<div class="codehilite">

``` insert-before
      case OP_RETURN: {
```

<div class="source-file">

*vm.c*  
in *run*()

</div>

``` insert
        printValue(pop());
        printf("\n");
```

``` insert-after
        return INTERPRET_OK;
```

</div>

<div class="source-file-narrow">

*vm.c*, in *run*()

</div>

Then we make `OP_RETURN` pop the stack and print the top value before
exiting. When we add support for real functions to clox, we’ll change
this code. But, for now, it gives us a way to get the VM executing
simple instruction sequences and displaying the result.

## <a href="#an-arithmetic-calculator" id="an-arithmetic-calculator"><span
class="small">15 . 3</span>An Arithmetic Calculator</a>

The heart and soul of our VM are in place now. The bytecode loop
dispatches and executes instructions. The stack grows and shrinks as
values flow through it. The two halves work, but it’s hard to get a feel
for how cleverly they interact with only the two rudimentary
instructions we have so far. So let’s teach our interpreter to do
arithmetic.

We’ll start with the simplest arithmetic operation, unary negation.

<div class="codehilite">

    var a = 1.2;
    print -a; // -1.2.

</div>

The prefix `-` operator takes one operand, the value to negate. It
produces a single result. We aren’t fussing with a parser yet, but we
can add the bytecode instruction that the above syntax will compile to.

<div class="codehilite">

``` insert-before
  OP_CONSTANT,
```

<div class="source-file">

*chunk.h*  
in enum *OpCode*

</div>

``` insert
  OP_NEGATE,
```

``` insert-after
  OP_RETURN,
```

</div>

<div class="source-file-narrow">

*chunk.h*, in enum *OpCode*

</div>

We execute it like so:

<div class="codehilite">

``` insert-before
      }
```

<div class="source-file">

*vm.c*  
in *run*()

</div>

``` insert
      case OP_NEGATE:   push(-pop()); break;
```

``` insert-after
      case OP_RETURN: {
```

</div>

<div class="source-file-narrow">

*vm.c*, in *run*()

</div>

The instruction needs a value to operate on, which it gets by popping
from the stack. It negates that, then pushes the result back on for
later instructions to use. Doesn’t get much easier than that. We can
disassemble it too.

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
    case OP_NEGATE:
      return simpleInstruction("OP_NEGATE", offset);
```

``` insert-after
    case OP_RETURN:
```

</div>

<div class="source-file-narrow">

*debug.c*, in *disassembleInstruction*()

</div>

And we can try it out in our test chunk.

<div class="codehilite">

``` insert-before
  writeChunk(&chunk, constant, 123);
```

<div class="source-file">

*main.c*  
in *main*()

</div>

``` insert
  writeChunk(&chunk, OP_NEGATE, 123);
```

``` insert-after

  writeChunk(&chunk, OP_RETURN, 123);
```

</div>

<div class="source-file-narrow">

*main.c*, in *main*()

</div>

After loading the constant, but before returning, we execute the negate
instruction. That replaces the constant on the stack with its negation.
Then the return instruction prints that out:

<div class="codehilite">

    -1.2

</div>

Magical!

### <a href="#binary-operators" id="binary-operators"><span
class="small">15 . 3 . 1</span>Binary operators</a>

OK, unary operators aren’t *that* impressive. We still only ever have a
single value on the stack. To really see some depth, we need binary
operators. Lox has four binary <span id="ops">arithmetic</span>
operators: addition, subtraction, multiplication, and division. We’ll go
ahead and implement them all at the same time.

Lox has some other binary operators<span class="em">—</span>comparison
and equality<span class="em">—</span>but those don’t produce numbers as
a result, so we aren’t ready for them yet.

<div class="codehilite">

``` insert-before
  OP_CONSTANT,
```

<div class="source-file">

*chunk.h*  
in enum *OpCode*

</div>

``` insert
  OP_ADD,
  OP_SUBTRACT,
  OP_MULTIPLY,
  OP_DIVIDE,
```

``` insert-after
  OP_NEGATE,
```

</div>

<div class="source-file-narrow">

*chunk.h*, in enum *OpCode*

</div>

Back in the bytecode loop, they are executed like this:

<div class="codehilite">

``` insert-before
      }
```

<div class="source-file">

*vm.c*  
in *run*()

</div>

``` insert
      case OP_ADD:      BINARY_OP(+); break;
      case OP_SUBTRACT: BINARY_OP(-); break;
      case OP_MULTIPLY: BINARY_OP(*); break;
      case OP_DIVIDE:   BINARY_OP(/); break;
```

``` insert-after
      case OP_NEGATE:   push(-pop()); break;
```

</div>

<div class="source-file-narrow">

*vm.c*, in *run*()

</div>

The only difference between these four instructions is which underlying
C operator they ultimately use to combine the two operands. Surrounding
that core arithmetic expression is some boilerplate code to pull values
off the stack and push the result. When we later add dynamic typing,
that boilerplate will grow. To avoid repeating that code four times, I
wrapped it up in a macro.

<div class="codehilite">

``` insert-before
#define READ_CONSTANT() (vm.chunk->constants.values[READ_BYTE()])
```

<div class="source-file">

*vm.c*  
in *run*()

</div>

``` insert
#define BINARY_OP(op) \
    do { \
      double b = pop(); \
      double a = pop(); \
      push(a op b); \
    } while (false)
```

``` insert-after

  for (;;) {
```

</div>

<div class="source-file-narrow">

*vm.c*, in *run*()

</div>

I admit this is a fairly <span id="operator">adventurous</span> use of
the C preprocessor. I hesitated to do this, but you’ll be glad in later
chapters when we need to add the type checking for each operand and
stuff. It would be a chore to walk you through the same code four times.

Did you even know you can pass an *operator* as an argument to a macro?
Now you do. The preprocessor doesn’t care that operators aren’t first
class in C. As far as it’s concerned, it’s all just text tokens.

I know, you can just *feel* the temptation to abuse this, can’t you?

If you aren’t familiar with the trick already, that outer `do while`
loop probably looks really weird. This macro needs to expand to a series
of statements. To be careful macro authors, we want to ensure those
statements all end up in the same scope when the macro is expanded.
Imagine if you defined:

<div class="codehilite">

    #define WAKE_UP() makeCoffee(); drinkCoffee();

</div>

And then used it like:

<div class="codehilite">

    if (morning) WAKE_UP();

</div>

The intent is to execute both statements of the macro body only if
`morning` is true. But it expands to:

<div class="codehilite">

    if (morning) makeCoffee(); drinkCoffee();;

</div>

Oops. The `if` attaches only to the *first* statement. You might think
you could fix this using a block.

<div class="codehilite">

    #define WAKE_UP() { makeCoffee(); drinkCoffee(); }

</div>

That’s better, but you still risk:

<div class="codehilite">

    if (morning)
      WAKE_UP();
    else
      sleepIn();

</div>

Now you get a compile error on the `else` because of that trailing `;`
after the macro’s block. Using a `do while` loop in the macro looks
funny, but it gives you a way to contain multiple statements inside a
block that *also* permits a semicolon at the end.

Where were we? Right, so what the body of that macro does is
straightforward. A binary operator takes two operands, so it pops twice.
It performs the operation on those two values and then pushes the
result.

Pay close attention to the *order* of the two pops. Note that we assign
the first popped operand to `b`, not `a`. It looks backwards. When the
operands themselves are calculated, the left is evaluated first, then
the right. That means the left operand gets pushed before the right
operand. So the right operand will be on top of the stack. Thus, the
first value we pop is `b`.

For example, if we compile `3 - 1`, the data flow between the
instructions looks like so:

![A sequence of instructions with the stack for each showing how pushing
and then popping values reverses their
order.](image/a-virtual-machine/reverse.png)

As we did with the other macros inside `run()`, we clean up after
ourselves at the end of the function.

<div class="codehilite">

``` insert-before
#undef READ_CONSTANT
```

<div class="source-file">

*vm.c*  
in *run*()

</div>

``` insert
#undef BINARY_OP
```

``` insert-after
}
```

</div>

<div class="source-file-narrow">

*vm.c*, in *run*()

</div>

Last is disassembler support.

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
    case OP_ADD:
      return simpleInstruction("OP_ADD", offset);
    case OP_SUBTRACT:
      return simpleInstruction("OP_SUBTRACT", offset);
    case OP_MULTIPLY:
      return simpleInstruction("OP_MULTIPLY", offset);
    case OP_DIVIDE:
      return simpleInstruction("OP_DIVIDE", offset);
```

``` insert-after
    case OP_NEGATE:
```

</div>

<div class="source-file-narrow">

*debug.c*, in *disassembleInstruction*()

</div>

The arithmetic instruction formats are simple, like `OP_RETURN`. Even
though the arithmetic *operators* take
operands<span class="em">—</span>which are found on the
stack<span class="em">—</span>the arithmetic *bytecode instructions* do
not.

Let’s put some of our new instructions through their paces by evaluating
a larger expression:

![The expression being evaluated: -((1.2 + 3.4) /
5.6)](image/a-virtual-machine/chunk.png)

Building on our existing example chunk, here’s the additional
instructions we need to hand-compile that AST to bytecode.

<div class="codehilite">

``` insert-before
  int constant = addConstant(&chunk, 1.2);
  writeChunk(&chunk, OP_CONSTANT, 123);
  writeChunk(&chunk, constant, 123);
```

<div class="source-file">

*main.c*  
in *main*()

</div>

``` insert

  constant = addConstant(&chunk, 3.4);
  writeChunk(&chunk, OP_CONSTANT, 123);
  writeChunk(&chunk, constant, 123);

  writeChunk(&chunk, OP_ADD, 123);

  constant = addConstant(&chunk, 5.6);
  writeChunk(&chunk, OP_CONSTANT, 123);
  writeChunk(&chunk, constant, 123);

  writeChunk(&chunk, OP_DIVIDE, 123);
```

``` insert-after
  writeChunk(&chunk, OP_NEGATE, 123);

  writeChunk(&chunk, OP_RETURN, 123);
```

</div>

<div class="source-file-narrow">

*main.c*, in *main*()

</div>

The addition goes first. The instruction for the left constant, 1.2, is
already there, so we add another for 3.4. Then we add those two using
`OP_ADD`, leaving it on the stack. That covers the left side of the
division. Next we push the 5.6, and divide the result of the addition by
it. Finally, we negate the result of that.

Note how the output of the `OP_ADD` implicitly flows into being an
operand of `OP_DIVIDE` without either instruction being directly coupled
to each other. That’s the magic of the stack. It lets us freely compose
instructions without them needing any complexity or awareness of the
data flow. The stack acts like a shared workspace that they all read
from and write to.

In this tiny example chunk, the stack still only gets two values tall,
but when we start compiling Lox source to bytecode, we’ll have chunks
that use much more of the stack. In the meantime, try playing around
with this hand-authored chunk to calculate different nested arithmetic
expressions and see how values flow through the instructions and stack.

You may as well get it out of your system now. This is the last chunk
we’ll build by hand. When we next revisit bytecode, we will be writing a
compiler to generate it for us.

<div class="challenges">

## <a href="#challenges" id="challenges">Challenges</a>

1.  What bytecode instruction sequences would you generate for the
    following expressions:

    <div class="codehilite">

        1 * 2 + 3
        1 + 2 * 3
        3 - 2 - 1
        1 + 2 * 3 - 4 / -5

    </div>

    (Remember that Lox does not have a syntax for negative number
    literals, so the `-5` is negating the number 5.)

2.  If we really wanted a minimal instruction set, we could eliminate
    either `OP_NEGATE` or `OP_SUBTRACT`. Show the bytecode instruction
    sequence you would generate for:

    <div class="codehilite">

        4 - 3 * -2

    </div>

    First, without using `OP_NEGATE`. Then, without using `OP_SUBTRACT`.

    Given the above, do you think it makes sense to have both
    instructions? Why or why not? Are there any other redundant
    instructions you would consider including?

3.  Our VM’s stack has a fixed size, and we don’t check if pushing a
    value overflows it. This means the wrong series of instructions
    could cause our interpreter to crash or go into undefined behavior.
    Avoid that by dynamically growing the stack as needed.

    What are the costs and benefits of doing so?

4.  To interpret `OP_NEGATE`, we pop the operand, negate the value, and
    then push the result. That’s a simple implementation, but it
    increments and decrements `stackTop` unnecessarily, since the stack
    ends up the same height in the end. It might be faster to simply
    negate the value in place on the stack and leave `stackTop` alone.
    Try that and see if you can measure a performance difference.

    Are there other instructions where you can do a similar
    optimization?

</div>

<div class="design-note">

## <a href="#design-note" id="design-note">Design Note: Register-Based
Bytecode</a>

For the remainder of this book, we’ll meticulously implement an
interpreter around a stack-based bytecode instruction set. There’s
another family of bytecode architectures out
there<span class="em">—</span>*register-based*. Despite the name, these
bytecode instructions aren’t quite as difficult to work with as the
registers in an actual chip like <span id="x64">x64</span>. With real
hardware registers, you usually have only a handful for the entire
program, so you spend a lot of effort [trying to use them efficiently
and shuttling stuff in and out of
them](https://en.wikipedia.org/wiki/Register_allocation).

Register-based bytecode is a little closer to the [*register
windows*](https://en.wikipedia.org/wiki/Register_window) supported by
SPARC chips.

In a register-based VM, you still have a stack. Temporary values still
get pushed onto it and popped when no longer needed. The main difference
is that instructions can read their inputs from anywhere in the stack
and can store their outputs into specific stack slots.

Take this little Lox script:

<div class="codehilite">

    var a = 1;
    var b = 2;
    var c = a + b;

</div>

In our stack-based VM, the last statement will get compiled to something
like:

<div class="codehilite">

    load <a>  // Read local variable a and push onto stack.
    load <b>  // Read local variable b and push onto stack.
    add       // Pop two values, add, push result.
    store <c> // Pop value and store in local variable c.

</div>

(Don’t worry if you don’t fully understand the load and store
instructions yet. We’ll go over them in much greater detail [when we
implement variables](global-variables.html).) We have four separate
instructions. That means four times through the bytecode interpret loop,
four instructions to decode and dispatch. It’s at least seven bytes of
code<span class="em">—</span>four for the opcodes and another three for
the operands identifying which locals to load and store. Three pushes
and three pops. A lot of work!

In a register-based instruction set, instructions can read from and
store directly into local variables. The bytecode for the last statement
above looks like:

<div class="codehilite">

    add <a> <b> <c> // Read values from a and b, add, store in c.

</div>

The add instruction is bigger<span class="em">—</span>it has three
instruction operands that define where in the stack it reads its inputs
from and writes the result to. But since local variables live on the
stack, it can read directly from `a` and `b` and then store the result
right into `c`.

There’s only a single instruction to decode and dispatch, and the whole
thing fits in four bytes. Decoding is more complex because of the
additional operands, but it’s still a net win. There’s no pushing and
popping or other stack manipulation.

The main implementation of Lua used to be stack-based. For
<span id="lua">Lua 5.0</span>, the implementers switched to a register
instruction set and noted a speed improvement. The amount of
improvement, naturally, depends heavily on the details of the language
semantics, specific instruction set, and compiler sophistication, but
that should get your attention.

The Lua dev team<span class="em">—</span>Roberto Ierusalimschy, Waldemar
Celes, and Luiz Henrique de Figueiredo<span class="em">—</span>wrote a
*fantastic* paper on this, one of my all time favorite computer science
papers, “[The Implementation of Lua
5.0](https://www.lua.org/doc/jucs05.pdf)” (PDF).

That raises the obvious question of why I’m going to spend the rest of
the book doing a stack-based bytecode. Register VMs are neat, but they
are quite a bit harder to write a compiler for. For what is likely to be
your very first compiler, I wanted to stick with an instruction set
that’s easy to generate and easy to execute. Stack-based bytecode is
marvelously simple.

It’s also *much* better known in the literature and the community. Even
though you may eventually move to something more advanced, it’s a good
common ground to share with the rest of your language hacker peers.

</div>

<a href="scanning-on-demand.html" class="next">Next Chapter: “Scanning
on Demand” →</a> Handcrafted by Robert Nystrom — <a
href="https://github.com/munificent/craftinginterpreters/blob/master/LICENSE"
target="_blank">© 2015 – 2021</a>

</div>

[![](image/logotype.png "Crafting Interpreters")](/)

<div class="contents">

### [Scanning on Demand<span class="small">16</span>](#top)

- [<span class="small">16.1</span> Spinning Up the
  Interpreter](#spinning-up-the-interpreter)
- [<span class="small">16.2</span> A Token at a
  Time](#a-token-at-a-time)
- [<span class="small">16.3</span> A Lexical Grammar for
  Lox](#a-lexical-grammar-for-lox)
- [<span class="small">16.4</span> Identifiers and
  Keywords](#identifiers-and-keywords)
- 
- [Challenges](#challenges)

<div class="prev-next">

<a href="a-virtual-machine.html" class="left"
title="A Virtual Machine">← Previous</a>
[↑ Up](a-bytecode-virtual-machine.html "A Bytecode Virtual Machine")
<a href="compiling-expressions.html" class="right"
title="Compiling Expressions">Next →</a>

</div>

</div>

[![](image/logotype.png "Crafting Interpreters")](/)
<a href="a-virtual-machine.html" class="prev"
title="A Virtual Machine">←</a>
<a href="compiling-expressions.html" class="next"
title="Compiling Expressions">→</a>

<div class="page">

<div class="nav-wrapper">

[![](image/logotype.png "Crafting Interpreters")](/)

<div class="expandable">

### [Scanning on Demand<span class="small">16</span>](#top)

- [<span class="small">16.1</span> Spinning Up the
  Interpreter](#spinning-up-the-interpreter)
- [<span class="small">16.2</span> A Token at a
  Time](#a-token-at-a-time)
- [<span class="small">16.3</span> A Lexical Grammar for
  Lox](#a-lexical-grammar-for-lox)
- [<span class="small">16.4</span> Identifiers and
  Keywords](#identifiers-and-keywords)
- 
- [Challenges](#challenges)

<div class="prev-next">

<a href="a-virtual-machine.html" class="left"
title="A Virtual Machine">← Previous</a>
[↑ Up](a-bytecode-virtual-machine.html "A Bytecode Virtual Machine")
<a href="compiling-expressions.html" class="right"
title="Compiling Expressions">Next →</a>

</div>

</div>

<span id="expand-nav">≡</span>

</div>

<div class="number">

16

</div>

# Scanning on Demand

> Literature is idiosyncratic arrangements in horizontal lines in only
> twenty-six phonetic symbols, ten Arabic numbers, and about eight
> punctuation marks.
>
> Kurt Vonnegut, *Like Shaking Hands With God: A Conversation about
> Writing*

Our second interpreter, clox, has three
phases<span class="em">—</span>scanner, compiler, and virtual machine. A
data structure joins each pair of phases. Tokens flow from scanner to
compiler, and chunks of bytecode from compiler to VM. We began our
implementation near the end with [chunks](chunks-of-bytecode.html) and
the [VM](a-virtual-machine.html). Now, we’re going to hop back to the
beginning and build a scanner that makes tokens. In the [next
chapter](compiling-expressions.html), we’ll tie the two ends together
with our bytecode compiler.

![Source code → scanner → tokens → compiler → bytecode chunk →
VM.](image/scanning-on-demand/pipeline.png)

I’ll admit, this is not the most exciting chapter in the book. With two
implementations of the same language, there’s bound to be some
redundancy. I did sneak in a few interesting differences compared to
jlox’s scanner. Read on to see what they are.

## <a href="#spinning-up-the-interpreter"
id="spinning-up-the-interpreter"><span
class="small">16 . 1</span>Spinning Up the Interpreter</a>

Now that we’re building the front end, we can get clox running like a
real interpreter. No more hand-authored chunks of bytecode. It’s time
for a REPL and script loading. Tear out most of the code in `main()` and
replace it with:

<div class="codehilite">

``` insert-before
int main(int argc, const char* argv[]) {
  initVM();
```

<div class="source-file">

*main.c*  
in *main*()  
replace 26 lines

</div>

``` insert
  if (argc == 1) {
    repl();
  } else if (argc == 2) {
    runFile(argv[1]);
  } else {
    fprintf(stderr, "Usage: clox [path]\n");
    exit(64);
  }

  freeVM();
```

``` insert-after
  return 0;
}
```

</div>

<div class="source-file-narrow">

*main.c*, in *main*(), replace 26 lines

</div>

If you pass <span id="args">no arguments</span> to the executable, you
are dropped into the REPL. A single command line argument is understood
to be the path to a script to run.

The code tests for one and two arguments, not zero and one, because the
first argument in `argv` is always the name of the executable being run.

We’ll need a few system headers, so let’s get them all out of the way.

<div class="codehilite">

<div class="source-file">

*main.c*  
add to top of file

</div>

``` insert
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
```

``` insert-after
#include "common.h"
```

</div>

<div class="source-file-narrow">

*main.c*, add to top of file

</div>

Next, we get the REPL up and REPL-ing.

<div class="codehilite">

``` insert-before
#include "vm.h"
```

<div class="source-file">

*main.c*

</div>

``` insert

static void repl() {
  char line[1024];
  for (;;) {
    printf("> ");

    if (!fgets(line, sizeof(line), stdin)) {
      printf("\n");
      break;
    }

    interpret(line);
  }
}
```

</div>

<div class="source-file-narrow">

*main.c*

</div>

A quality REPL handles input that spans multiple lines gracefully and
doesn’t have a hardcoded line length limit. This REPL here is a little
more, ahem, austere, but it’s fine for our purposes.

The real work happens in `interpret()`. We’ll get to that soon, but
first let’s take care of loading scripts.

<div class="codehilite">

<div class="source-file">

*main.c*  
add after *repl*()

</div>

    static void runFile(const char* path) {
      char* source = readFile(path);
      InterpretResult result = interpret(source);
      free(source); 

      if (result == INTERPRET_COMPILE_ERROR) exit(65);
      if (result == INTERPRET_RUNTIME_ERROR) exit(70);
    }

</div>

<div class="source-file-narrow">

*main.c*, add after *repl*()

</div>

We read the file and execute the resulting string of Lox source code.
Then, based on the result of that, we set the exit code appropriately
because we’re scrupulous tool builders and care about little details
like that.

We also need to free the source code string because `readFile()`
dynamically allocates it and passes ownership to its caller. That
function looks like this:

C asks us not just to manage memory explicitly, but *mentally*. We
programmers have to remember the ownership rules and hand-implement them
throughout the program. Java just does it for us. C++ gives us tools to
encode the policy directly so that the compiler validates it for us.

I like C’s simplicity, but we pay a real price for
it<span class="em">—</span>the language requires us to be more
conscientious.

<div class="codehilite">

<div class="source-file">

*main.c*  
add after *repl*()

</div>

    static char* readFile(const char* path) {
      FILE* file = fopen(path, "rb");

      fseek(file, 0L, SEEK_END);
      size_t fileSize = ftell(file);
      rewind(file);

      char* buffer = (char*)malloc(fileSize + 1);
      size_t bytesRead = fread(buffer, sizeof(char), fileSize, file);
      buffer[bytesRead] = '\0';

      fclose(file);
      return buffer;
    }

</div>

<div class="source-file-narrow">

*main.c*, add after *repl*()

</div>

Like a lot of C code, it takes more effort than it seems like it should,
especially for a language expressly designed for operating systems. The
difficult part is that we want to allocate a big enough string to read
the whole file, but we don’t know how big the file is until we’ve read
it.

The code here is the classic trick to solve that. We open the file, but
before reading it, we seek to the very end using `fseek()`. Then we call
`ftell()` which tells us how many bytes we are from the start of the
file. Since we seeked (sought?) to the end, that’s the size. We rewind
back to the beginning, allocate a string of that
<span id="one">size</span>, and read the whole file in a single batch.

Well, that size *plus one*. Always gotta remember to make room for the
null byte.

So we’re done, right? Not quite. These function calls, like most calls
in the C standard library, can fail. If this were Java, the failures
would be thrown as exceptions and automatically unwind the stack so we
wouldn’t *really* need to handle them. In C, if we don’t check for them,
they silently get ignored.

This isn’t really a book on good C programming practice, but I hate to
encourage bad style, so let’s go ahead and handle the errors. It’s good
for us, like eating our vegetables or flossing.

Fortunately, we don’t need to do anything particularly clever if a
failure occurs. If we can’t correctly read the user’s script, all we can
really do is tell the user and exit the interpreter gracefully. First of
all, we might fail to open the file.

<div class="codehilite">

``` insert-before
  FILE* file = fopen(path, "rb");
```

<div class="source-file">

*main.c*  
in *readFile*()

</div>

``` insert
  if (file == NULL) {
    fprintf(stderr, "Could not open file \"%s\".\n", path);
    exit(74);
  }
```

``` insert-after

  fseek(file, 0L, SEEK_END);
```

</div>

<div class="source-file-narrow">

*main.c*, in *readFile*()

</div>

This can happen if the file doesn’t exist or the user doesn’t have
access to it. It’s pretty common<span class="em">—</span>people mistype
paths all the time.

This failure is much rarer:

<div class="codehilite">

``` insert-before
  char* buffer = (char*)malloc(fileSize + 1);
```

<div class="source-file">

*main.c*  
in *readFile*()

</div>

``` insert
  if (buffer == NULL) {
    fprintf(stderr, "Not enough memory to read \"%s\".\n", path);
    exit(74);
  }
```

``` insert-after
  size_t bytesRead = fread(buffer, sizeof(char), fileSize, file);
```

</div>

<div class="source-file-narrow">

*main.c*, in *readFile*()

</div>

If we can’t even allocate enough memory to read the Lox script, the
user’s probably got bigger problems to worry about, but we should do our
best to at least let them know.

Finally, the read itself may fail.

<div class="codehilite">

``` insert-before
  size_t bytesRead = fread(buffer, sizeof(char), fileSize, file);
```

<div class="source-file">

*main.c*  
in *readFile*()

</div>

``` insert
  if (bytesRead < fileSize) {
    fprintf(stderr, "Could not read file \"%s\".\n", path);
    exit(74);
  }
```

``` insert-after
  buffer[bytesRead] = '\0';
```

</div>

<div class="source-file-narrow">

*main.c*, in *readFile*()

</div>

This is also unlikely. Actually, the <span id="printf"> calls</span> to
`fseek()`, `ftell()`, and `rewind()` could theoretically fail too, but
let’s not go too far off in the weeds, shall we?

Even good old `printf()` can fail. Yup. How many times have you handled
*that* error?

### <a href="#opening-the-compilation-pipeline"
id="opening-the-compilation-pipeline"><span
class="small">16 . 1 . 1</span>Opening the compilation pipeline</a>

We’ve got ourselves a string of Lox source code, so now we’re ready to
set up a pipeline to scan, compile, and execute it. It’s driven by
`interpret()`. Right now, that function runs our old hardcoded test
chunk. Let’s change it to something closer to its final incarnation.

<div class="codehilite">

``` insert-before
void freeVM();
```

<div class="source-file">

*vm.h*  
function *interpret*()  
replace 1 line

</div>

``` insert
InterpretResult interpret(const char* source);
```

``` insert-after
void push(Value value);
```

</div>

<div class="source-file-narrow">

*vm.h*, function *interpret*(), replace 1 line

</div>

Where before we passed in a Chunk, now we pass in the string of source
code. Here’s the new implementation:

<div class="codehilite">

<div class="source-file">

*vm.c*  
function *interpret*()  
replace 4 lines

</div>

``` insert
InterpretResult interpret(const char* source) {
  compile(source);
  return INTERPRET_OK;
```

``` insert-after
}
```

</div>

<div class="source-file-narrow">

*vm.c*, function *interpret*(), replace 4 lines

</div>

We won’t build the actual *compiler* yet in this chapter, but we can
start laying out its structure. It lives in a new module.

<div class="codehilite">

``` insert-before
#include "common.h"
```

<div class="source-file">

*vm.c*

</div>

``` insert
#include "compiler.h"
```

``` insert-after
#include "debug.h"
```

</div>

<div class="source-file-narrow">

*vm.c*

</div>

For now, the one function in it is declared like so:

<div class="codehilite">

<div class="source-file">

*compiler.h*  
create new file

</div>

    #ifndef clox_compiler_h
    #define clox_compiler_h

    void compile(const char* source);

    #endif

</div>

<div class="source-file-narrow">

*compiler.h*, create new file

</div>

That signature will change, but it gets us going.

The first phase of compilation is scanning<span class="em">—</span>the
thing we’re doing in this chapter<span class="em">—</span>so right now
all the compiler does is set that up.

<div class="codehilite">

<div class="source-file">

*compiler.c*  
create new file

</div>

    #include <stdio.h>

    #include "common.h"
    #include "compiler.h"
    #include "scanner.h"

    void compile(const char* source) {
      initScanner(source);
    }

</div>

<div class="source-file-narrow">

*compiler.c*, create new file

</div>

This will also grow in later chapters, naturally.

### <a href="#the-scanner-scans" id="the-scanner-scans"><span
class="small">16 . 1 . 2</span>The scanner scans</a>

There are still a few more feet of scaffolding to stand up before we can
start writing useful code. First, a new header:

<div class="codehilite">

<div class="source-file">

*scanner.h*  
create new file

</div>

    #ifndef clox_scanner_h
    #define clox_scanner_h

    void initScanner(const char* source);

    #endif

</div>

<div class="source-file-narrow">

*scanner.h*, create new file

</div>

And its corresponding implementation:

<div class="codehilite">

<div class="source-file">

*scanner.c*  
create new file

</div>

    #include <stdio.h>
    #include <string.h>

    #include "common.h"
    #include "scanner.h"

    typedef struct {
      const char* start;
      const char* current;
      int line;
    } Scanner;

    Scanner scanner;

</div>

<div class="source-file-narrow">

*scanner.c*, create new file

</div>

As our scanner chews through the user’s source code, it tracks how far
it’s gone. Like we did with the VM, we wrap that state in a struct and
then create a single top-level module variable of that type so we don’t
have to pass it around all of the various functions.

There are surprisingly few fields. The `start` pointer marks the
beginning of the current lexeme being scanned, and `current` points to
the current character being looked at.

<span id="fields"></span>

![The start and current fields pointing at 'print bacon;'. Start points
at 'b' and current points at 'o'.](image/scanning-on-demand/fields.png)

Here, we are in the middle of scanning the identifier `bacon`. The
current character is `o` and the character we most recently consumed is
`c`.

We have a `line` field to track what line the current lexeme is on for
error reporting. That’s it! We don’t even keep a pointer to the
beginning of the source code string. The scanner works its way through
the code once and is done after that.

Since we have some state, we should initialize it.

<div class="codehilite">

<div class="source-file">

*scanner.c*  
add after variable *scanner*

</div>

    void initScanner(const char* source) {
      scanner.start = source;
      scanner.current = source;
      scanner.line = 1;
    }

</div>

<div class="source-file-narrow">

*scanner.c*, add after variable *scanner*

</div>

We start at the very first character on the very first line, like a
runner crouched at the starting line.

## <a href="#a-token-at-a-time" id="a-token-at-a-time"><span
class="small">16 . 2</span>A Token at a Time</a>

In jlox, when the starting gun went off, the scanner raced ahead and
eagerly scanned the whole program, returning a list of tokens. This
would be a challenge in clox. We’d need some sort of growable array or
list to store the tokens in. We’d need to manage allocating and freeing
the tokens, and the collection itself. That’s a lot of code, and a lot
of memory churn.

At any point in time, the compiler needs only one or two
tokens<span class="em">—</span>remember our grammar requires only a
single token of lookahead<span class="em">—</span>so we don’t need to
keep them *all* around at the same time. Instead, the simplest solution
is to not scan a token until the compiler needs one. When the scanner
provides one, it returns the token by value. It doesn’t need to
dynamically allocate anything<span class="em">—</span>it can just pass
tokens around on the C stack.

Unfortunately, we don’t have a compiler yet that can ask the scanner for
tokens, so the scanner will just sit there doing nothing. To kick it
into action, we’ll write some temporary code to drive it.

<div class="codehilite">

``` insert-before
  initScanner(source);
```

<div class="source-file">

*compiler.c*  
in *compile*()

</div>

``` insert
  int line = -1;
  for (;;) {
    Token token = scanToken();
    if (token.line != line) {
      printf("%4d ", token.line);
      line = token.line;
    } else {
      printf("   | ");
    }
    printf("%2d '%.*s'\n", token.type, token.length, token.start); 

    if (token.type == TOKEN_EOF) break;
  }
```

``` insert-after
}
```

</div>

<div class="source-file-narrow">

*compiler.c*, in *compile*()

</div>

That `%.*s` in the format string is a neat feature. Usually, you set the
output precision<span class="em">—</span>the number of characters to
show<span class="em">—</span>by placing a number inside the format
string. Using `*` instead lets you pass the precision as an argument. So
that `printf()` call prints the first `token.length` characters of the
string at `token.start`. We need to limit the length like that because
the lexeme points into the original source string and doesn’t have a
terminator at the end.

This loops indefinitely. Each turn through the loop, it scans one token
and prints it. When it reaches a special “end of file” token or an
error, it stops. For example, if we run the interpreter on this program:

<div class="codehilite">

    print 1 + 2;

</div>

It prints out:

<div class="codehilite">

       1 31 'print'
       | 21 '1'
       |  7 '+'
       | 21 '2'
       |  8 ';'
       2 39 ''

</div>

The first column is the line number, the second is the numeric value of
the token <span id="token">type</span>, and then finally the lexeme.
That last empty lexeme on line 2 is the EOF token.

Yeah, the raw index of the token type isn’t exactly human readable, but
it’s all C gives us.

The goal for the rest of the chapter is to make that blob of code work
by implementing this key function:

<div class="codehilite">

``` insert-before
void initScanner(const char* source);
```

<div class="source-file">

*scanner.h*  
add after *initScanner*()

</div>

``` insert
Token scanToken();
```

``` insert-after

#endif
```

</div>

<div class="source-file-narrow">

*scanner.h*, add after *initScanner*()

</div>

Each call scans and returns the next token in the source code. A token
looks like this:

<div class="codehilite">

``` insert-before
#define clox_scanner_h
```

<div class="source-file">

*scanner.h*

</div>

``` insert

typedef struct {
  TokenType type;
  const char* start;
  int length;
  int line;
} Token;
```

``` insert-after

void initScanner(const char* source);
```

</div>

<div class="source-file-narrow">

*scanner.h*

</div>

It’s pretty similar to jlox’s Token class. We have an enum identifying
what type of token it is<span class="em">—</span>number, identifier, `+`
operator, etc. The enum is virtually identical to the one in jlox, so
let’s just hammer out the whole thing.

<div class="codehilite">

``` insert-before
#ifndef clox_scanner_h
#define clox_scanner_h
```

<div class="source-file">

*scanner.h*

</div>

``` insert

typedef enum {
  // Single-character tokens.
  TOKEN_LEFT_PAREN, TOKEN_RIGHT_PAREN,
  TOKEN_LEFT_BRACE, TOKEN_RIGHT_BRACE,
  TOKEN_COMMA, TOKEN_DOT, TOKEN_MINUS, TOKEN_PLUS,
  TOKEN_SEMICOLON, TOKEN_SLASH, TOKEN_STAR,
  // One or two character tokens.
  TOKEN_BANG, TOKEN_BANG_EQUAL,
  TOKEN_EQUAL, TOKEN_EQUAL_EQUAL,
  TOKEN_GREATER, TOKEN_GREATER_EQUAL,
  TOKEN_LESS, TOKEN_LESS_EQUAL,
  // Literals.
  TOKEN_IDENTIFIER, TOKEN_STRING, TOKEN_NUMBER,
  // Keywords.
  TOKEN_AND, TOKEN_CLASS, TOKEN_ELSE, TOKEN_FALSE,
  TOKEN_FOR, TOKEN_FUN, TOKEN_IF, TOKEN_NIL, TOKEN_OR,
  TOKEN_PRINT, TOKEN_RETURN, TOKEN_SUPER, TOKEN_THIS,
  TOKEN_TRUE, TOKEN_VAR, TOKEN_WHILE,

  TOKEN_ERROR, TOKEN_EOF
} TokenType;
```

``` insert-after

typedef struct {
```

</div>

<div class="source-file-narrow">

*scanner.h*

</div>

Aside from prefixing all the names with `TOKEN_` (since C tosses enum
names in the top-level namespace) the only difference is that extra
`TOKEN_ERROR` type. What’s that about?

There are only a couple of errors that get detected during scanning:
unterminated strings and unrecognized characters. In jlox, the scanner
reports those itself. In clox, the scanner produces a synthetic “error”
token for that error and passes it over to the compiler. This way, the
compiler knows an error occurred and can kick off error recovery before
reporting it.

The novel part in clox’s Token type is how it represents the lexeme. In
jlox, each Token stored the lexeme as its own separate little Java
string. If we did that for clox, we’d have to figure out how to manage
the memory for those strings. That’s especially hard since we pass
tokens by value<span class="em">—</span>multiple tokens could point to
the same lexeme string. Ownership gets weird.

Instead, we use the original source string as our character store. We
represent a lexeme by a pointer to its first character and the number of
characters it contains. This means we don’t need to worry about managing
memory for lexemes at all and we can freely copy tokens around. As long
as the main source code string <span id="outlive">outlives</span> all of
the tokens, everything works fine.

I don’t mean to sound flippant. We really do need to think about and
ensure that the source string, which is created far away over in the
“main” module, has a long enough lifetime. That’s why `runFile()`
doesn’t free the string until `interpret()` finishes executing the code
and returns.

### <a href="#scanning-tokens" id="scanning-tokens"><span
class="small">16 . 2 . 1</span>Scanning tokens</a>

We’re ready to scan some tokens. We’ll work our way up to the complete
implementation, starting with this:

<div class="codehilite">

<div class="source-file">

*scanner.c*  
add after *initScanner*()

</div>

    Token scanToken() {
      scanner.start = scanner.current;

      if (isAtEnd()) return makeToken(TOKEN_EOF);

      return errorToken("Unexpected character.");
    }

</div>

<div class="source-file-narrow">

*scanner.c*, add after *initScanner*()

</div>

Since each call to this function scans a complete token, we know we are
at the beginning of a new token when we enter the function. Thus, we set
`scanner.start` to point to the current character so we remember where
the lexeme we’re about to scan starts.

Then we check to see if we’ve reached the end of the source code. If so,
we return an EOF token and stop. This is a sentinel value that signals
to the compiler to stop asking for more tokens.

If we aren’t at the end, we do
some<span class="ellipse"> . . . </span>stuff<span class="ellipse"> . . . </span>to
scan the next token. But we haven’t written that code yet. We’ll get to
that soon. If that code doesn’t successfully scan and return a token,
then we reach the end of the function. That must mean we’re at a
character that the scanner can’t recognize, so we return an error token
for that.

This function relies on a couple of helpers, most of which are familiar
from jlox. First up:

<div class="codehilite">

<div class="source-file">

*scanner.c*  
add after *initScanner*()

</div>

    static bool isAtEnd() {
      return *scanner.current == '\0';
    }

</div>

<div class="source-file-narrow">

*scanner.c*, add after *initScanner*()

</div>

We require the source string to be a good null-terminated C string. If
the current character is the null byte, then we’ve reached the end.

To create a token, we have this constructor-like function:

<div class="codehilite">

<div class="source-file">

*scanner.c*  
add after *isAtEnd*()

</div>

    static Token makeToken(TokenType type) {
      Token token;
      token.type = type;
      token.start = scanner.start;
      token.length = (int)(scanner.current - scanner.start);
      token.line = scanner.line;
      return token;
    }

</div>

<div class="source-file-narrow">

*scanner.c*, add after *isAtEnd*()

</div>

It uses the scanner’s `start` and `current` pointers to capture the
token’s lexeme. It sets a couple of other obvious fields then returns
the token. It has a sister function for returning error tokens.

<div class="codehilite">

<div class="source-file">

*scanner.c*  
add after *makeToken*()

</div>

    static Token errorToken(const char* message) {
      Token token;
      token.type = TOKEN_ERROR;
      token.start = message;
      token.length = (int)strlen(message);
      token.line = scanner.line;
      return token;
    }

</div>

<div class="source-file-narrow">

*scanner.c*, add after *makeToken*()

</div>

<span id="axolotl"></span>

This part of the chapter is pretty dry, so here’s a picture of an
axolotl.

![A drawing of an axolotl.](image/scanning-on-demand/axolotl.png)

The only difference is that the “lexeme” points to the error message
string instead of pointing into the user’s source code. Again, we need
to ensure that the error message sticks around long enough for the
compiler to read it. In practice, we only ever call this function with C
string literals. Those are constant and eternal, so we’re fine.

What we have now is basically a working scanner for a language with an
empty lexical grammar. Since the grammar has no productions, every
character is an error. That’s not exactly a fun language to program in,
so let’s fill in the rules.

## <a href="#a-lexical-grammar-for-lox"
id="a-lexical-grammar-for-lox"><span class="small">16 . 3</span>A
Lexical Grammar for Lox</a>

The simplest tokens are only a single character. We recognize those like
so:

<div class="codehilite">

``` insert-before
  if (isAtEnd()) return makeToken(TOKEN_EOF);
```

<div class="source-file">

*scanner.c*  
in *scanToken*()

</div>

``` insert

  char c = advance();

  switch (c) {
    case '(': return makeToken(TOKEN_LEFT_PAREN);
    case ')': return makeToken(TOKEN_RIGHT_PAREN);
    case '{': return makeToken(TOKEN_LEFT_BRACE);
    case '}': return makeToken(TOKEN_RIGHT_BRACE);
    case ';': return makeToken(TOKEN_SEMICOLON);
    case ',': return makeToken(TOKEN_COMMA);
    case '.': return makeToken(TOKEN_DOT);
    case '-': return makeToken(TOKEN_MINUS);
    case '+': return makeToken(TOKEN_PLUS);
    case '/': return makeToken(TOKEN_SLASH);
    case '*': return makeToken(TOKEN_STAR);
  }
```

``` insert-after

  return errorToken("Unexpected character.");
```

</div>

<div class="source-file-narrow">

*scanner.c*, in *scanToken*()

</div>

We read the next character from the source code, and then do a
straightforward switch to see if it matches any of Lox’s one-character
lexemes. To read the next character, we use a new helper which consumes
the current character and returns it.

<div class="codehilite">

<div class="source-file">

*scanner.c*  
add after *isAtEnd*()

</div>

    static char advance() {
      scanner.current++;
      return scanner.current[-1];
    }

</div>

<div class="source-file-narrow">

*scanner.c*, add after *isAtEnd*()

</div>

Next up are the two-character punctuation tokens like `!=` and `>=`.
Each of these also has a corresponding single-character token. That
means that when we see a character like `!`, we don’t know if we’re in a
`!` token or a `!=` until we look at the next character too. We handle
those like so:

<div class="codehilite">

``` insert-before
    case '*': return makeToken(TOKEN_STAR);
```

<div class="source-file">

*scanner.c*  
in *scanToken*()

</div>

``` insert
    case '!':
      return makeToken(
          match('=') ? TOKEN_BANG_EQUAL : TOKEN_BANG);
    case '=':
      return makeToken(
          match('=') ? TOKEN_EQUAL_EQUAL : TOKEN_EQUAL);
    case '<':
      return makeToken(
          match('=') ? TOKEN_LESS_EQUAL : TOKEN_LESS);
    case '>':
      return makeToken(
          match('=') ? TOKEN_GREATER_EQUAL : TOKEN_GREATER);
```

``` insert-after
  }
```

</div>

<div class="source-file-narrow">

*scanner.c*, in *scanToken*()

</div>

After consuming the first character, we look for an `=`. If found, we
consume it and return the corresponding two-character token. Otherwise,
we leave the current character alone (so it can be part of the *next*
token) and return the appropriate one-character token.

That logic for conditionally consuming the second character lives here:

<div class="codehilite">

<div class="source-file">

*scanner.c*  
add after *advance*()

</div>

    static bool match(char expected) {
      if (isAtEnd()) return false;
      if (*scanner.current != expected) return false;
      scanner.current++;
      return true;
    }

</div>

<div class="source-file-narrow">

*scanner.c*, add after *advance*()

</div>

If the current character is the desired one, we advance and return
`true`. Otherwise, we return `false` to indicate it wasn’t matched.

Now our scanner supports all of the punctuation-like tokens. Before we
get to the longer ones, let’s take a little side trip to handle
characters that aren’t part of a token at all.

### <a href="#whitespace" id="whitespace"><span
class="small">16 . 3 . 1</span>Whitespace</a>

Our scanner needs to handle spaces, tabs, and newlines, but those
characters don’t become part of any token’s lexeme. We could check for
those inside the main character switch in `scanToken()` but it gets a
little tricky to ensure that the function still correctly finds the next
token *after* the whitespace when you call it. We’d have to wrap the
whole body of the function in a loop or something.

Instead, before starting the token, we shunt off to a separate function.

<div class="codehilite">

``` insert-before
Token scanToken() {
```

<div class="source-file">

*scanner.c*  
in *scanToken*()

</div>

``` insert
  skipWhitespace();
```

``` insert-after
  scanner.start = scanner.current;
```

</div>

<div class="source-file-narrow">

*scanner.c*, in *scanToken*()

</div>

This advances the scanner past any leading whitespace. After this call
returns, we know the very next character is a meaningful one (or we’re
at the end of the source code).

<div class="codehilite">

<div class="source-file">

*scanner.c*  
add after *errorToken*()

</div>

    static void skipWhitespace() {
      for (;;) {
        char c = peek();
        switch (c) {
          case ' ':
          case '\r':
          case '\t':
            advance();
            break;
          default:
            return;
        }
      }
    }

</div>

<div class="source-file-narrow">

*scanner.c*, add after *errorToken*()

</div>

It’s sort of a separate mini-scanner. It loops, consuming every
whitespace character it encounters. We need to be careful that it does
*not* consume any *non*-whitespace characters. To support that, we use
this:

<div class="codehilite">

<div class="source-file">

*scanner.c*  
add after *advance*()

</div>

    static char peek() {
      return *scanner.current;
    }

</div>

<div class="source-file-narrow">

*scanner.c*, add after *advance*()

</div>

This simply returns the current character, but doesn’t consume it. The
previous code handles all the whitespace characters except for newlines.

<div class="codehilite">

``` insert-before
        break;
```

<div class="source-file">

*scanner.c*  
in *skipWhitespace*()

</div>

``` insert
      case '\n':
        scanner.line++;
        advance();
        break;
```

``` insert-after
      default:
        return;
```

</div>

<div class="source-file-narrow">

*scanner.c*, in *skipWhitespace*()

</div>

When we consume one of those, we also bump the current line number.

### <a href="#comments" id="comments"><span
class="small">16 . 3 . 2</span>Comments</a>

Comments aren’t technically “whitespace”, if you want to get all precise
with your terminology, but as far as Lox is concerned, they may as well
be, so we skip those too.

<div class="codehilite">

``` insert-before
        break;
```

<div class="source-file">

*scanner.c*  
in *skipWhitespace*()

</div>

``` insert
      case '/':
        if (peekNext() == '/') {
          // A comment goes until the end of the line.
          while (peek() != '\n' && !isAtEnd()) advance();
        } else {
          return;
        }
        break;
```

``` insert-after
      default:
        return;
```

</div>

<div class="source-file-narrow">

*scanner.c*, in *skipWhitespace*()

</div>

Comments start with `//` in Lox, so as with `!=` and friends, we need a
second character of lookahead. However, with `!=`, we still wanted to
consume the `!` even if the `=` wasn’t found. Comments are different. If
we don’t find a second `/`, then `skipWhitespace()` needs to not consume
the *first* slash either.

To handle that, we add:

<div class="codehilite">

<div class="source-file">

*scanner.c*  
add after *peek*()

</div>

    static char peekNext() {
      if (isAtEnd()) return '\0';
      return scanner.current[1];
    }

</div>

<div class="source-file-narrow">

*scanner.c*, add after *peek*()

</div>

This is like `peek()` but for one character past the current one. If the
current character and the next one are both `/`, we consume them and
then any other characters until the next newline or the end of the
source code.

We use `peek()` to check for the newline but not consume it. That way,
the newline will be the current character on the next turn of the outer
loop in `skipWhitespace()` and we’ll recognize it and increment
`scanner.line`.

### <a href="#literal-tokens" id="literal-tokens"><span
class="small">16 . 3 . 3</span>Literal tokens</a>

Number and string tokens are special because they have a runtime value
associated with them. We’ll start with strings because they are easy to
recognize<span class="em">—</span>they always begin with a double quote.

<div class="codehilite">

``` insert-before
          match('=') ? TOKEN_GREATER_EQUAL : TOKEN_GREATER);
```

<div class="source-file">

*scanner.c*  
in *scanToken*()

</div>

``` insert
    case '"': return string();
```

``` insert-after
  }
```

</div>

<div class="source-file-narrow">

*scanner.c*, in *scanToken*()

</div>

That calls a new function.

<div class="codehilite">

<div class="source-file">

*scanner.c*  
add after *skipWhitespace*()

</div>

    static Token string() {
      while (peek() != '"' && !isAtEnd()) {
        if (peek() == '\n') scanner.line++;
        advance();
      }

      if (isAtEnd()) return errorToken("Unterminated string.");

      // The closing quote.
      advance();
      return makeToken(TOKEN_STRING);
    }

</div>

<div class="source-file-narrow">

*scanner.c*, add after *skipWhitespace*()

</div>

Similar to jlox, we consume characters until we reach the closing quote.
We also track newlines inside the string literal. (Lox supports
multi-line strings.) And, as ever, we gracefully handle running out of
source code before we find the end quote.

The main change here in clox is something that’s *not* present. Again,
it relates to memory management. In jlox, the Token class had a field of
type Object to store the runtime value converted from the literal
token’s lexeme.

Implementing that in C would require a lot of work. We’d need some sort
of union and type tag to tell whether the token contains a string or
double value. If it’s a string, we’d need to manage the memory for the
string’s character array somehow.

Instead of adding that complexity to the scanner, we defer
<span id="convert">converting</span> the literal lexeme to a runtime
value until later. In clox, tokens only store the
lexeme<span class="em">—</span>the character sequence exactly as it
appears in the user’s source code. Later in the compiler, we’ll convert
that lexeme to a runtime value right when we are ready to store it in
the chunk’s constant table.

Doing the lexeme-to-value conversion in the compiler does introduce some
redundancy. The work to scan a number literal is awfully similar to the
work required to convert a sequence of digit characters to a number
value. But there isn’t *that* much redundancy, it isn’t in anything
performance critical, and it keeps our scanner simpler.

Next up, numbers. Instead of adding a switch case for each of the ten
digits that can start a number, we handle them here:

<div class="codehilite">

``` insert-before
  char c = advance();
```

<div class="source-file">

*scanner.c*  
in *scanToken*()

</div>

``` insert
  if (isDigit(c)) return number();
```

``` insert-after

  switch (c) {
```

</div>

<div class="source-file-narrow">

*scanner.c*, in *scanToken*()

</div>

That uses this obvious utility function:

<div class="codehilite">

<div class="source-file">

*scanner.c*  
add after *initScanner*()

</div>

    static bool isDigit(char c) {
      return c >= '0' && c <= '9';
    }

</div>

<div class="source-file-narrow">

*scanner.c*, add after *initScanner*()

</div>

We finish scanning the number using this:

<div class="codehilite">

<div class="source-file">

*scanner.c*  
add after *skipWhitespace*()

</div>

    static Token number() {
      while (isDigit(peek())) advance();

      // Look for a fractional part.
      if (peek() == '.' && isDigit(peekNext())) {
        // Consume the ".".
        advance();

        while (isDigit(peek())) advance();
      }

      return makeToken(TOKEN_NUMBER);
    }

</div>

<div class="source-file-narrow">

*scanner.c*, add after *skipWhitespace*()

</div>

It’s virtually identical to jlox’s version except, again, we don’t
convert the lexeme to a double yet.

## <a href="#identifiers-and-keywords" id="identifiers-and-keywords"><span
class="small">16 . 4</span>Identifiers and Keywords</a>

The last batch of tokens are identifiers, both user-defined and
reserved. This section should be fun<span class="em">—</span>the way we
recognize keywords in clox is quite different from how we did it in
jlox, and touches on some important data structures.

First, though, we have to scan the lexeme. Names start with a letter or
underscore.

<div class="codehilite">

``` insert-before
  char c = advance();
```

<div class="source-file">

*scanner.c*  
in *scanToken*()

</div>

``` insert
  if (isAlpha(c)) return identifier();
```

``` insert-after
  if (isDigit(c)) return number();
```

</div>

<div class="source-file-narrow">

*scanner.c*, in *scanToken*()

</div>

We recognize those using this:

<div class="codehilite">

<div class="source-file">

*scanner.c*  
add after *initScanner*()

</div>

    static bool isAlpha(char c) {
      return (c >= 'a' && c <= 'z') ||
             (c >= 'A' && c <= 'Z') ||
              c == '_';
    }

</div>

<div class="source-file-narrow">

*scanner.c*, add after *initScanner*()

</div>

Once we’ve found an identifier, we scan the rest of it here:

<div class="codehilite">

<div class="source-file">

*scanner.c*  
add after *skipWhitespace*()

</div>

    static Token identifier() {
      while (isAlpha(peek()) || isDigit(peek())) advance();
      return makeToken(identifierType());
    }

</div>

<div class="source-file-narrow">

*scanner.c*, add after *skipWhitespace*()

</div>

After the first letter, we allow digits too, and we keep consuming
alphanumerics until we run out of them. Then we produce a token with the
proper type. Determining that “proper” type is the unique part of this
chapter.

<div class="codehilite">

<div class="source-file">

*scanner.c*  
add after *skipWhitespace*()

</div>

    static TokenType identifierType() {
      return TOKEN_IDENTIFIER;
    }

</div>

<div class="source-file-narrow">

*scanner.c*, add after *skipWhitespace*()

</div>

Okay, I guess that’s not very exciting yet. That’s what it looks like if
we have no reserved words at all. How should we go about recognizing
keywords? In jlox, we stuffed them all in a Java Map and looked them up
by name. We don’t have any sort of hash table structure in clox, at
least not yet.

A hash table would be overkill anyway. To look up a string in a hash
<span id="hash">table</span>, we need to walk the string to calculate
its hash code, find the corresponding bucket in the hash table, and then
do a character-by-character equality comparison on any string it happens
to find there.

Don’t worry if this is unfamiliar to you. When we get to [building our
own hash table from scratch](hash-tables.html), we’ll learn all about it
in exquisite detail.

Let’s say we’ve scanned the identifier “gorgonzola”. How much work
*should* we need to do to tell if that’s a reserved word? Well, no Lox
keyword starts with “g”, so looking at the first character is enough to
definitively answer no. That’s a lot simpler than a hash table lookup.

What about “cardigan”? We do have a keyword in Lox that starts with “c”:
“class”. But the second character in “cardigan”, “a”, rules that out.
What about “forest”? Since “for” is a keyword, we have to go farther in
the string before we can establish that we don’t have a reserved word.
But, in most cases, only a character or two is enough to tell we’ve got
a user-defined name on our hands. We should be able to recognize that
and fail fast.

Here’s a visual representation of that branching character-inspection
logic:

<span id="down"></span>

![A trie that contains all of Lox's
keywords.](image/scanning-on-demand/keywords.png)

Read down each chain of nodes and you’ll see Lox’s keywords emerge.

We start at the root node. If there is a child node whose letter matches
the first character in the lexeme, we move to that node. Then repeat for
the next letter in the lexeme and so on. If at any point the next letter
in the lexeme doesn’t match a child node, then the identifier must not
be a keyword and we stop. If we reach a double-lined box, and we’re at
the last character of the lexeme, then we found a keyword.

### <a href="#tries-and-state-machines" id="tries-and-state-machines"><span
class="small">16 . 4 . 1</span>Tries and state machines</a>

This tree diagram is an example of a thing called a
<span id="trie">[**trie**](https://en.wikipedia.org/wiki/Trie)</span>. A
trie stores a set of strings. Most other data structures for storing
strings contain the raw character arrays and then wrap them inside some
larger construct that helps you search faster. A trie is different.
Nowhere in the trie will you find a whole string.

“Trie” is one of the most confusing names in CS. Edward Fredkin yanked
it out of the middle of the word “retrieval”, which means it should be
pronounced like “tree”. But, uh, there is already a pretty important
data structure pronounced “tree” *which tries are a special case of*, so
unless you never speak of these things out loud, no one can tell which
one you’re talking about. Thus, people these days often pronounce it
like “try” to avoid the headache.

Instead, each string the trie “contains” is represented as a *path*
through the tree of character nodes, as in our traversal above. Nodes
that match the last character in a string have a special
marker<span class="em">—</span>the double lined boxes in the
illustration. That way, if your trie contains, say, “banquet” and “ban”,
you are able to tell that it does *not* contain
“banque”<span class="em">—</span>the “e” node won’t have that marker,
while the “n” and “t” nodes will.

Tries are a special case of an even more fundamental data structure: a
[**deterministic finite
automaton**](https://en.wikipedia.org/wiki/Deterministic_finite_automaton)
(**DFA**). You might also know these by other names: **finite state
machine**, or just **state machine**. State machines are rad. They end
up useful in everything from [game
programming](http://gameprogrammingpatterns.com/state.html) to
implementing networking protocols.

In a DFA, you have a set of *states* with *transitions* between them,
forming a graph. At any point in time, the machine is “in” exactly one
state. It gets to other states by following transitions. When you use a
DFA for lexical analysis, each transition is a character that gets
matched from the string. Each state represents a set of allowed
characters.

Our keyword tree is exactly a DFA that recognizes Lox keywords. But DFAs
are more powerful than simple trees because they can be arbitrary
*graphs*. Transitions can form cycles between states. That lets you
recognize arbitrarily long strings. For example, here’s a DFA that
recognizes number literals:

<span id="railroad"></span>

![A syntax diagram that recognizes integer and floating point
literals.](image/scanning-on-demand/numbers.png)

This style of diagram is called a [**syntax
diagram**](https://en.wikipedia.org/wiki/Syntax_diagram) or the more
charming **railroad diagram**. The latter name is because it looks
something like a switching yard for trains.

Back before Backus-Naur Form was a thing, this was one of the
predominant ways of documenting a language’s grammar. These days, we
mostly use text, but there’s something delightful about the official
specification for a *textual language* relying on an *image*.

I’ve collapsed the nodes for the ten digits together to keep it more
readable, but the basic process works the
same<span class="em">—</span>you work through the path, entering nodes
whenever you consume a corresponding character in the lexeme. If we were
so inclined, we could construct one big giant DFA that does *all* of the
lexical analysis for Lox, a single state machine that recognizes and
spits out all of the tokens we need.

However, crafting that mega-DFA by <span id="regex">hand</span> would be
challenging. That’s why
[Lex](https://en.wikipedia.org/wiki/Lex_(software)) was created. You
give it a simple textual description of your lexical
grammar<span class="em">—</span>a bunch of regular
expressions<span class="em">—</span>and it automatically generates a DFA
for you and produces a pile of C code that implements it.

This is also how most regular expression engines in programming
languages and text editors work under the hood. They take your regex
string and convert it to a DFA, which they then use to match strings.

If you want to learn the algorithm to convert a regular expression into
a DFA, [the dragon
book](https://en.wikipedia.org/wiki/Compilers:_Principles,_Techniques,_and_Tools)
has you covered.

We won’t go down that road. We already have a perfectly serviceable
hand-rolled scanner. We just need a tiny trie for recognizing keywords.
How should we map that to code?

The absolute simplest <span id="v8">solution</span> is to use a switch
statement for each node with cases for each branch. We’ll start with the
root node and handle the easy keywords.

Simple doesn’t mean dumb. The same approach is [essentially what V8
does](https://github.com/v8/v8/blob/e77eebfe3b747fb315bd3baad09bec0953e53e68/src/parsing/scanner.cc#L1643),
and that’s currently one of the world’s most sophisticated, fastest
language implementations.

<div class="codehilite">

``` insert-before
static TokenType identifierType() {
```

<div class="source-file">

*scanner.c*  
in *identifierType*()

</div>

``` insert
  switch (scanner.start[0]) {
    case 'a': return checkKeyword(1, 2, "nd", TOKEN_AND);
    case 'c': return checkKeyword(1, 4, "lass", TOKEN_CLASS);
    case 'e': return checkKeyword(1, 3, "lse", TOKEN_ELSE);
    case 'i': return checkKeyword(1, 1, "f", TOKEN_IF);
    case 'n': return checkKeyword(1, 2, "il", TOKEN_NIL);
    case 'o': return checkKeyword(1, 1, "r", TOKEN_OR);
    case 'p': return checkKeyword(1, 4, "rint", TOKEN_PRINT);
    case 'r': return checkKeyword(1, 5, "eturn", TOKEN_RETURN);
    case 's': return checkKeyword(1, 4, "uper", TOKEN_SUPER);
    case 'v': return checkKeyword(1, 2, "ar", TOKEN_VAR);
    case 'w': return checkKeyword(1, 4, "hile", TOKEN_WHILE);
  }
```

``` insert-after
  return TOKEN_IDENTIFIER;
```

</div>

<div class="source-file-narrow">

*scanner.c*, in *identifierType*()

</div>

These are the initial letters that correspond to a single keyword. If we
see an “s”, the only keyword the identifier could possibly be is
`super`. It might not be, though, so we still need to check the rest of
the letters too. In the tree diagram, this is basically that straight
path hanging off the “s”.

We won’t roll a switch for each of those nodes. Instead, we have a
utility function that tests the rest of a potential keyword’s lexeme.

<div class="codehilite">

<div class="source-file">

*scanner.c*  
add after *skipWhitespace*()

</div>

    static TokenType checkKeyword(int start, int length,
        const char* rest, TokenType type) {
      if (scanner.current - scanner.start == start + length &&
          memcmp(scanner.start + start, rest, length) == 0) {
        return type;
      }

      return TOKEN_IDENTIFIER;
    }

</div>

<div class="source-file-narrow">

*scanner.c*, add after *skipWhitespace*()

</div>

We use this for all of the unbranching paths in the tree. Once we’ve
found a prefix that could only be one possible reserved word, we need to
verify two things. The lexeme must be exactly as long as the keyword. If
the first letter is “s”, the lexeme could still be “sup” or “superb”.
And the remaining characters must match
exactly<span class="em">—</span>“supar” isn’t good enough.

If we do have the right number of characters, and they’re the ones we
want, then it’s a keyword, and we return the associated token type.
Otherwise, it must be a normal identifier.

We have a couple of keywords where the tree branches again after the
first letter. If the lexeme starts with “f”, it could be `false`, `for`,
or `fun`. So we add another switch for the branches coming off the “f”
node.

<div class="codehilite">

``` insert-before
    case 'e': return checkKeyword(1, 3, "lse", TOKEN_ELSE);
```

<div class="source-file">

*scanner.c*  
in *identifierType*()

</div>

``` insert
    case 'f':
      if (scanner.current - scanner.start > 1) {
        switch (scanner.start[1]) {
          case 'a': return checkKeyword(2, 3, "lse", TOKEN_FALSE);
          case 'o': return checkKeyword(2, 1, "r", TOKEN_FOR);
          case 'u': return checkKeyword(2, 1, "n", TOKEN_FUN);
        }
      }
      break;
```

``` insert-after
    case 'i': return checkKeyword(1, 1, "f", TOKEN_IF);
```

</div>

<div class="source-file-narrow">

*scanner.c*, in *identifierType*()

</div>

Before we switch, we need to check that there even *is* a second letter.
“f” by itself is a valid identifier too, after all. The other letter
that branches is “t”.

<div class="codehilite">

``` insert-before
    case 's': return checkKeyword(1, 4, "uper", TOKEN_SUPER);
```

<div class="source-file">

*scanner.c*  
in *identifierType*()

</div>

``` insert
    case 't':
      if (scanner.current - scanner.start > 1) {
        switch (scanner.start[1]) {
          case 'h': return checkKeyword(2, 2, "is", TOKEN_THIS);
          case 'r': return checkKeyword(2, 2, "ue", TOKEN_TRUE);
        }
      }
      break;
```

``` insert-after
    case 'v': return checkKeyword(1, 2, "ar", TOKEN_VAR);
```

</div>

<div class="source-file-narrow">

*scanner.c*, in *identifierType*()

</div>

That’s it. A couple of nested `switch` statements. Not only is this code
<span id="short">short</span>, but it’s very, very fast. It does the
minimum amount of work required to detect a keyword, and bails out as
soon as it can tell the identifier will not be a reserved one.

And with that, our scanner is complete.

We sometimes fall into the trap of thinking that performance comes from
complicated data structures, layers of caching, and other fancy
optimizations. But, many times, all that’s required is to do less work,
and I often find that writing the simplest code I can is sufficient to
accomplish that.

<div class="challenges">

## <a href="#challenges" id="challenges">Challenges</a>

1.  Many newer languages support [**string
    interpolation**](https://en.wikipedia.org/wiki/String_interpolation).
    Inside a string literal, you have some sort of special
    delimiters<span class="em">—</span>most commonly `${` at the
    beginning and `}` at the end. Between those delimiters, any
    expression can appear. When the string literal is executed, the
    inner expression is evaluated, converted to a string, and then
    merged with the surrounding string literal.

    For example, if Lox supported string interpolation, then
    this<span class="ellipse"> . . . </span>

    <div class="codehilite">

        var drink = "Tea";
        var steep = 4;
        var cool = 2;
        print "${drink} will be ready in ${steep + cool} minutes.";

    </div>

    <span class="ellipse"> . . . </span>would print:

    <div class="codehilite">

        Tea will be ready in 6 minutes.

    </div>

    What token types would you define to implement a scanner for string
    interpolation? What sequence of tokens would you emit for the above
    string literal?

    What tokens would you emit for:

    <div class="codehilite">

        "Nested ${"interpolation?! Are you ${"mad?!"}"}"

    </div>

    Consider looking at other language implementations that support
    interpolation to see how they handle it.

2.  Several languages use angle brackets for generics and also have a
    `>>` right shift operator. This led to a classic problem in early
    versions of C++:

    <div class="codehilite">

        vector<vector<string>> nestedVectors;

    </div>

    This would produce a compile error because the `>>` was lexed to a
    single right shift token, not two `>` tokens. Users were forced to
    avoid this by putting a space between the closing angle brackets.

    Later versions of C++ are smarter and can handle the above code.
    Java and C# never had the problem. How do those languages specify
    and implement this?

3.  Many languages, especially later in their evolution, define
    “contextual keywords”. These are identifiers that act like reserved
    words in some contexts but can be normal user-defined identifiers in
    others.

    For example, `await` is a keyword inside an `async` method in C#,
    but in other methods, you can use `await` as your own identifier.

    Name a few contextual keywords from other languages, and the context
    where they are meaningful. What are the pros and cons of having
    contextual keywords? How would you implement them in your language’s
    front end if you needed to?

</div>

<a href="compiling-expressions.html" class="next">Next Chapter:
“Compiling Expressions” →</a> Handcrafted by Robert Nystrom — <a
href="https://github.com/munificent/craftinginterpreters/blob/master/LICENSE"
target="_blank">© 2015 – 2021</a>

</div>

[![](image/logotype.png "Crafting Interpreters")](/)

<div class="contents">

### [Compiling Expressions<span class="small">17</span>](#top)

- [<span class="small">17.1</span> Single-Pass
  Compilation](#single-pass-compilation)
- [<span class="small">17.2</span> Parsing Tokens](#parsing-tokens)
- [<span class="small">17.3</span> Emitting
  Bytecode](#emitting-bytecode)
- [<span class="small">17.4</span> Parsing Prefix
  Expressions](#parsing-prefix-expressions)
- [<span class="small">17.5</span> Parsing Infix
  Expressions](#parsing-infix-expressions)
- [<span class="small">17.6</span> A Pratt Parser](#a-pratt-parser)
- [<span class="small">17.7</span> Dumping Chunks](#dumping-chunks)
- 
- [Challenges](#challenges)
- [<span class="small">note</span>It's Just Parsing](#design-note)

<div class="prev-next">

<a href="scanning-on-demand.html" class="left"
title="Scanning on Demand">← Previous</a>
[↑ Up](a-bytecode-virtual-machine.html "A Bytecode Virtual Machine")
<a href="types-of-values.html" class="right"
title="Types of Values">Next →</a>

</div>

</div>

[![](image/logotype.png "Crafting Interpreters")](/)
<a href="scanning-on-demand.html" class="prev"
title="Scanning on Demand">←</a>
<a href="types-of-values.html" class="next"
title="Types of Values">→</a>

<div class="page">

<div class="nav-wrapper">

[![](image/logotype.png "Crafting Interpreters")](/)

<div class="expandable">

### [Compiling Expressions<span class="small">17</span>](#top)

- [<span class="small">17.1</span> Single-Pass
  Compilation](#single-pass-compilation)
- [<span class="small">17.2</span> Parsing Tokens](#parsing-tokens)
- [<span class="small">17.3</span> Emitting
  Bytecode](#emitting-bytecode)
- [<span class="small">17.4</span> Parsing Prefix
  Expressions](#parsing-prefix-expressions)
- [<span class="small">17.5</span> Parsing Infix
  Expressions](#parsing-infix-expressions)
- [<span class="small">17.6</span> A Pratt Parser](#a-pratt-parser)
- [<span class="small">17.7</span> Dumping Chunks](#dumping-chunks)
- 
- [Challenges](#challenges)
- [<span class="small">note</span>It's Just Parsing](#design-note)

<div class="prev-next">

<a href="scanning-on-demand.html" class="left"
title="Scanning on Demand">← Previous</a>
[↑ Up](a-bytecode-virtual-machine.html "A Bytecode Virtual Machine")
<a href="types-of-values.html" class="right"
title="Types of Values">Next →</a>

</div>

</div>

<span id="expand-nav">≡</span>

</div>

<div class="number">

17

</div>

# Compiling Expressions

> In the middle of the journey of our life I found myself within a dark
> woods where the straight way was lost.
>
> Dante Alighieri, *Inferno*

This chapter is exciting for not one, not two, but *three* reasons.
First, it provides the final segment of our VM’s execution pipeline.
Once in place, we can plumb the user’s source code from scanning all the
way through to executing it.

![Lowering the 'compiler' section of pipe between 'scanner' and
'VM'.](image/compiling-expressions/pipeline.png)

Second, we get to write an actual, honest-to-God *compiler*. It parses
source code and outputs a low-level series of binary instructions. Sure,
it’s <span id="wirth">bytecode</span> and not some chip’s native
instruction set, but it’s way closer to the metal than jlox was. We’re
about to be real language hackers.

Bytecode was good enough for Niklaus Wirth, and no one questions his
street cred.

<span id="pratt">Third</span> and finally, I get to show you one of my
absolute favorite algorithms: Vaughan Pratt’s “top-down operator
precedence parsing”. It’s the most elegant way I know to parse
expressions. It gracefully handles prefix operators, postfix, infix,
*mixfix*, any kind of *-fix* you got. It deals with precedence and
associativity without breaking a sweat. I love it.

Pratt parsers are a sort of oral tradition in industry. No compiler or
language book I’ve read teaches them. Academia is very focused on
generated parsers, and Pratt’s technique is for handwritten ones, so it
gets overlooked.

But in production compilers, where hand-rolled parsers are common, you’d
be surprised how many people know it. Ask where they learned it, and
it’s always, “Oh, I worked on this compiler years ago and my coworker
said they took it from this old front
end<span class="ellipse"> . . . </span>”

As usual, before we get to the fun stuff, we’ve got some preliminaries
to work through. You have to eat your vegetables before you get dessert.
First, let’s ditch that temporary scaffolding we wrote for testing the
scanner and replace it with something more useful.

<div class="codehilite">

``` insert-before
InterpretResult interpret(const char* source) {
```

<div class="source-file">

*vm.c*  
in *interpret*()  
replace 2 lines

</div>

``` insert
  Chunk chunk;
  initChunk(&chunk);

  if (!compile(source, &chunk)) {
    freeChunk(&chunk);
    return INTERPRET_COMPILE_ERROR;
  }

  vm.chunk = &chunk;
  vm.ip = vm.chunk->code;

  InterpretResult result = run();

  freeChunk(&chunk);
  return result;
```

``` insert-after
}
```

</div>

<div class="source-file-narrow">

*vm.c*, in *interpret*(), replace 2 lines

</div>

We create a new empty chunk and pass it over to the compiler. The
compiler will take the user’s program and fill up the chunk with
bytecode. At least, that’s what it will do if the program doesn’t have
any compile errors. If it does encounter an error, `compile()` returns
`false` and we discard the unusable chunk.

Otherwise, we send the completed chunk over to the VM to be executed.
When the VM finishes, we free the chunk and we’re done. As you can see,
the signature to `compile()` is different now.

<div class="codehilite">

``` insert-before
#define clox_compiler_h
```

<div class="source-file">

*compiler.h*  
replace 1 line

</div>

``` insert
#include "vm.h"

bool compile(const char* source, Chunk* chunk);
```

``` insert-after

#endif
```

</div>

<div class="source-file-narrow">

*compiler.h*, replace 1 line

</div>

We pass in the chunk where the compiler will write the code, and then
`compile()` returns whether or not compilation succeeded. We make the
same change to the signature in the implementation.

<div class="codehilite">

``` insert-before
#include "scanner.h"
```

<div class="source-file">

*compiler.c*  
function *compile*()  
replace 1 line

</div>

``` insert
bool compile(const char* source, Chunk* chunk) {
```

``` insert-after
  initScanner(source);
```

</div>

<div class="source-file-narrow">

*compiler.c*, function *compile*(), replace 1 line

</div>

That call to `initScanner()` is the only line that survives this
chapter. Rip out the temporary code we wrote to test the scanner and
replace it with these three lines:

<div class="codehilite">

``` insert-before
  initScanner(source);
```

<div class="source-file">

*compiler.c*  
in *compile*()  
replace 13 lines

</div>

``` insert
  advance();
  expression();
  consume(TOKEN_EOF, "Expect end of expression.");
```

``` insert-after
}
```

</div>

<div class="source-file-narrow">

*compiler.c*, in *compile*(), replace 13 lines

</div>

The call to `advance()` “primes the pump” on the scanner. We’ll see what
it does soon. Then we parse a single expression. We aren’t going to do
statements yet, so that’s the only subset of the grammar we support.
We’ll revisit this when we [add statements in a few
chapters](global-variables.html). After we compile the expression, we
should be at the end of the source code, so we check for the sentinel
EOF token.

We’re going to spend the rest of the chapter making this function work,
especially that little `expression()` call. Normally, we’d dive right
into that function definition and work our way through the
implementation from top to bottom.

This chapter is <span id="blog">different</span>. Pratt’s parsing
technique is remarkably simple once you have it all loaded in your head,
but it’s a little tricky to break into bite-sized pieces. It’s
recursive, of course, which is part of the problem. But it also relies
on a big table of data. As we build up the algorithm, that table grows
additional columns.

If this chapter isn’t clicking with you and you’d like another take on
the concepts, I wrote an article that teaches the same algorithm but
using Java and an object-oriented style: [“Pratt Parsing: Expression
Parsing Made
Easy”](http://journal.stuffwithstuff.com/2011/03/19/pratt-parsers-expression-parsing-made-easy/).

I don’t want to revisit 40-something lines of code each time we extend
the table. So we’re going to work our way into the core of the parser
from the outside and cover all of the surrounding bits before we get to
the juicy center. This will require a little more patience and mental
scratch space than most chapters, but it’s the best I could do.

## <a href="#single-pass-compilation" id="single-pass-compilation"><span
class="small">17 . 1</span>Single-Pass Compilation</a>

A compiler has roughly two jobs. It parses the user’s source code to
understand what it means. Then it takes that knowledge and outputs
low-level instructions that produce the same semantics. Many languages
split those two roles into two separate <span id="passes">passes</span>
in the implementation. A parser produces an
AST<span class="em">—</span>just like jlox
does<span class="em">—</span>and then a code generator traverses the AST
and outputs target code.

In fact, most sophisticated optimizing compilers have a heck of a lot
more than two passes. Determining not just *what* optimization passes to
have, but how to order them to squeeze the most performance out of the
compiler<span class="em">—</span>since the optimizations often interact
in complex ways<span class="em">—</span>is somewhere between an “open
area of research” and a “dark art”.

In clox, we’re taking an old-school approach and merging these two
passes into one. Back in the day, language hackers did this because
computers literally didn’t have enough memory to store an entire source
file’s AST. We’re doing it because it keeps our compiler simpler, which
is a real asset when programming in C.

Single-pass compilers like we’re going to build don’t work well for all
languages. Since the compiler has only a peephole view into the user’s
program while generating code, the language must be designed such that
you don’t need much surrounding context to understand a piece of syntax.
Fortunately, tiny, dynamically typed Lox is
<span id="lox">well-suited</span> to that.

Not that this should come as much of a surprise. I did design the
language specifically for this book after all.

![Peering through a keyhole at 'var
x;'](image/compiling-expressions/keyhole.png)

What this means in practical terms is that our “compiler” C module has
functionality you’ll recognize from jlox for
parsing<span class="em">—</span>consuming tokens, matching expected
token types, etc. And it also has functions for code
gen<span class="em">—</span>emitting bytecode and adding constants to
the destination chunk. (And it means I’ll use “parsing” and “compiling”
interchangeably throughout this and later chapters.)

We’ll build the parsing and code generation halves first. Then we’ll
stitch them together with the code in the middle that uses Pratt’s
technique to parse Lox’s particular grammar and output the right
bytecode.

## <a href="#parsing-tokens" id="parsing-tokens"><span
class="small">17 . 2</span>Parsing Tokens</a>

First up, the front half of the compiler. This function’s name should
sound familiar.

<div class="codehilite">

``` insert-before
#include "scanner.h"
```

<div class="source-file">

*compiler.c*

</div>

``` insert

static void advance() {
  parser.previous = parser.current;

  for (;;) {
    parser.current = scanToken();
    if (parser.current.type != TOKEN_ERROR) break;

    errorAtCurrent(parser.current.start);
  }
}
```

</div>

<div class="source-file-narrow">

*compiler.c*

</div>

Just like in jlox, it steps forward through the token stream. It asks
the scanner for the next token and stores it for later use. Before doing
that, it takes the old `current` token and stashes that in a `previous`
field. That will come in handy later so that we can get at the lexeme
after we match a token.

The code to read the next token is wrapped in a loop. Remember, clox’s
scanner doesn’t report lexical errors. Instead, it creates special
*error tokens* and leaves it up to the parser to report them. We do that
here.

We keep looping, reading tokens and reporting the errors, until we hit a
non-error one or reach the end. That way, the rest of the parser sees
only valid tokens. The current and previous token are stored in this
struct:

<div class="codehilite">

``` insert-before
#include "scanner.h"
```

<div class="source-file">

*compiler.c*

</div>

``` insert

typedef struct {
  Token current;
  Token previous;
} Parser;

Parser parser;
```

``` insert-after

static void advance() {
```

</div>

<div class="source-file-narrow">

*compiler.c*

</div>

Like we did in other modules, we have a single global variable of this
struct type so we don’t need to pass the state around from function to
function in the compiler.

### <a href="#handling-syntax-errors" id="handling-syntax-errors"><span
class="small">17 . 2 . 1</span>Handling syntax errors</a>

If the scanner hands us an error token, we need to actually tell the
user. That happens using this:

<div class="codehilite">

<div class="source-file">

*compiler.c*  
add after variable *parser*

</div>

    static void errorAtCurrent(const char* message) {
      errorAt(&parser.current, message);
    }

</div>

<div class="source-file-narrow">

*compiler.c*, add after variable *parser*

</div>

We pull the location out of the current token in order to tell the user
where the error occurred and forward it to `errorAt()`. More often,
we’ll report an error at the location of the token we just consumed, so
we give the shorter name to this other function:

<div class="codehilite">

<div class="source-file">

*compiler.c*  
add after variable *parser*

</div>

    static void error(const char* message) {
      errorAt(&parser.previous, message);
    }

</div>

<div class="source-file-narrow">

*compiler.c*, add after variable *parser*

</div>

The actual work happens here:

<div class="codehilite">

<div class="source-file">

*compiler.c*  
add after variable *parser*

</div>

    static void errorAt(Token* token, const char* message) {
      fprintf(stderr, "[line %d] Error", token->line);

      if (token->type == TOKEN_EOF) {
        fprintf(stderr, " at end");
      } else if (token->type == TOKEN_ERROR) {
        // Nothing.
      } else {
        fprintf(stderr, " at '%.*s'", token->length, token->start);
      }

      fprintf(stderr, ": %s\n", message);
      parser.hadError = true;
    }

</div>

<div class="source-file-narrow">

*compiler.c*, add after variable *parser*

</div>

First, we print where the error occurred. We try to show the lexeme if
it’s human-readable. Then we print the error message itself. After that,
we set this `hadError` flag. That records whether any errors occurred
during compilation. This field also lives in the parser struct.

<div class="codehilite">

``` insert-before
  Token previous;
```

<div class="source-file">

*compiler.c*  
in struct *Parser*

</div>

``` insert
  bool hadError;
```

``` insert-after
} Parser;
```

</div>

<div class="source-file-narrow">

*compiler.c*, in struct *Parser*

</div>

Earlier I said that `compile()` should return `false` if an error
occurred. Now we can make it do that.

<div class="codehilite">

``` insert-before
  consume(TOKEN_EOF, "Expect end of expression.");
```

<div class="source-file">

*compiler.c*  
in *compile*()

</div>

``` insert
  return !parser.hadError;
```

``` insert-after
}
```

</div>

<div class="source-file-narrow">

*compiler.c*, in *compile*()

</div>

I’ve got another flag to introduce for error handling. We want to avoid
error cascades. If the user has a mistake in their code and the parser
gets confused about where it is in the grammar, we don’t want it to spew
out a whole pile of meaningless knock-on errors after the first one.

We fixed that in jlox using panic mode error recovery. In the Java
interpreter, we threw an exception to unwind out of all of the parser
code to a point where we could skip tokens and resynchronize. We don’t
have <span id="setjmp">exceptions</span> in C. Instead, we’ll do a
little smoke and mirrors. We add a flag to track whether we’re currently
in panic mode.

There is `setjmp()` and `longjmp()`, but I’d rather not go there. Those
make it too easy to leak memory, forget to maintain invariants, or
otherwise have a Very Bad Day.

<div class="codehilite">

``` insert-before
  bool hadError;
```

<div class="source-file">

*compiler.c*  
in struct *Parser*

</div>

``` insert
  bool panicMode;
```

``` insert-after
} Parser;
```

</div>

<div class="source-file-narrow">

*compiler.c*, in struct *Parser*

</div>

When an error occurs, we set it.

<div class="codehilite">

``` insert-before
static void errorAt(Token* token, const char* message) {
```

<div class="source-file">

*compiler.c*  
in *errorAt*()

</div>

``` insert
  parser.panicMode = true;
```

``` insert-after
  fprintf(stderr, "[line %d] Error", token->line);
```

</div>

<div class="source-file-narrow">

*compiler.c*, in *errorAt*()

</div>

After that, we go ahead and keep compiling as normal as if the error
never occurred. The bytecode will never get executed, so it’s harmless
to keep on trucking. The trick is that while the panic mode flag is set,
we simply suppress any other errors that get detected.

<div class="codehilite">

``` insert-before
static void errorAt(Token* token, const char* message) {
```

<div class="source-file">

*compiler.c*  
in *errorAt*()

</div>

``` insert
  if (parser.panicMode) return;
```

``` insert-after
  parser.panicMode = true;
```

</div>

<div class="source-file-narrow">

*compiler.c*, in *errorAt*()

</div>

There’s a good chance the parser will go off in the weeds, but the user
won’t know because the errors all get swallowed. Panic mode ends when
the parser reaches a synchronization point. For Lox, we chose statement
boundaries, so when we later add those to our compiler, we’ll clear the
flag there.

These new fields need to be initialized.

<div class="codehilite">

``` insert-before
  initScanner(source);
```

<div class="source-file">

*compiler.c*  
in *compile*()

</div>

``` insert

  parser.hadError = false;
  parser.panicMode = false;
```

``` insert-after
  advance();
```

</div>

<div class="source-file-narrow">

*compiler.c*, in *compile*()

</div>

And to display the errors, we need a standard header.

<div class="codehilite">

``` insert-before
#include <stdio.h>
```

<div class="source-file">

*compiler.c*

</div>

``` insert
#include <stdlib.h>
```

``` insert-after

#include "common.h"
```

</div>

<div class="source-file-narrow">

*compiler.c*

</div>

There’s one last parsing function, another old friend from jlox.

<div class="codehilite">

<div class="source-file">

*compiler.c*  
add after *advance*()

</div>

    static void consume(TokenType type, const char* message) {
      if (parser.current.type == type) {
        advance();
        return;
      }

      errorAtCurrent(message);
    }

</div>

<div class="source-file-narrow">

*compiler.c*, add after *advance*()

</div>

It’s similar to `advance()` in that it reads the next token. But it also
validates that the token has an expected type. If not, it reports an
error. This function is the foundation of most syntax errors in the
compiler.

OK, that’s enough on the front end for now.

## <a href="#emitting-bytecode" id="emitting-bytecode"><span
class="small">17 . 3</span>Emitting Bytecode</a>

After we parse and understand a piece of the user’s program, the next
step is to translate that to a series of bytecode instructions. It
starts with the easiest possible step: appending a single byte to the
chunk.

<div class="codehilite">

<div class="source-file">

*compiler.c*  
add after *consume*()

</div>

    static void emitByte(uint8_t byte) {
      writeChunk(currentChunk(), byte, parser.previous.line);
    }

</div>

<div class="source-file-narrow">

*compiler.c*, add after *consume*()

</div>

It’s hard to believe great things will flow through such a simple
function. It writes the given byte, which may be an opcode or an operand
to an instruction. It sends in the previous token’s line information so
that runtime errors are associated with that line.

The chunk that we’re writing gets passed into `compile()`, but it needs
to make its way to `emitByte()`. To do that, we rely on this
intermediary function:

<div class="codehilite">

``` insert-before
Parser parser;
```

<div class="source-file">

*compiler.c*  
add after variable *parser*

</div>

``` insert
Chunk* compilingChunk;

static Chunk* currentChunk() {
  return compilingChunk;
}
```

``` insert-after
static void errorAt(Token* token, const char* message) {
```

</div>

<div class="source-file-narrow">

*compiler.c*, add after variable *parser*

</div>

Right now, the chunk pointer is stored in a module-level variable like
we store other global state. Later, when we start compiling user-defined
functions, the notion of “current chunk” gets more complicated. To avoid
having to go back and change a lot of code, I encapsulate that logic in
the `currentChunk()` function.

We initialize this new module variable before we write any bytecode:

<div class="codehilite">

``` insert-before
bool compile(const char* source, Chunk* chunk) {
  initScanner(source);
```

<div class="source-file">

*compiler.c*  
in *compile*()

</div>

``` insert
  compilingChunk = chunk;
```

``` insert-after

  parser.hadError = false;
```

</div>

<div class="source-file-narrow">

*compiler.c*, in *compile*()

</div>

Then, at the very end, when we’re done compiling the chunk, we wrap
things up.

<div class="codehilite">

``` insert-before
  consume(TOKEN_EOF, "Expect end of expression.");
```

<div class="source-file">

*compiler.c*  
in *compile*()

</div>

``` insert
  endCompiler();
```

``` insert-after
  return !parser.hadError;
```

</div>

<div class="source-file-narrow">

*compiler.c*, in *compile*()

</div>

That calls this:

<div class="codehilite">

<div class="source-file">

*compiler.c*  
add after *emitByte*()

</div>

    static void endCompiler() {
      emitReturn();
    }

</div>

<div class="source-file-narrow">

*compiler.c*, add after *emitByte*()

</div>

In this chapter, our VM deals only with expressions. When you run clox,
it will parse, compile, and execute a single expression, then print the
result. To print that value, we are temporarily using the `OP_RETURN`
instruction. So we have the compiler add one of those to the end of the
chunk.

<div class="codehilite">

<div class="source-file">

*compiler.c*  
add after *emitByte*()

</div>

    static void emitReturn() {
      emitByte(OP_RETURN);
    }

</div>

<div class="source-file-narrow">

*compiler.c*, add after *emitByte*()

</div>

While we’re here in the back end we may as well make our lives easier.

<div class="codehilite">

<div class="source-file">

*compiler.c*  
add after *emitByte*()

</div>

    static void emitBytes(uint8_t byte1, uint8_t byte2) {
      emitByte(byte1);
      emitByte(byte2);
    }

</div>

<div class="source-file-narrow">

*compiler.c*, add after *emitByte*()

</div>

Over time, we’ll have enough cases where we need to write an opcode
followed by a one-byte operand that it’s worth defining this convenience
function.

## <a href="#parsing-prefix-expressions"
id="parsing-prefix-expressions"><span class="small">17 . 4</span>Parsing
Prefix Expressions</a>

We’ve assembled our parsing and code generation utility functions. The
missing piece is the code in the middle that connects those together.

![Parsing functions on the left, bytecode emitting functions on the
right. What goes in the
middle?](image/compiling-expressions/mystery.png)

The only step in `compile()` that we have left to implement is this
function:

<div class="codehilite">

<div class="source-file">

*compiler.c*  
add after *endCompiler*()

</div>

    static void expression() {
      // What goes here?
    }

</div>

<div class="source-file-narrow">

*compiler.c*, add after *endCompiler*()

</div>

We aren’t ready to implement every kind of expression in Lox yet. Heck,
we don’t even have Booleans. For this chapter, we’re only going to worry
about four:

- Number literals: `123`
- Parentheses for grouping: `(123)`
- Unary negation: `-123`
- The Four Horsemen of the Arithmetic: `+`, `-`, `*`, `/`

As we work through the functions to compile each of those kinds of
expressions, we’ll also assemble the requirements for the table-driven
parser that calls them.

### <a href="#parsers-for-tokens" id="parsers-for-tokens"><span
class="small">17 . 4 . 1</span>Parsers for tokens</a>

For now, let’s focus on the Lox expressions that are each only a single
token. In this chapter, that’s just number literals, but there will be
more later. Here’s how we can compile them:

We map each token type to a different kind of expression. We define a
function for each expression that outputs the appropriate bytecode. Then
we build an array of function pointers. The indexes in the array
correspond to the `TokenType` enum values, and the function at each
index is the code to compile an expression of that token type.

To compile number literals, we store a pointer to the following function
at the `TOKEN_NUMBER` index in the array.

<div class="codehilite">

<div class="source-file">

*compiler.c*  
add after *endCompiler*()

</div>

    static void number() {
      double value = strtod(parser.previous.start, NULL);
      emitConstant(value);
    }

</div>

<div class="source-file-narrow">

*compiler.c*, add after *endCompiler*()

</div>

We assume the token for the number literal has already been consumed and
is stored in `previous`. We take that lexeme and use the C standard
library to convert it to a double value. Then we generate the code to
load that value using this function:

<div class="codehilite">

<div class="source-file">

*compiler.c*  
add after *emitReturn*()

</div>

    static void emitConstant(Value value) {
      emitBytes(OP_CONSTANT, makeConstant(value));
    }

</div>

<div class="source-file-narrow">

*compiler.c*, add after *emitReturn*()

</div>

First, we add the value to the constant table, then we emit an
`OP_CONSTANT` instruction that pushes it onto the stack at runtime. To
insert an entry in the constant table, we rely on:

<div class="codehilite">

<div class="source-file">

*compiler.c*  
add after *emitReturn*()

</div>

    static uint8_t makeConstant(Value value) {
      int constant = addConstant(currentChunk(), value);
      if (constant > UINT8_MAX) {
        error("Too many constants in one chunk.");
        return 0;
      }

      return (uint8_t)constant;
    }

</div>

<div class="source-file-narrow">

*compiler.c*, add after *emitReturn*()

</div>

Most of the work happens in `addConstant()`, which we defined back in an
[earlier chapter](chunks-of-bytecode.html). That adds the given value to
the end of the chunk’s constant table and returns its index. The new
function’s job is mostly to make sure we don’t have too many constants.
Since the `OP_CONSTANT` instruction uses a single byte for the index
operand, we can store and load only up to <span id="256">256</span>
constants in a chunk.

Yes, that limit is pretty low. If this were a full-sized language
implementation, we’d want to add another instruction like
`OP_CONSTANT_16` that stores the index as a two-byte operand so we could
handle more constants when needed.

The code to support that isn’t particularly illuminating, so I omitted
it from clox, but you’ll want your VMs to scale to larger programs.

That’s basically all it takes. Provided there is some suitable code that
consumes a `TOKEN_NUMBER` token, looks up `number()` in the function
pointer array, and then calls it, we can now compile number literals to
bytecode.

### <a href="#parentheses-for-grouping" id="parentheses-for-grouping"><span
class="small">17 . 4 . 2</span>Parentheses for grouping</a>

Our as-yet-imaginary array of parsing function pointers would be great
if every expression was only a single token long. Alas, most are longer.
However, many expressions *start* with a particular token. We call these
*prefix* expressions. For example, when we’re parsing an expression and
the current token is `(`, we know we must be looking at a parenthesized
grouping expression.

It turns out our function pointer array handles those too. The parsing
function for an expression type can consume any additional tokens that
it wants to, just like in a regular recursive descent parser. Here’s how
parentheses work:

<div class="codehilite">

<div class="source-file">

*compiler.c*  
add after *endCompiler*()

</div>

    static void grouping() {
      expression();
      consume(TOKEN_RIGHT_PAREN, "Expect ')' after expression.");
    }

</div>

<div class="source-file-narrow">

*compiler.c*, add after *endCompiler*()

</div>

Again, we assume the initial `(` has already been consumed. We
<span id="recursive">recursively</span> call back into `expression()` to
compile the expression between the parentheses, then parse the closing
`)` at the end.

A Pratt parser isn’t a recursive *descent* parser, but it’s still
recursive. That’s to be expected since the grammar itself is recursive.

As far as the back end is concerned, there’s literally nothing to a
grouping expression. Its sole function is
syntactic<span class="em">—</span>it lets you insert a lower-precedence
expression where a higher precedence is expected. Thus, it has no
runtime semantics on its own and therefore doesn’t emit any bytecode.
The inner call to `expression()` takes care of generating bytecode for
the expression inside the parentheses.

### <a href="#unary-negation" id="unary-negation"><span
class="small">17 . 4 . 3</span>Unary negation</a>

Unary minus is also a prefix expression, so it works with our model too.

<div class="codehilite">

<div class="source-file">

*compiler.c*  
add after *number*()

</div>

    static void unary() {
      TokenType operatorType = parser.previous.type;

      // Compile the operand.
      expression();

      // Emit the operator instruction.
      switch (operatorType) {
        case TOKEN_MINUS: emitByte(OP_NEGATE); break;
        default: return; // Unreachable.
      }
    }

</div>

<div class="source-file-narrow">

*compiler.c*, add after *number*()

</div>

The leading `-` token has been consumed and is sitting in
`parser.previous`. We grab the token type from that to note which unary
operator we’re dealing with. It’s unnecessary right now, but this will
make more sense when we use this same function to compile the `!`
operator in [the next chapter](types-of-values.html).

As in `grouping()`, we recursively call `expression()` to compile the
operand. After that, we emit the bytecode to perform the negation. It
might seem a little weird to write the negate instruction *after* its
operand’s bytecode since the `-` appears on the left, but think about it
in terms of order of execution:

1.  We evaluate the operand first which leaves its value on the stack.

2.  Then we pop that value, negate it, and push the result.

So the `OP_NEGATE` instruction should be emitted
<span id="line">last</span>. This is part of the compiler’s
job<span class="em">—</span>parsing the program in the order it appears
in the source code and rearranging it into the order that execution
happens.

Emitting the `OP_NEGATE` instruction after the operands does mean that
the current token when the bytecode is written is *not* the `-` token.
That mostly doesn’t matter, except that we use that token for the line
number to associate with that instruction.

This means if you have a multi-line negation expression, like:

<div class="codehilite">

    print -
      true;

</div>

Then the runtime error will be reported on the wrong line. Here, it
would show the error on line 2, even though the `-` is on line 1. A more
robust approach would be to store the token’s line before compiling the
operand and then pass that into `emitByte()`, but I wanted to keep
things simple for the book.

There is one problem with this code, though. The `expression()` function
it calls will parse any expression for the operand, regardless of
precedence. Once we add binary operators and other syntax, that will do
the wrong thing. Consider:

<div class="codehilite">

    -a.b + c;

</div>

Here, the operand to `-` should be just the `a.b` expression, not the
entire `a.b + c`. But if `unary()` calls `expression()`, the latter will
happily chew through all of the remaining code including the `+`. It
will erroneously treat the `-` as lower precedence than the `+`.

When parsing the operand to unary `-`, we need to compile only
expressions at a certain precedence level or higher. In jlox’s recursive
descent parser we accomplished that by calling into the parsing method
for the lowest-precedence expression we wanted to allow (in this case,
`call()`). Each method for parsing a specific expression also parsed any
expressions of higher precedence too, so that included the rest of the
precedence table.

The parsing functions like `number()` and `unary()` here in clox are
different. Each only parses exactly one type of expression. They don’t
cascade to include higher-precedence expression types too. We need a
different solution, and it looks like this:

<div class="codehilite">

<div class="source-file">

*compiler.c*  
add after *unary*()

</div>

    static void parsePrecedence(Precedence precedence) {
      // What goes here?
    }

</div>

<div class="source-file-narrow">

*compiler.c*, add after *unary*()

</div>

This function<span class="em">—</span>once we implement
it<span class="em">—</span>starts at the current token and parses any
expression at the given precedence level or higher. We have some other
setup to get through before we can write the body of this function, but
you can probably guess that it will use that table of parsing function
pointers I’ve been talking about. For now, don’t worry too much about
how it works. In order to take the “precedence” as a parameter, we
define it numerically.

<div class="codehilite">

``` insert-before
} Parser;
```

<div class="source-file">

*compiler.c*  
add after struct *Parser*

</div>

``` insert

typedef enum {
  PREC_NONE,
  PREC_ASSIGNMENT,  // =
  PREC_OR,          // or
  PREC_AND,         // and
  PREC_EQUALITY,    // == !=
  PREC_COMPARISON,  // < > <= >=
  PREC_TERM,        // + -
  PREC_FACTOR,      // * /
  PREC_UNARY,       // ! -
  PREC_CALL,        // . ()
  PREC_PRIMARY
} Precedence;
```

``` insert-after

Parser parser;
```

</div>

<div class="source-file-narrow">

*compiler.c*, add after struct *Parser*

</div>

These are all of Lox’s precedence levels in order from lowest to
highest. Since C implicitly gives successively larger numbers for enums,
this means that `PREC_CALL` is numerically larger than `PREC_UNARY`. For
example, say the compiler is sitting on a chunk of code like:

<div class="codehilite">

    -a.b + c

</div>

If we call `parsePrecedence(PREC_ASSIGNMENT)`, then it will parse the
entire expression because `+` has higher precedence than assignment. If
instead we call `parsePrecedence(PREC_UNARY)`, it will compile the
`-a.b` and stop there. It doesn’t keep going through the `+` because the
addition has lower precedence than unary operators.

With this function in hand, it’s a snap to fill in the missing body for
`expression()`.

<div class="codehilite">

``` insert-before
static void expression() {
```

<div class="source-file">

*compiler.c*  
in *expression*()  
replace 1 line

</div>

``` insert
  parsePrecedence(PREC_ASSIGNMENT);
```

``` insert-after
}
```

</div>

<div class="source-file-narrow">

*compiler.c*, in *expression*(), replace 1 line

</div>

We simply parse the lowest precedence level, which subsumes all of the
higher-precedence expressions too. Now, to compile the operand for a
unary expression, we call this new function and limit it to the
appropriate level:

<div class="codehilite">

``` insert-before
  // Compile the operand.
```

<div class="source-file">

*compiler.c*  
in *unary*()  
replace 1 line

</div>

``` insert
  parsePrecedence(PREC_UNARY);
```

``` insert-after

  // Emit the operator instruction.
```

</div>

<div class="source-file-narrow">

*compiler.c*, in *unary*(), replace 1 line

</div>

We use the unary operator’s own `PREC_UNARY` precedence to permit
<span id="useful">nested</span> unary expressions like
`!!doubleNegative`. Since unary operators have pretty high precedence,
that correctly excludes things like binary operators. Speaking of
which<span class="ellipse"> . . . </span>

Not that nesting unary expressions is particularly useful in Lox. But
other languages let you do it, so we do too.

## <a href="#parsing-infix-expressions"
id="parsing-infix-expressions"><span class="small">17 . 5</span>Parsing
Infix Expressions</a>

Binary operators are different from the previous expressions because
they are *infix*. With the other expressions, we know what we are
parsing from the very first token. With infix expressions, we don’t know
we’re in the middle of a binary operator until *after* we’ve parsed its
left operand and then stumbled onto the operator token in the middle.

Here’s an example:

<div class="codehilite">

    1 + 2

</div>

Let’s walk through trying to compile it with what we know so far:

1.  We call `expression()`. That in turn calls
    `parsePrecedence(PREC_ASSIGNMENT)`.

2.  That function (once we implement it) sees the leading number token
    and recognizes it is parsing a number literal. It hands off control
    to `number()`.

3.  `number()` creates a constant, emits an `OP_CONSTANT`, and returns
    back to `parsePrecedence()`.

Now what? The call to `parsePrecedence()` should consume the entire
addition expression, so it needs to keep going somehow. Fortunately, the
parser is right where we need it to be. Now that we’ve compiled the
leading number expression, the next token is `+`. That’s the exact token
that `parsePrecedence()` needs to detect that we’re in the middle of an
infix expression and to realize that the expression we already compiled
is actually an operand to that.

So this hypothetical array of function pointers doesn’t just list
functions to parse expressions that start with a given token. Instead,
it’s a *table* of function pointers. One column associates prefix parser
functions with token types. The second column associates infix parser
functions with token types.

The function we will use as the infix parser for `TOKEN_PLUS`,
`TOKEN_MINUS`, `TOKEN_STAR`, and `TOKEN_SLASH` is this:

<div class="codehilite">

<div class="source-file">

*compiler.c*  
add after *endCompiler*()

</div>

    static void binary() {
      TokenType operatorType = parser.previous.type;
      ParseRule* rule = getRule(operatorType);
      parsePrecedence((Precedence)(rule->precedence + 1));

      switch (operatorType) {
        case TOKEN_PLUS:          emitByte(OP_ADD); break;
        case TOKEN_MINUS:         emitByte(OP_SUBTRACT); break;
        case TOKEN_STAR:          emitByte(OP_MULTIPLY); break;
        case TOKEN_SLASH:         emitByte(OP_DIVIDE); break;
        default: return; // Unreachable.
      }
    }

</div>

<div class="source-file-narrow">

*compiler.c*, add after *endCompiler*()

</div>

When a prefix parser function is called, the leading token has already
been consumed. An infix parser function is even more *in medias
res*<span class="em">—</span>the entire left-hand operand expression has
already been compiled and the subsequent infix operator consumed.

The fact that the left operand gets compiled first works out fine. It
means at runtime, that code gets executed first. When it runs, the value
it produces will end up on the stack. That’s right where the infix
operator is going to need it.

Then we come here to `binary()` to handle the rest of the arithmetic
operators. This function compiles the right operand, much like how
`unary()` compiles its own trailing operand. Finally, it emits the
bytecode instruction that performs the binary operation.

When run, the VM will execute the left and right operand code, in that
order, leaving their values on the stack. Then it executes the
instruction for the operator. That pops the two values, computes the
operation, and pushes the result.

The code that probably caught your eye here is that `getRule()` line.
When we parse the right-hand operand, we again need to worry about
precedence. Take an expression like:

<div class="codehilite">

    2 * 3 + 4

</div>

When we parse the right operand of the `*` expression, we need to just
capture `3`, and not `3 + 4`, because `+` is lower precedence than `*`.
We could define a separate function for each binary operator. Each would
call `parsePrecedence()` and pass in the correct precedence level for
its operand.

But that’s kind of tedious. Each binary operator’s right-hand operand
precedence is one level <span id="higher">higher</span> than its own. We
can look that up dynamically with this `getRule()` thing we’ll get to
soon. Using that, we call `parsePrecedence()` with one level higher than
this operator’s level.

We use one *higher* level of precedence for the right operand because
the binary operators are left-associative. Given a series of the *same*
operator, like:

<div class="codehilite">

    1 + 2 + 3 + 4

</div>

We want to parse it like:

<div class="codehilite">

    ((1 + 2) + 3) + 4

</div>

Thus, when parsing the right-hand operand to the first `+`, we want to
consume the `2`, but not the rest, so we use one level above `+`’s
precedence. But if our operator was *right*-associative, this would be
wrong. Given:

<div class="codehilite">

    a = b = c = d

</div>

Since assignment is right-associative, we want to parse it as:

<div class="codehilite">

    a = (b = (c = d))

</div>

To enable that, we would call `parsePrecedence()` with the *same*
precedence as the current operator.

This way, we can use a single `binary()` function for all binary
operators even though they have different precedences.

## <a href="#a-pratt-parser" id="a-pratt-parser"><span
class="small">17 . 6</span>A Pratt Parser</a>

We now have all of the pieces and parts of the compiler laid out. We
have a function for each grammar production: `number()`, `grouping()`,
`unary()`, and `binary()`. We still need to implement
`parsePrecedence()`, and `getRule()`. We also know we need a table that,
given a token type, lets us find

- the function to compile a prefix expression starting with a token of
  that type,

- the function to compile an infix expression whose left operand is
  followed by a token of that type, and

- the precedence of an <span id="prefix">infix</span> expression that
  uses that token as an operator.

We don’t need to track the precedence of the *prefix* expression
starting with a given token because all prefix operators in Lox have the
same precedence.

We wrap these three properties in a little struct which represents a
single row in the parser table.

<div class="codehilite">

``` insert-before
} Precedence;
```

<div class="source-file">

*compiler.c*  
add after enum *Precedence*

</div>

``` insert

typedef struct {
  ParseFn prefix;
  ParseFn infix;
  Precedence precedence;
} ParseRule;
```

``` insert-after

Parser parser;
```

</div>

<div class="source-file-narrow">

*compiler.c*, add after enum *Precedence*

</div>

That ParseFn type is a simple <span id="typedef">typedef</span> for a
function type that takes no arguments and returns nothing.

C’s syntax for function pointer types is so bad that I always hide it
behind a typedef. I understand the intent behind the
syntax<span class="em">—</span>the whole “declaration reflects use”
thing<span class="em">—</span>but I think it was a failed syntactic
experiment.

<div class="codehilite">

``` insert-before
} Precedence;
```

<div class="source-file">

*compiler.c*  
add after enum *Precedence*

</div>

``` insert

typedef void (*ParseFn)();
```

``` insert-after

typedef struct {
```

</div>

<div class="source-file-narrow">

*compiler.c*, add after enum *Precedence*

</div>

The table that drives our whole parser is an array of ParseRules. We’ve
been talking about it forever, and finally you get to see it.

<div class="codehilite">

<div class="source-file">

*compiler.c*  
add after *unary*()

</div>

    ParseRule rules[] = {
      [TOKEN_LEFT_PAREN]    = {grouping, NULL,   PREC_NONE},
      [TOKEN_RIGHT_PAREN]   = {NULL,     NULL,   PREC_NONE},
      [TOKEN_LEFT_BRACE]    = {NULL,     NULL,   PREC_NONE}, 
      [TOKEN_RIGHT_BRACE]   = {NULL,     NULL,   PREC_NONE},
      [TOKEN_COMMA]         = {NULL,     NULL,   PREC_NONE},
      [TOKEN_DOT]           = {NULL,     NULL,   PREC_NONE},
      [TOKEN_MINUS]         = {unary,    binary, PREC_TERM},
      [TOKEN_PLUS]          = {NULL,     binary, PREC_TERM},
      [TOKEN_SEMICOLON]     = {NULL,     NULL,   PREC_NONE},
      [TOKEN_SLASH]         = {NULL,     binary, PREC_FACTOR},
      [TOKEN_STAR]          = {NULL,     binary, PREC_FACTOR},
      [TOKEN_BANG]          = {NULL,     NULL,   PREC_NONE},
      [TOKEN_BANG_EQUAL]    = {NULL,     NULL,   PREC_NONE},
      [TOKEN_EQUAL]         = {NULL,     NULL,   PREC_NONE},
      [TOKEN_EQUAL_EQUAL]   = {NULL,     NULL,   PREC_NONE},
      [TOKEN_GREATER]       = {NULL,     NULL,   PREC_NONE},
      [TOKEN_GREATER_EQUAL] = {NULL,     NULL,   PREC_NONE},
      [TOKEN_LESS]          = {NULL,     NULL,   PREC_NONE},
      [TOKEN_LESS_EQUAL]    = {NULL,     NULL,   PREC_NONE},
      [TOKEN_IDENTIFIER]    = {NULL,     NULL,   PREC_NONE},
      [TOKEN_STRING]        = {NULL,     NULL,   PREC_NONE},
      [TOKEN_NUMBER]        = {number,   NULL,   PREC_NONE},
      [TOKEN_AND]           = {NULL,     NULL,   PREC_NONE},
      [TOKEN_CLASS]         = {NULL,     NULL,   PREC_NONE},
      [TOKEN_ELSE]          = {NULL,     NULL,   PREC_NONE},
      [TOKEN_FALSE]         = {NULL,     NULL,   PREC_NONE},
      [TOKEN_FOR]           = {NULL,     NULL,   PREC_NONE},
      [TOKEN_FUN]           = {NULL,     NULL,   PREC_NONE},
      [TOKEN_IF]            = {NULL,     NULL,   PREC_NONE},
      [TOKEN_NIL]           = {NULL,     NULL,   PREC_NONE},
      [TOKEN_OR]            = {NULL,     NULL,   PREC_NONE},
      [TOKEN_PRINT]         = {NULL,     NULL,   PREC_NONE},
      [TOKEN_RETURN]        = {NULL,     NULL,   PREC_NONE},
      [TOKEN_SUPER]         = {NULL,     NULL,   PREC_NONE},
      [TOKEN_THIS]          = {NULL,     NULL,   PREC_NONE},
      [TOKEN_TRUE]          = {NULL,     NULL,   PREC_NONE},
      [TOKEN_VAR]           = {NULL,     NULL,   PREC_NONE},
      [TOKEN_WHILE]         = {NULL,     NULL,   PREC_NONE},
      [TOKEN_ERROR]         = {NULL,     NULL,   PREC_NONE},
      [TOKEN_EOF]           = {NULL,     NULL,   PREC_NONE},
    };

</div>

<div class="source-file-narrow">

*compiler.c*, add after *unary*()

</div>

See what I mean about not wanting to revisit the table each time we
needed a new column? It’s a beast.

If you haven’t seen the `[TOKEN_DOT] =` syntax in a C array literal,
that is C99’s designated initializer syntax. It’s clearer than having to
count array indexes by hand.

You can see how `grouping` and `unary` are slotted into the prefix
parser column for their respective token types. In the next column,
`binary` is wired up to the four arithmetic infix operators. Those infix
operators also have their precedences set in the last column.

Aside from those, the rest of the table is full of `NULL` and
`PREC_NONE`. Most of those empty cells are because there is no
expression associated with those tokens. You can’t start an expression
with, say, `else`, and `}` would make for a pretty confusing infix
operator.

But, also, we haven’t filled in the entire grammar yet. In later
chapters, as we add new expression types, some of these slots will get
functions in them. One of the things I like about this approach to
parsing is that it makes it very easy to see which tokens are in use by
the grammar and which are available.

Now that we have the table, we are finally ready to write the code that
uses it. This is where our Pratt parser comes to life. The easiest
function to define is `getRule()`.

<div class="codehilite">

<div class="source-file">

*compiler.c*  
add after *parsePrecedence*()

</div>

    static ParseRule* getRule(TokenType type) {
      return &rules[type];
    }

</div>

<div class="source-file-narrow">

*compiler.c*, add after *parsePrecedence*()

</div>

It simply returns the rule at the given index. It’s called by `binary()`
to look up the precedence of the current operator. This function exists
solely to handle a declaration cycle in the C code. `binary()` is
defined *before* the rules table so that the table can store a pointer
to it. That means the body of `binary()` cannot access the table
directly.

Instead, we wrap the lookup in a function. That lets us forward declare
`getRule()` before the definition of `binary()`, and
<span id="forward">then</span> *define* `getRule()` after the table.
We’ll need a couple of other forward declarations to handle the fact
that our grammar is recursive, so let’s get them all out of the way.

This is what happens when you write your VM in a language that was
designed to be compiled on a PDP-11.

<div class="codehilite">

``` insert-before
  emitReturn();
}
```

<div class="source-file">

*compiler.c*  
add after *endCompiler*()

</div>

``` insert

static void expression();
static ParseRule* getRule(TokenType type);
static void parsePrecedence(Precedence precedence);
```

``` insert-after
static void binary() {
```

</div>

<div class="source-file-narrow">

*compiler.c*, add after *endCompiler*()

</div>

If you’re following along and implementing clox yourself, pay close
attention to the little annotations that tell you where to put these
code snippets. Don’t worry, though, if you get it wrong, the C compiler
will be happy to tell you.

### <a href="#parsing-with-precedence" id="parsing-with-precedence"><span
class="small">17 . 6 . 1</span>Parsing with precedence</a>

Now we’re getting to the fun stuff. The maestro that orchestrates all of
the parsing functions we’ve defined is `parsePrecedence()`. Let’s start
with parsing prefix expressions.

<div class="codehilite">

``` insert-before
static void parsePrecedence(Precedence precedence) {
```

<div class="source-file">

*compiler.c*  
in *parsePrecedence*()  
replace 1 line

</div>

``` insert
  advance();
  ParseFn prefixRule = getRule(parser.previous.type)->prefix;
  if (prefixRule == NULL) {
    error("Expect expression.");
    return;
  }

  prefixRule();
```

``` insert-after
}
```

</div>

<div class="source-file-narrow">

*compiler.c*, in *parsePrecedence*(), replace 1 line

</div>

We read the next token and look up the corresponding ParseRule. If there
is no prefix parser, then the token must be a syntax error. We report
that and return to the caller.

Otherwise, we call that prefix parse function and let it do its thing.
That prefix parser compiles the rest of the prefix expression, consuming
any other tokens it needs, and returns back here. Infix expressions are
where it gets interesting since precedence comes into play. The
implementation is remarkably simple.

<div class="codehilite">

``` insert-before
  prefixRule();
```

<div class="source-file">

*compiler.c*  
in *parsePrecedence*()

</div>

``` insert

  while (precedence <= getRule(parser.current.type)->precedence) {
    advance();
    ParseFn infixRule = getRule(parser.previous.type)->infix;
    infixRule();
  }
```

``` insert-after
}
```

</div>

<div class="source-file-narrow">

*compiler.c*, in *parsePrecedence*()

</div>

That’s the whole thing. Really. Here’s how the entire function works: At
the beginning of `parsePrecedence()`, we look up a prefix parser for the
current token. The first token is *always* going to belong to some kind
of prefix expression, by definition. It may turn out to be nested as an
operand inside one or more infix expressions, but as you read the code
from left to right, the first token you hit always belongs to a prefix
expression.

After parsing that, which may consume more tokens, the prefix expression
is done. Now we look for an infix parser for the next token. If we find
one, it means the prefix expression we already compiled might be an
operand for it. But only if the call to `parsePrecedence()` has a
`precedence` that is low enough to permit that infix operator.

If the next token is too low precedence, or isn’t an infix operator at
all, we’re done. We’ve parsed as much expression as we can. Otherwise,
we consume the operator and hand off control to the infix parser we
found. It consumes whatever other tokens it needs (usually the right
operand) and returns back to `parsePrecedence()`. Then we loop back
around and see if the *next* token is also a valid infix operator that
can take the entire preceding expression as its operand. We keep looping
like that, crunching through infix operators and their operands until we
hit a token that isn’t an infix operator or is too low precedence and
stop.

That’s a lot of prose, but if you really want to mind meld with Vaughan
Pratt and fully understand the algorithm, step through the parser in
your debugger as it works through some expressions. Maybe a picture will
help. There’s only a handful of functions, but they are marvelously
intertwined:

<span id="connections"></span>

![The various parsing functions and how they call each
other.](image/compiling-expressions/connections.png)

The <img src="image/compiling-expressions/calls.png" class="arrow"
alt="A solid arrow." /> arrow connects a function to another function it
directly calls. The
<img src="image/compiling-expressions/points-to.png" class="arrow"
alt="An open arrow." /> arrow shows the table’s pointers to the parsing
functions.

Later, we’ll need to tweak the code in this chapter to handle
assignment. But, otherwise, what we wrote covers all of our expression
compiling needs for the rest of the book. We’ll plug additional parsing
functions into the table when we add new kinds of expressions, but
`parsePrecedence()` is complete.

## <a href="#dumping-chunks" id="dumping-chunks"><span
class="small">17 . 7</span>Dumping Chunks</a>

While we’re here in the core of our compiler, we should put in some
instrumentation. To help debug the generated bytecode, we’ll add support
for dumping the chunk once the compiler finishes. We had some temporary
logging earlier when we hand-authored the chunk. Now we’ll put in some
real code so that we can enable it whenever we want.

Since this isn’t for end users, we hide it behind a flag.

<div class="codehilite">

``` insert-before
#include <stdint.h>
```

<div class="source-file">

*common.h*

</div>

``` insert
#define DEBUG_PRINT_CODE
```

``` insert-after
#define DEBUG_TRACE_EXECUTION
```

</div>

<div class="source-file-narrow">

*common.h*

</div>

When that flag is defined, we use our existing “debug” module to print
out the chunk’s bytecode.

<div class="codehilite">

``` insert-before
  emitReturn();
```

<div class="source-file">

*compiler.c*  
in *endCompiler*()

</div>

``` insert
#ifdef DEBUG_PRINT_CODE
  if (!parser.hadError) {
    disassembleChunk(currentChunk(), "code");
  }
#endif
```

``` insert-after
}
```

</div>

<div class="source-file-narrow">

*compiler.c*, in *endCompiler*()

</div>

We do this only if the code was free of errors. After a syntax error,
the compiler keeps on going but it’s in kind of a weird state and might
produce broken code. That’s harmless because it won’t get executed, but
we’ll just confuse ourselves if we try to read it.

Finally, to access `disassembleChunk()`, we need to include its header.

<div class="codehilite">

``` insert-before
#include "scanner.h"
```

<div class="source-file">

*compiler.c*

</div>

``` insert

#ifdef DEBUG_PRINT_CODE
#include "debug.h"
#endif
```

``` insert-after

typedef struct {
```

</div>

<div class="source-file-narrow">

*compiler.c*

</div>

We made it! This was the last major section to install in our VM’s
compilation and execution pipeline. Our interpreter doesn’t *look* like
much, but inside it is scanning, parsing, compiling to bytecode, and
executing.

Fire up the VM and type in an expression. If we did everything right, it
should calculate and print the result. We now have a very
over-engineered arithmetic calculator. We have a lot of language
features to add in the coming chapters, but the foundation is in place.

<div class="challenges">

## <a href="#challenges" id="challenges">Challenges</a>

1.  To really understand the parser, you need to see how execution
    threads through the interesting parsing
    functions<span class="em">—</span>`parsePrecedence()` and the parser
    functions stored in the table. Take this (strange) expression:

    <div class="codehilite">

        (-1 + 2) * 3 - -4

    </div>

    Write a trace of how those functions are called. Show the order they
    are called, which calls which, and the arguments passed to them.

2.  The ParseRule row for `TOKEN_MINUS` has both prefix and infix
    function pointers. That’s because `-` is both a prefix operator
    (unary negation) and an infix one (subtraction).

    In the full Lox language, what other tokens can be used in both
    prefix and infix positions? What about in C or in another language
    of your choice?

3.  You might be wondering about complex “mixfix” expressions that have
    more than two operands separated by tokens. C’s conditional or
    “ternary” operator, `?:`, is a widely known one.

    Add support for that operator to the compiler. You don’t have to
    generate any bytecode, just show how you would hook it up to the
    parser and handle the operands.

</div>

<div class="design-note">

## <a href="#design-note" id="design-note">Design Note: It’s Just
Parsing</a>

I’m going to make a claim here that will be unpopular with some compiler
and language people. It’s OK if you don’t agree. Personally, I learn
more from strongly stated opinions that I disagree with than I do from
several pages of qualifiers and equivocation. My claim is that *parsing
doesn’t matter*.

Over the years, many programming language people, especially in
academia, have gotten *really* into parsers and taken them very
seriously. Initially, it was the compiler folks who got into
<span id="yacc">compiler-compilers</span>, LALR, and other stuff like
that. The first half of the dragon book is a long love letter to the
wonders of parser generators.

All of us suffer from the vice of “when all you have is a hammer,
everything looks like a nail”, but perhaps none so visibly as compiler
people. You wouldn’t believe the breadth of software problems that
miraculously seem to require a new little language in their solution as
soon as you ask a compiler hacker for help.

Yacc and other compiler-compilers are the most delightfully recursive
example. “Wow, writing compilers is a chore. I know, let’s write a
compiler to write our compiler for us.”

For the record, I don’t claim immunity to this affliction.

Later, the functional programming folks got into parser combinators,
packrat parsers, and other sorts of things. Because, obviously, if you
give a functional programmer a problem, the first thing they’ll do is
whip out a pocketful of higher-order functions.

Over in math and algorithm analysis land, there is a long legacy of
research into proving time and memory usage for various parsing
techniques, transforming parsing problems into other problems and back,
and assigning complexity classes to different grammars.

At one level, this stuff is important. If you’re implementing a
language, you want some assurance that your parser won’t go exponential
and take 7,000 years to parse a weird edge case in the grammar. Parser
theory gives you that bound. As an intellectual exercise, learning about
parsing techniques is also fun and rewarding.

But if your goal is just to implement a language and get it in front of
users, almost all of that stuff doesn’t matter. It’s really easy to get
worked up by the enthusiasm of the people who *are* into it and think
that your front end *needs* some whiz-bang generated
combinator-parser-factory thing. I’ve seen people burn tons of time
writing and rewriting their parser using whatever today’s hot library or
technique is.

That’s time that doesn’t add any value to your user’s life. If you’re
just trying to get your parser done, pick one of the bog-standard
techniques, use it, and move on. Recursive descent, Pratt parsing, and
the popular parser generators like ANTLR or Bison are all fine.

Take the extra time you saved not rewriting your parsing code and spend
it improving the compile error messages your compiler shows users. Good
error handling and reporting is more valuable to users than almost
anything else you can put time into in the front end.

</div>

<a href="types-of-values.html" class="next">Next Chapter: “Types of
Values” →</a> Handcrafted by Robert Nystrom — <a
href="https://github.com/munificent/craftinginterpreters/blob/master/LICENSE"
target="_blank">© 2015 – 2021</a>

</div>

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

[![](image/logotype.png "Crafting Interpreters")](/)

<div class="contents">

### [Strings<span class="small">19</span>](#top)

- [<span class="small">19.1</span> Values and
  Objects](#values-and-objects)
- [<span class="small">19.2</span> Struct
  Inheritance](#struct-inheritance)
- [<span class="small">19.3</span> Strings](#strings)
- [<span class="small">19.4</span> Operations on
  Strings](#operations-on-strings)
- [<span class="small">19.5</span> Freeing Objects](#freeing-objects)
- 
- [Challenges](#challenges)
- [<span class="small">note</span>String Encoding](#design-note)

<div class="prev-next">

<a href="types-of-values.html" class="left"
title="Types of Values">← Previous</a>
[↑ Up](a-bytecode-virtual-machine.html "A Bytecode Virtual Machine")
<a href="hash-tables.html" class="right" title="Hash Tables">Next →</a>

</div>

</div>

[![](image/logotype.png "Crafting Interpreters")](/)
<a href="types-of-values.html" class="prev"
title="Types of Values">←</a>
<a href="hash-tables.html" class="next" title="Hash Tables">→</a>

<div class="page">

<div class="nav-wrapper">

[![](image/logotype.png "Crafting Interpreters")](/)

<div class="expandable">

### [Strings<span class="small">19</span>](#top)

- [<span class="small">19.1</span> Values and
  Objects](#values-and-objects)
- [<span class="small">19.2</span> Struct
  Inheritance](#struct-inheritance)
- [<span class="small">19.3</span> Strings](#strings)
- [<span class="small">19.4</span> Operations on
  Strings](#operations-on-strings)
- [<span class="small">19.5</span> Freeing Objects](#freeing-objects)
- 
- [Challenges](#challenges)
- [<span class="small">note</span>String Encoding](#design-note)

<div class="prev-next">

<a href="types-of-values.html" class="left"
title="Types of Values">← Previous</a>
[↑ Up](a-bytecode-virtual-machine.html "A Bytecode Virtual Machine")
<a href="hash-tables.html" class="right" title="Hash Tables">Next →</a>

</div>

</div>

<span id="expand-nav">≡</span>

</div>

<div class="number">

19

</div>

# Strings

> “Ah? A small aversion to menial labor?” The doctor cocked an eyebrow.
> “Understandable, but misplaced. One should treasure those hum-drum
> tasks that keep the body occupied but leave the mind and heart
> unfettered.”
>
> Tad Williams, *The Dragonbone Chair*

Our little VM can represent three types of values right now: numbers,
Booleans, and `nil`. Those types have two important things in common:
they’re immutable and they’re small. Numbers are the largest, and they
still fit into two 64-bit words. That’s a small enough price that we can
afford to pay it for all values, even Booleans and nils which don’t need
that much space.

Strings, unfortunately, are not so petite. There’s no maximum length for
a string. Even if we were to artificially cap it at some contrived limit
like <span id="pascal">255</span> characters, that’s still too much
memory to spend on every single value.

UCSD Pascal, one of the first implementations of Pascal, had this exact
limit. Instead of using a terminating null byte to indicate the end of
the string like C, Pascal strings started with a length value. Since
UCSD used only a single byte to store the length, strings couldn’t be
any longer than 255 characters.

![The Pascal string 'hello' with a length byte of 5 preceding
it.](image/strings/pstring.png)

We need a way to support values whose sizes vary, sometimes greatly.
This is exactly what dynamic allocation on the heap is designed for. We
can allocate as many bytes as we need. We get back a pointer that we’ll
use to keep track of the value as it flows through the VM.

## <a href="#values-and-objects" id="values-and-objects"><span
class="small">19 . 1</span>Values and Objects</a>

Using the heap for larger, variable-sized values and the stack for
smaller, atomic ones leads to a two-level representation. Every Lox
value that you can store in a variable or return from an expression will
be a Value. For small, fixed-size types like numbers, the payload is
stored directly inside the Value struct itself.

If the object is larger, its data lives on the heap. Then the Value’s
payload is a *pointer* to that blob of memory. We’ll eventually have a
handful of heap-allocated types in clox: strings, instances, functions,
you get the idea. Each type has its own unique data, but there is also
state they all share that [our future garbage
collector](garbage-collection.html) will use to manage their memory.

<img src="image/strings/value.png" class="wide"
alt="Field layout of number and obj values." />

We’ll call this common representation <span id="short">“Obj”</span>.
Each Lox value whose state lives on the heap is an Obj. We can thus use
a single new ValueType case to refer to all heap-allocated types.

“Obj” is short for “object”, natch.

<div class="codehilite">

``` insert-before
  VAL_NUMBER,
```

<div class="source-file">

*value.h*  
in enum *ValueType*

</div>

``` insert
  VAL_OBJ
```

``` insert-after
} ValueType;
```

</div>

<div class="source-file-narrow">

*value.h*, in enum *ValueType*

</div>

When a Value’s type is `VAL_OBJ`, the payload is a pointer to the heap
memory, so we add another case to the union for that.

<div class="codehilite">

``` insert-before
    double number;
```

<div class="source-file">

*value.h*  
in struct *Value*

</div>

``` insert
    Obj* obj;
```

``` insert-after
  } as; 
```

</div>

<div class="source-file-narrow">

*value.h*, in struct *Value*

</div>

As we did with the other value types, we crank out a couple of helpful
macros for working with Obj values.

<div class="codehilite">

``` insert-before
#define IS_NUMBER(value)  ((value).type == VAL_NUMBER)
```

<div class="source-file">

*value.h*  
add after struct *Value*

</div>

``` insert
#define IS_OBJ(value)     ((value).type == VAL_OBJ)
```

``` insert-after

#define AS_BOOL(value)    ((value).as.boolean)
```

</div>

<div class="source-file-narrow">

*value.h*, add after struct *Value*

</div>

This evaluates to `true` if the given Value is an Obj. If so, we can use
this:

<div class="codehilite">

``` insert-before
#define IS_OBJ(value)     ((value).type == VAL_OBJ)
```

<div class="source-file">

*value.h*

</div>

``` insert
#define AS_OBJ(value)     ((value).as.obj)
```

``` insert-after
#define AS_BOOL(value)    ((value).as.boolean)
```

</div>

<div class="source-file-narrow">

*value.h*

</div>

It extracts the Obj pointer from the value. We can also go the other
way.

<div class="codehilite">

``` insert-before
#define NUMBER_VAL(value) ((Value){VAL_NUMBER, {.number = value}})
```

<div class="source-file">

*value.h*

</div>

``` insert
#define OBJ_VAL(object)   ((Value){VAL_OBJ, {.obj = (Obj*)object}})
```

``` insert-after

typedef struct {
```

</div>

<div class="source-file-narrow">

*value.h*

</div>

This takes a bare Obj pointer and wraps it in a full Value.

## <a href="#struct-inheritance" id="struct-inheritance"><span
class="small">19 . 2</span>Struct Inheritance</a>

Every heap-allocated value is an Obj, but <span id="objs">Objs</span>
are not all the same. For strings, we need the array of characters. When
we get to instances, they will need their data fields. A function object
will need its chunk of bytecode. How do we handle different payloads and
sizes? We can’t use another union like we did for Value since the sizes
are all over the place.

No, I don’t know how to pronounce “objs” either. Feels like there should
be a vowel in there somewhere.

Instead, we’ll use another technique. It’s been around for ages, to the
point that the C specification carves out specific support for it, but I
don’t know that it has a canonical name. It’s an example of [*type
punning*](https://en.wikipedia.org/wiki/Type_punning), but that term is
too broad. In the absence of any better ideas, I’ll call it **struct
inheritance**, because it relies on structs and roughly follows how
single-inheritance of state works in object-oriented languages.

Like a tagged union, each Obj starts with a tag field that identifies
what kind of object it is<span class="em">—</span>string, instance, etc.
Following that are the payload fields. Instead of a union with cases for
each type, each type is its own separate struct. The tricky part is how
to treat these structs uniformly since C has no concept of inheritance
or polymorphism. I’ll explain that soon, but first lets get the
preliminary stuff out of the way.

The name “Obj” itself refers to a struct that contains the state shared
across all object types. It’s sort of like the “base class” for objects.
Because of some cyclic dependencies between values and objects, we
forward-declare it in the “value” module.

<div class="codehilite">

``` insert-before
#include "common.h"
```

<div class="source-file">

*value.h*

</div>

``` insert
typedef struct Obj Obj;
```

``` insert-after
typedef enum {
```

</div>

<div class="source-file-narrow">

*value.h*

</div>

And the actual definition is in a new module.

<div class="codehilite">

<div class="source-file">

*object.h*  
create new file

</div>

    #ifndef clox_object_h
    #define clox_object_h

    #include "common.h"
    #include "value.h"

    struct Obj {
      ObjType type;
    };

    #endif

</div>

<div class="source-file-narrow">

*object.h*, create new file

</div>

Right now, it contains only the type tag. Shortly, we’ll add some other
bookkeeping information for memory management. The type enum is this:

<div class="codehilite">

``` insert-before
#include "value.h"
```

<div class="source-file">

*object.h*

</div>

``` insert

typedef enum {
  OBJ_STRING,
} ObjType;
```

``` insert-after

struct Obj {
```

</div>

<div class="source-file-narrow">

*object.h*

</div>

Obviously, that will be more useful in later chapters after we add more
heap-allocated types. Since we’ll be accessing these tag types
frequently, it’s worth making a little macro that extracts the object
type tag from a given Value.

<div class="codehilite">

``` insert-before
#include "value.h"
```

<div class="source-file">

*object.h*

</div>

``` insert

#define OBJ_TYPE(value)        (AS_OBJ(value)->type)
```

``` insert-after

typedef enum {
```

</div>

<div class="source-file-narrow">

*object.h*

</div>

That’s our foundation.

Now, let’s build strings on top of it. The payload for strings is
defined in a separate struct. Again, we need to forward-declare it.

<div class="codehilite">

``` insert-before
typedef struct Obj Obj;
```

<div class="source-file">

*value.h*

</div>

``` insert
typedef struct ObjString ObjString;
```

``` insert-after

typedef enum {
```

</div>

<div class="source-file-narrow">

*value.h*

</div>

The definition lives alongside Obj.

<div class="codehilite">

``` insert-before
};
```

<div class="source-file">

*object.h*  
add after struct *Obj*

</div>

``` insert

struct ObjString {
  Obj obj;
  int length;
  char* chars;
};
```

``` insert-after

#endif
```

</div>

<div class="source-file-narrow">

*object.h*, add after struct *Obj*

</div>

A string object contains an array of characters. Those are stored in a
separate, heap-allocated array so that we set aside only as much room as
needed for each string. We also store the number of bytes in the array.
This isn’t strictly necessary but lets us tell how much memory is
allocated for the string without walking the character array to find the
null terminator.

Because ObjString is an Obj, it also needs the state all Objs share. It
accomplishes that by having its first field be an Obj. C specifies that
struct fields are arranged in memory in the order that they are
declared. Also, when you nest structs, the inner struct’s fields are
expanded right in place. So the memory for Obj and for ObjString looks
like this:

![The memory layout for the fields in Obj and
ObjString.](image/strings/obj.png)

Note how the first bytes of ObjString exactly line up with Obj. This is
not a coincidence<span class="em">—</span>C
<span id="spec">mandates</span> it. This is designed to enable a clever
pattern: You can take a pointer to a struct and safely convert it to a
pointer to its first field and back.

The key part of the spec is:

> § 6.7.2.1 13
>
> Within a structure object, the non-bit-field members and the units in
> which bit-fields reside have addresses that increase in the order in
> which they are declared. A pointer to a structure object, suitably
> converted, points to its initial member (or if that member is a
> bit-field, then to the unit in which it resides), and vice versa.
> There may be unnamed padding within a structure object, but not at its
> beginning.

Given an `ObjString*`, you can safely cast it to `Obj*` and then access
the `type` field from it. Every ObjString “is” an Obj in the OOP sense
of “is”. When we later add other object types, each struct will have an
Obj as its first field. Any code that wants to work with all objects can
treat them as base `Obj*` and ignore any other fields that may happen to
follow.

You can go in the other direction too. Given an `Obj*`, you can
“downcast” it to an `ObjString*`. Of course, you need to ensure that the
`Obj*` pointer you have does point to the `obj` field of an actual
ObjString. Otherwise, you are unsafely reinterpreting random bits of
memory. To detect that such a cast is safe, we add another macro.

<div class="codehilite">

``` insert-before
#define OBJ_TYPE(value)        (AS_OBJ(value)->type)
```

<div class="source-file">

*object.h*

</div>

``` insert

#define IS_STRING(value)       isObjType(value, OBJ_STRING)
```

``` insert-after

typedef enum {
```

</div>

<div class="source-file-narrow">

*object.h*

</div>

It takes a Value, not a raw `Obj*` because most code in the VM works
with Values. It relies on this inline function:

<div class="codehilite">

``` insert-before
};
```

<div class="source-file">

*object.h*  
add after struct *ObjString*

</div>

``` insert
static inline bool isObjType(Value value, ObjType type) {
  return IS_OBJ(value) && AS_OBJ(value)->type == type;
}
```

``` insert-after
#endif
```

</div>

<div class="source-file-narrow">

*object.h*, add after struct *ObjString*

</div>

Pop quiz: Why not just put the body of this function right in the macro?
What’s different about this one compared to the others? Right, it’s
because the body uses `value` twice. A macro is expanded by inserting
the argument *expression* every place the parameter name appears in the
body. If a macro uses a parameter more than once, that expression gets
evaluated multiple times.

That’s bad if the expression has side effects. If we put the body of
`isObjType()` into the macro definition and then you did, say,

<div class="codehilite">

    IS_STRING(POP())

</div>

then it would pop two values off the stack! Using a function fixes that.

As long as we ensure that we set the type tag correctly whenever we
create an Obj of some type, this macro will tell us when it’s safe to
cast a value to a specific object type. We can do that using these:

<div class="codehilite">

``` insert-before
#define IS_STRING(value)       isObjType(value, OBJ_STRING)
```

<div class="source-file">

*object.h*

</div>

``` insert

#define AS_STRING(value)       ((ObjString*)AS_OBJ(value))
#define AS_CSTRING(value)      (((ObjString*)AS_OBJ(value))->chars)
```

``` insert-after

typedef enum {
```

</div>

<div class="source-file-narrow">

*object.h*

</div>

These two macros take a Value that is expected to contain a pointer to a
valid ObjString on the heap. The first one returns the `ObjString*`
pointer. The second one steps through that to return the character array
itself, since that’s often what we’ll end up needing.

## <a href="#strings" id="strings"><span
class="small">19 . 3</span>Strings</a>

OK, our VM can now represent string values. It’s time to add strings to
the language itself. As usual, we begin in the front end. The lexer
already tokenizes string literals, so it’s the parser’s turn.

<div class="codehilite">

``` insert-before
  [TOKEN_IDENTIFIER]    = {NULL,     NULL,   PREC_NONE},
```

<div class="source-file">

*compiler.c*  
replace 1 line

</div>

``` insert
  [TOKEN_STRING]        = {string,   NULL,   PREC_NONE},
```

``` insert-after
  [TOKEN_NUMBER]        = {number,   NULL,   PREC_NONE},
```

</div>

<div class="source-file-narrow">

*compiler.c*, replace 1 line

</div>

When the parser hits a string token, it calls this parse function:

<div class="codehilite">

<div class="source-file">

*compiler.c*  
add after *number*()

</div>

    static void string() {
      emitConstant(OBJ_VAL(copyString(parser.previous.start + 1,
                                      parser.previous.length - 2)));
    }

</div>

<div class="source-file-narrow">

*compiler.c*, add after *number*()

</div>

This takes the string’s characters <span id="escape">directly</span>
from the lexeme. The `+ 1` and `- 2` parts trim the leading and trailing
quotation marks. It then creates a string object, wraps it in a Value,
and stuffs it into the constant table.

If Lox supported string escape sequences like `\n`, we’d translate those
here. Since it doesn’t, we can take the characters as they are.

To create the string, we use `copyString()`, which is declared in
`object.h`.

<div class="codehilite">

``` insert-before
};
```

<div class="source-file">

*object.h*  
add after struct *ObjString*

</div>

``` insert
ObjString* copyString(const char* chars, int length);
```

``` insert-after
static inline bool isObjType(Value value, ObjType type) {
```

</div>

<div class="source-file-narrow">

*object.h*, add after struct *ObjString*

</div>

The compiler module needs to include that.

<div class="codehilite">

``` insert-before
#define clox_compiler_h
```

<div class="source-file">

*compiler.h*

</div>

``` insert
#include "object.h"
```

``` insert-after
#include "vm.h"
```

</div>

<div class="source-file-narrow">

*compiler.h*

</div>

Our “object” module gets an implementation file where we define the new
function.

<div class="codehilite">

<div class="source-file">

*object.c*  
create new file

</div>

    #include <stdio.h>
    #include <string.h>

    #include "memory.h"
    #include "object.h"
    #include "value.h"
    #include "vm.h"

    ObjString* copyString(const char* chars, int length) {
      char* heapChars = ALLOCATE(char, length + 1);
      memcpy(heapChars, chars, length);
      heapChars[length] = '\0';
      return allocateString(heapChars, length);
    }

</div>

<div class="source-file-narrow">

*object.c*, create new file

</div>

First, we allocate a new array on the heap, just big enough for the
string’s characters and the trailing
<span id="terminator">terminator</span>, using this low-level macro that
allocates an array with a given element type and count:

<div class="codehilite">

``` insert-before
#include "common.h"
```

<div class="source-file">

*memory.h*

</div>

``` insert
#define ALLOCATE(type, count) \
    (type*)reallocate(NULL, 0, sizeof(type) * (count))
```

``` insert-after
#define GROW_CAPACITY(capacity) \
```

</div>

<div class="source-file-narrow">

*memory.h*

</div>

Once we have the array, we copy over the characters from the lexeme and
terminate it.

We need to terminate the string ourselves because the lexeme points at a
range of characters inside the monolithic source string and isn’t
terminated.

Since ObjString stores the length explicitly, we *could* leave the
character array unterminated, but slapping a terminator on the end costs
us only a byte and lets us pass the character array to C standard
library functions that expect a terminated string.

You might wonder why the ObjString can’t just point back to the original
characters in the source string. Some ObjStrings will be created
dynamically at runtime as a result of string operations like
concatenation. Those strings obviously need to dynamically allocate
memory for the characters, which means the string needs to *free* that
memory when it’s no longer needed.

If we had an ObjString for a string literal, and tried to free its
character array that pointed into the original source code string, bad
things would happen. So, for literals, we preemptively copy the
characters over to the heap. This way, every ObjString reliably owns its
character array and can free it.

The real work of creating a string object happens in this function:

<div class="codehilite">

``` insert-before
#include "vm.h"
```

<div class="source-file">

*object.c*

</div>

``` insert
static ObjString* allocateString(char* chars, int length) {
  ObjString* string = ALLOCATE_OBJ(ObjString, OBJ_STRING);
  string->length = length;
  string->chars = chars;
  return string;
}
```

</div>

<div class="source-file-narrow">

*object.c*

</div>

It creates a new ObjString on the heap and then initializes its fields.
It’s sort of like a constructor in an OOP language. As such, it first
calls the “base class” constructor to initialize the Obj state, using a
new macro.

<div class="codehilite">

``` insert-before
#include "vm.h"
```

<div class="source-file">

*object.c*

</div>

``` insert

#define ALLOCATE_OBJ(type, objectType) \
    (type*)allocateObject(sizeof(type), objectType)
```

``` insert-after

static ObjString* allocateString(char* chars, int length) {
```

</div>

<div class="source-file-narrow">

*object.c*

</div>

<span id="factored">Like</span> the previous macro, this exists mainly
to avoid the need to redundantly cast a `void*` back to the desired
type. The actual functionality is here:

I admit this chapter has a sea of helper functions and macros to wade
through. I try to keep the code nicely factored, but that leads to a
scattering of tiny functions. They will pay off when we reuse them
later.

<div class="codehilite">

``` insert-before
#define ALLOCATE_OBJ(type, objectType) \
    (type*)allocateObject(sizeof(type), objectType)
```

<div class="source-file">

*object.c*

</div>

``` insert

static Obj* allocateObject(size_t size, ObjType type) {
  Obj* object = (Obj*)reallocate(NULL, 0, size);
  object->type = type;
  return object;
}
```

``` insert-after

static ObjString* allocateString(char* chars, int length) {
```

</div>

<div class="source-file-narrow">

*object.c*

</div>

It allocates an object of the given size on the heap. Note that the size
is *not* just the size of Obj itself. The caller passes in the number of
bytes so that there is room for the extra payload fields needed by the
specific object type being created.

Then it initializes the Obj state<span class="em">—</span>right now,
that’s just the type tag. This function returns to `allocateString()`,
which finishes initializing the ObjString fields.
<span id="viola">*Voilà*</span>, we can compile and execute string
literals.

<img src="image/strings/viola.png" class="above" alt="A viola." />

Don’t get “voilà” confused with “viola”. One means “there it is” and the
other is a string instrument, the middle child between a violin and a
cello. Yes, I did spend two hours drawing a viola just to mention that.

## <a href="#operations-on-strings" id="operations-on-strings"><span
class="small">19 . 4</span>Operations on Strings</a>

Our fancy strings are there, but they don’t do much of anything yet. A
good first step is to make the existing print code not barf on the new
value type.

<div class="codehilite">

``` insert-before
    case VAL_NUMBER: printf("%g", AS_NUMBER(value)); break;
```

<div class="source-file">

*value.c*  
in *printValue*()

</div>

``` insert
    case VAL_OBJ: printObject(value); break;
```

``` insert-after
  }
```

</div>

<div class="source-file-narrow">

*value.c*, in *printValue*()

</div>

If the value is a heap-allocated object, it defers to a helper function
over in the “object” module.

<div class="codehilite">

``` insert-before
ObjString* copyString(const char* chars, int length);
```

<div class="source-file">

*object.h*  
add after *copyString*()

</div>

``` insert
void printObject(Value value);
```

``` insert-after

static inline bool isObjType(Value value, ObjType type) {
```

</div>

<div class="source-file-narrow">

*object.h*, add after *copyString*()

</div>

The implementation looks like this:

<div class="codehilite">

<div class="source-file">

*object.c*  
add after *copyString*()

</div>

    void printObject(Value value) {
      switch (OBJ_TYPE(value)) {
        case OBJ_STRING:
          printf("%s", AS_CSTRING(value));
          break;
      }
    }

</div>

<div class="source-file-narrow">

*object.c*, add after *copyString*()

</div>

We have only a single object type now, but this function will sprout
additional switch cases in later chapters. For string objects, it simply
<span id="term-2">prints</span> the character array as a C string.

I told you terminating the string would come in handy.

The equality operators also need to gracefully handle strings. Consider:

<div class="codehilite">

    "string" == "string"

</div>

These are two separate string literals. The compiler will make two
separate calls to `copyString()`, create two distinct ObjString objects
and store them as two constants in the chunk. They are different objects
in the heap. But our users (and thus we) expect strings to have value
equality. The above expression should evaluate to `true`. That requires
a little special support.

<div class="codehilite">

``` insert-before
    case VAL_NUMBER: return AS_NUMBER(a) == AS_NUMBER(b);
```

<div class="source-file">

*value.c*  
in *valuesEqual*()

</div>

``` insert
    case VAL_OBJ: {
      ObjString* aString = AS_STRING(a);
      ObjString* bString = AS_STRING(b);
      return aString->length == bString->length &&
          memcmp(aString->chars, bString->chars,
                 aString->length) == 0;
    }
```

``` insert-after
    default:         return false; // Unreachable.
```

</div>

<div class="source-file-narrow">

*value.c*, in *valuesEqual*()

</div>

If the two values are both strings, then they are equal if their
character arrays contain the same characters, regardless of whether they
are two separate objects or the exact same one. This does mean that
string equality is slower than equality on other types since it has to
walk the whole string. We’ll revise that [later](hash-tables.html), but
this gives us the right semantics for now.

Finally, in order to use `memcmp()` and the new stuff in the “object”
module, we need a couple of includes. Here:

<div class="codehilite">

``` insert-before
#include <stdio.h>
```

<div class="source-file">

*value.c*

</div>

``` insert
#include <string.h>
```

``` insert-after

#include "memory.h"
```

</div>

<div class="source-file-narrow">

*value.c*

</div>

And here:

<div class="codehilite">

``` insert-before
#include <string.h>
```

<div class="source-file">

*value.c*

</div>

``` insert
#include "object.h"
```

``` insert-after
#include "memory.h"
```

</div>

<div class="source-file-narrow">

*value.c*

</div>

### <a href="#concatenation" id="concatenation"><span
class="small">19 . 4 . 1</span>Concatenation</a>

Full-grown languages provide lots of operations for working with
strings<span class="em">—</span>access to individual characters, the
string’s length, changing case, splitting, joining, searching, etc. When
you implement your language, you’ll likely want all that. But for this
book, we keep things *very* minimal.

The only interesting operation we support on strings is `+`. If you use
that operator on two string objects, it produces a new string that’s a
concatenation of the two operands. Since Lox is dynamically typed, we
can’t tell which behavior is needed at compile time because we don’t
know the types of the operands until runtime. Thus, the `OP_ADD`
instruction dynamically inspects the operands and chooses the right
operation.

<div class="codehilite">

``` insert-before
      case OP_LESS:     BINARY_OP(BOOL_VAL, <); break;
```

<div class="source-file">

*vm.c*  
in *run*()  
replace 1 line

</div>

``` insert
      case OP_ADD: {
        if (IS_STRING(peek(0)) && IS_STRING(peek(1))) {
          concatenate();
        } else if (IS_NUMBER(peek(0)) && IS_NUMBER(peek(1))) {
          double b = AS_NUMBER(pop());
          double a = AS_NUMBER(pop());
          push(NUMBER_VAL(a + b));
        } else {
          runtimeError(
              "Operands must be two numbers or two strings.");
          return INTERPRET_RUNTIME_ERROR;
        }
        break;
      }
```

``` insert-after
      case OP_SUBTRACT: BINARY_OP(NUMBER_VAL, -); break;
```

</div>

<div class="source-file-narrow">

*vm.c*, in *run*(), replace 1 line

</div>

If both operands are strings, it concatenates. If they’re both numbers,
it adds them. Any other <span id="convert">combination</span> of operand
types is a runtime error.

This is more conservative than most languages. In other languages, if
one operand is a string, the other can be any type and it will be
implicitly converted to a string before concatenating the two.

I think that’s a fine feature, but would require writing tedious
“convert to string” code for each type, so I left it out of Lox.

To concatenate strings, we define a new function.

<div class="codehilite">

<div class="source-file">

*vm.c*  
add after *isFalsey*()

</div>

    static void concatenate() {
      ObjString* b = AS_STRING(pop());
      ObjString* a = AS_STRING(pop());

      int length = a->length + b->length;
      char* chars = ALLOCATE(char, length + 1);
      memcpy(chars, a->chars, a->length);
      memcpy(chars + a->length, b->chars, b->length);
      chars[length] = '\0';

      ObjString* result = takeString(chars, length);
      push(OBJ_VAL(result));
    }

</div>

<div class="source-file-narrow">

*vm.c*, add after *isFalsey*()

</div>

It’s pretty verbose, as C code that works with strings tends to be.
First, we calculate the length of the result string based on the lengths
of the operands. We allocate a character array for the result and then
copy the two halves in. As always, we carefully ensure the string is
terminated.

In order to call `memcpy()`, the VM needs an include.

<div class="codehilite">

``` insert-before
#include <stdio.h>
```

<div class="source-file">

*vm.c*

</div>

``` insert
#include <string.h>
```

``` insert-after

#include "common.h"
```

</div>

<div class="source-file-narrow">

*vm.c*

</div>

Finally, we produce an ObjString to contain those characters. This time
we use a new function, `takeString()`.

<div class="codehilite">

``` insert-before
};
```

<div class="source-file">

*object.h*  
add after struct *ObjString*

</div>

``` insert
ObjString* takeString(char* chars, int length);
```

``` insert-after
ObjString* copyString(const char* chars, int length);
```

</div>

<div class="source-file-narrow">

*object.h*, add after struct *ObjString*

</div>

The implementation looks like this:

<div class="codehilite">

<div class="source-file">

*object.c*  
add after *allocateString*()

</div>

    ObjString* takeString(char* chars, int length) {
      return allocateString(chars, length);
    }

</div>

<div class="source-file-narrow">

*object.c*, add after *allocateString*()

</div>

The previous `copyString()` function assumes it *cannot* take ownership
of the characters you pass in. Instead, it conservatively creates a copy
of the characters on the heap that the ObjString can own. That’s the
right thing for string literals where the passed-in characters are in
the middle of the source string.

But, for concatenation, we’ve already dynamically allocated a character
array on the heap. Making another copy of that would be redundant (and
would mean `concatenate()` has to remember to free its copy). Instead,
this function claims ownership of the string you give it.

As usual, stitching this functionality together requires a couple of
includes.

<div class="codehilite">

``` insert-before
#include "debug.h"
```

<div class="source-file">

*vm.c*

</div>

``` insert
#include "object.h"
#include "memory.h"
```

``` insert-after
#include "vm.h"
```

</div>

<div class="source-file-narrow">

*vm.c*

</div>

## <a href="#freeing-objects" id="freeing-objects"><span
class="small">19 . 5</span>Freeing Objects</a>

Behold this innocuous-seeming expression:

<div class="codehilite">

    "st" + "ri" + "ng"

</div>

When the compiler chews through this, it allocates an ObjString for each
of those three string literals and stores them in the chunk’s constant
table and generates this <span id="stack">bytecode</span>:

Here’s what the stack looks like after each instruction:

![The state of the stack at each instruction.](image/strings/stack.png)

<div class="codehilite">

    0000    OP_CONSTANT         0 "st"
    0002    OP_CONSTANT         1 "ri"
    0004    OP_ADD
    0005    OP_CONSTANT         2 "ng"
    0007    OP_ADD
    0008    OP_RETURN

</div>

The first two instructions push `"st"` and `"ri"` onto the stack. Then
the `OP_ADD` pops those and concatenates them. That dynamically
allocates a new `"stri"` string on the heap. The VM pushes that and then
pushes the `"ng"` constant. The last `OP_ADD` pops `"stri"` and `"ng"`,
concatenates them, and pushes the result: `"string"`. Great, that’s what
we expect.

But, wait. What happened to that `"stri"` string? We dynamically
allocated it, then the VM discarded it after concatenating it with
`"ng"`. We popped it from the stack and no longer have a reference to
it, but we never freed its memory. We’ve got ourselves a classic memory
leak.

Of course, it’s perfectly fine for the *Lox program* to forget about
intermediate strings and not worry about freeing them. Lox automatically
manages memory on the user’s behalf. The responsibility to manage memory
doesn’t *disappear*. Instead, it falls on our shoulders as VM
implementers.

The full <span id="borrowed">solution</span> is a [garbage
collector](garbage-collection.html) that reclaims unused memory while
the program is running. We’ve got some other stuff to get in place
before we’re ready to tackle that project. Until then, we are living on
borrowed time. The longer we wait to add the collector, the harder it is
to do.

I’ve seen a number of people implement large swathes of their language
before trying to start on the GC. For the kind of toy programs you
typically run while a language is being developed, you actually don’t
run out of memory before reaching the end of the program, so this gets
you surprisingly far.

But that underestimates how *hard* it is to add a garbage collector
later. The collector *must* ensure it can find every bit of memory that
*is* still being used so that it doesn’t collect live data. There are
hundreds of places a language implementation can squirrel away a
reference to some object. If you don’t find all of them, you get
nightmarish bugs.

I’ve seen language implementations die because it was too hard to get
the GC in later. If your language needs GC, get it working as soon as
you can. It’s a crosscutting concern that touches the entire codebase.

Today, we should at least do the bare minimum: avoid *leaking* memory by
making sure the VM can still find every allocated object even if the Lox
program itself no longer references them. There are many sophisticated
techniques that advanced memory managers use to allocate and track
memory for objects. We’re going to take the simplest practical approach.

We’ll create a linked list that stores every Obj. The VM can traverse
that list to find every single object that has been allocated on the
heap, whether or not the user’s program or the VM’s stack still has a
reference to it.

We could define a separate linked list node struct but then we’d have to
allocate those too. Instead, we’ll use an **intrusive
list**<span class="em">—</span>the Obj struct itself will be the linked
list node. Each Obj gets a pointer to the next Obj in the chain.

<div class="codehilite">

``` insert-before
struct Obj {
  ObjType type;
```

<div class="source-file">

*object.h*  
in struct *Obj*

</div>

``` insert
  struct Obj* next;
```

``` insert-after
};
```

</div>

<div class="source-file-narrow">

*object.h*, in struct *Obj*

</div>

The VM stores a pointer to the head of the list.

<div class="codehilite">

``` insert-before
  Value* stackTop;
```

<div class="source-file">

*vm.h*  
in struct *VM*

</div>

``` insert
  Obj* objects;
```

``` insert-after
} VM;
```

</div>

<div class="source-file-narrow">

*vm.h*, in struct *VM*

</div>

When we first initialize the VM, there are no allocated objects.

<div class="codehilite">

``` insert-before
  resetStack();
```

<div class="source-file">

*vm.c*  
in *initVM*()

</div>

``` insert
  vm.objects = NULL;
```

``` insert-after
}
```

</div>

<div class="source-file-narrow">

*vm.c*, in *initVM*()

</div>

Every time we allocate an Obj, we insert it in the list.

<div class="codehilite">

``` insert-before
  object->type = type;
```

<div class="source-file">

*object.c*  
in *allocateObject*()

</div>

``` insert

  object->next = vm.objects;
  vm.objects = object;
```

``` insert-after
  return object;
```

</div>

<div class="source-file-narrow">

*object.c*, in *allocateObject*()

</div>

Since this is a singly linked list, the easiest place to insert it is as
the head. That way, we don’t need to also store a pointer to the tail
and keep it updated.

The “object” module is directly using the global `vm` variable from the
“vm” module, so we need to expose that externally.

<div class="codehilite">

``` insert-before
} InterpretResult;
```

<div class="source-file">

*vm.h*  
add after enum *InterpretResult*

</div>

``` insert
extern VM vm;
```

``` insert-after
void initVM();
```

</div>

<div class="source-file-narrow">

*vm.h*, add after enum *InterpretResult*

</div>

Eventually, the garbage collector will free memory while the VM is still
running. But, even then, there will usually be unused objects still
lingering in memory when the user’s program completes. The VM should
free those too.

There’s no sophisticated logic for that. Once the program is done, we
can free *every* object. We can and should implement that now.

<div class="codehilite">

``` insert-before
void freeVM() {
```

<div class="source-file">

*vm.c*  
in *freeVM*()

</div>

``` insert
  freeObjects();
```

``` insert-after
}
```

</div>

<div class="source-file-narrow">

*vm.c*, in *freeVM*()

</div>

That empty function we defined [way back
when](a-virtual-machine.html#an-instruction-execution-machine) finally
does something! It calls this:

<div class="codehilite">

``` insert-before
void* reallocate(void* pointer, size_t oldSize, size_t newSize);
```

<div class="source-file">

*memory.h*  
add after *reallocate*()

</div>

``` insert
void freeObjects();
```

``` insert-after

#endif
```

</div>

<div class="source-file-narrow">

*memory.h*, add after *reallocate*()

</div>

Here’s how we free the objects:

<div class="codehilite">

<div class="source-file">

*memory.c*  
add after *reallocate*()

</div>

    void freeObjects() {
      Obj* object = vm.objects;
      while (object != NULL) {
        Obj* next = object->next;
        freeObject(object);
        object = next;
      }
    }

</div>

<div class="source-file-narrow">

*memory.c*, add after *reallocate*()

</div>

This is a CS 101 textbook implementation of walking a linked list and
freeing its nodes. For each node, we call:

<div class="codehilite">

<div class="source-file">

*memory.c*  
add after *reallocate*()

</div>

    static void freeObject(Obj* object) {
      switch (object->type) {
        case OBJ_STRING: {
          ObjString* string = (ObjString*)object;
          FREE_ARRAY(char, string->chars, string->length + 1);
          FREE(ObjString, object);
          break;
        }
      }
    }

</div>

<div class="source-file-narrow">

*memory.c*, add after *reallocate*()

</div>

We aren’t only freeing the Obj itself. Since some object types also
allocate other memory that they own, we also need a little type-specific
code to handle each object type’s special needs. Here, that means we
free the character array and then free the ObjString. Those both use one
last memory management macro.

<div class="codehilite">

``` insert-before
    (type*)reallocate(NULL, 0, sizeof(type) * (count))
```

<div class="source-file">

*memory.h*

</div>

``` insert

#define FREE(type, pointer) reallocate(pointer, sizeof(type), 0)
```

``` insert-after

#define GROW_CAPACITY(capacity) \
```

</div>

<div class="source-file-narrow">

*memory.h*

</div>

It’s a tiny <span id="free">wrapper</span> around `reallocate()` that
“resizes” an allocation down to zero bytes.

Using `reallocate()` to free memory might seem pointless. Why not just
call `free()`? Later, this will help the VM track how much memory is
still being used. If all allocation and freeing goes through
`reallocate()`, it’s easy to keep a running count of the number of bytes
of allocated memory.

As usual, we need an include to wire everything together.

<div class="codehilite">

``` insert-before
#include "common.h"
```

<div class="source-file">

*memory.h*

</div>

``` insert
#include "object.h"
```

``` insert-after

#define ALLOCATE(type, count) \
```

</div>

<div class="source-file-narrow">

*memory.h*

</div>

Then in the implementation file:

<div class="codehilite">

``` insert-before
#include "memory.h"
```

<div class="source-file">

*memory.c*

</div>

``` insert
#include "vm.h"
```

``` insert-after

void* reallocate(void* pointer, size_t oldSize, size_t newSize) {
```

</div>

<div class="source-file-narrow">

*memory.c*

</div>

With this, our VM no longer leaks memory. Like a good C program, it
cleans up its mess before exiting. But it doesn’t free any objects while
the VM is running. Later, when it’s possible to write longer-running Lox
programs, the VM will eat more and more memory as it goes, not
relinquishing a single byte until the entire program is done.

We won’t address that until we’ve added [a real garbage
collector](garbage-collection.html), but this is a big step. We now have
the infrastructure to support a variety of different kinds of
dynamically allocated objects. And we’ve used that to add strings to
clox, one of the most used types in most programming languages. Strings
in turn enable us to build another fundamental data type, especially in
dynamic languages: the venerable [hash table](hash-tables.html). But
that’s for the next chapter<span class="ellipse"> . . . </span>

<div class="challenges">

## <a href="#challenges" id="challenges">Challenges</a>

1.  Each string requires two separate dynamic
    allocations<span class="em">—</span>one for the ObjString and a
    second for the character array. Accessing the characters from a
    value requires two pointer indirections, which can be bad for
    performance. A more efficient solution relies on a technique called
    **[flexible array
    members](https://en.wikipedia.org/wiki/Flexible_array_member)**. Use
    that to store the ObjString and its character array in a single
    contiguous allocation.

2.  When we create the ObjString for each string literal, we copy the
    characters onto the heap. That way, when the string is later freed,
    we know it is safe to free the characters too.

    This is a simpler approach but wastes some memory, which might be a
    problem on very constrained devices. Instead, we could keep track of
    which ObjStrings own their character array and which are “constant
    strings” that just point back to the original source string or some
    other non-freeable location. Add support for this.

3.  If Lox was your language, what would you have it do when a user
    tries to use `+` with one string operand and the other some other
    type? Justify your choice. What do other languages do?

</div>

<div class="design-note">

## <a href="#design-note" id="design-note">Design Note: String Encoding</a>

In this book, I try not to shy away from the gnarly problems you’ll run
into in a real language implementation. We might not always use the most
*sophisticated* solution<span class="em">—</span>it’s an intro book
after all<span class="em">—</span>but I don’t think it’s honest to
pretend the problem doesn’t exist at all. However, I did skirt around
one really nasty conundrum: deciding how to represent strings.

There are two facets to a string encoding:

- **What is a single “character” in a string?** How many different
  values are there and what do they represent? The first widely adopted
  standard answer to this was
  [ASCII](https://en.wikipedia.org/wiki/ASCII). It gave you 127
  different character values and specified what they were. It was
  great<span class="ellipse"> . . . </span>if you only ever cared about
  English. While it has weird, mostly forgotten characters like “record
  separator” and “synchronous idle”, it doesn’t have a single umlaut,
  acute, or grave. It can’t represent “jalapeño”, “naïve”,
  <span id="gruyere">“Gruyère”</span>, or “Mötley Crüe”.

  It goes without saying that a language that does not let one discuss
  Gruyère or Mötley Crüe is a language not worth using.

  Next came [Unicode](https://en.wikipedia.org/wiki/Unicode). Initially,
  it supported 16,384 different characters (**code points**), which fit
  nicely in 16 bits with a couple of bits to spare. Later that grew and
  grew, and now there are well over 100,000 different code points
  including such vital instruments of human communication as 💩 (Unicode
  Character ‘PILE OF POO’, `U+1F4A9`).

  Even that long list of code points is not enough to represent each
  possible visible glyph a language might support. To handle that,
  Unicode also has **combining characters** that modify a preceding code
  point. For example, “a” followed by the combining character “¨” gives
  you “ä”. (To make things more confusing Unicode *also* has a single
  code point that looks like “ä”.)

  If a user accesses the fourth “character” in “naïve”, do they expect
  to get back “v” or “¨”? The former means they are thinking of each
  code point and its combining character as a single
  unit<span class="em">—</span>what Unicode calls an **extended grapheme
  cluster**<span class="em">—</span>the latter means they are thinking
  in individual code points. Which do your users expect?

- **How is a single unit represented in memory?** Most systems using
  ASCII gave a single byte to each character and left the high bit
  unused. Unicode has a handful of common encodings. UTF-16 packs most
  code points into 16 bits. That was great when every code point fit in
  that size. When that overflowed, they added *surrogate pairs* that use
  multiple 16-bit code units to represent a single code point. UTF-32 is
  the next evolution of UTF-16<span class="em">—</span>it gives a full
  32 bits to each and every code point.

  UTF-8 is more complex than either of those. It uses a variable number
  of bytes to encode a code point. Lower-valued code points fit in fewer
  bytes. Since each character may occupy a different number of bytes,
  you can’t directly index into the string to find a specific code
  point. If you want, say, the 10th code point, you don’t know how many
  bytes into the string that is without walking and decoding all of the
  preceding ones.

Choosing a character representation and encoding involves fundamental
trade-offs. Like many things in engineering, there’s no
<span id="python">perfect</span> solution:

An example of how difficult this problem is comes from Python. The
achingly long transition from Python 2 to 3 is painful mostly because of
its changes around string encoding.

- ASCII is memory efficient and fast, but it kicks non-Latin languages
  to the side.

- UTF-32 is fast and supports the whole Unicode range, but wastes a lot
  of memory given that most code points do tend to be in the lower range
  of values, where a full 32 bits aren’t needed.

- UTF-8 is memory efficient and supports the whole Unicode range, but
  its variable-length encoding makes it slow to access arbitrary code
  points.

- UTF-16 is worse than all of them<span class="em">—</span>an ugly
  consequence of Unicode outgrowing its earlier 16-bit range. It’s less
  memory efficient than UTF-8 but is still a variable-length encoding
  thanks to surrogate pairs. Avoid it if you can. Alas, if your language
  needs to run on or interoperate with the browser, the JVM, or the CLR,
  you might be stuck with it, since those all use UTF-16 for their
  strings and you don’t want to have to convert every time you pass a
  string to the underlying system.

One option is to take the maximal approach and do the “rightest” thing.
Support all the Unicode code points. Internally, select an encoding for
each string based on its contents<span class="em">—</span>use ASCII if
every code point fits in a byte, UTF-16 if there are no surrogate pairs,
etc. Provide APIs to let users iterate over both code points and
extended grapheme clusters.

This covers all your bases but is really complex. It’s a lot to
implement, debug, and optimize. When serializing strings or
interoperating with other systems, you have to deal with all of the
encodings. Users need to understand the two indexing APIs and know which
to use when. This is the approach that newer, big languages tend to
take<span class="em">—</span>like Raku and Swift.

A simpler compromise is to always encode using UTF-8 and only expose an
API that works with code points. For users that want to work with
grapheme clusters, let them use a third-party library for that. This is
less Latin-centric than ASCII but not much more complex. You lose fast
direct indexing by code point, but you can usually live without that or
afford to make it *O(n)* instead of *O(1)*.

If I were designing a big workhorse language for people writing large
applications, I’d probably go with the maximal approach. For my little
embedded scripting language [Wren](http://wren.io), I went with UTF-8
and code points.

</div>

<a href="hash-tables.html" class="next">Next Chapter: “Hash Tables”
→</a> Handcrafted by Robert Nystrom — <a
href="https://github.com/munificent/craftinginterpreters/blob/master/LICENSE"
target="_blank">© 2015 – 2021</a>

</div>

[![](image/logotype.png "Crafting Interpreters")](/)

<div class="contents">

### [Hash Tables<span class="small">20</span>](#top)

- [<span class="small">20.1</span> An Array of
  Buckets](#an-array-of-buckets)
- [<span class="small">20.2</span> Collision
  Resolution](#collision-resolution)
- [<span class="small">20.3</span> Hash Functions](#hash-functions)
- [<span class="small">20.4</span> Building a Hash
  Table](#building-a-hash-table)
- [<span class="small">20.5</span> String Interning](#string-interning)
- 
- [Challenges](#challenges)

<div class="prev-next">

<a href="strings.html" class="left" title="Strings">← Previous</a>
[↑ Up](a-bytecode-virtual-machine.html "A Bytecode Virtual Machine")
<a href="global-variables.html" class="right"
title="Global Variables">Next →</a>

</div>

</div>

[![](image/logotype.png "Crafting Interpreters")](/)
<a href="strings.html" class="prev" title="Strings">←</a>
<a href="global-variables.html" class="next"
title="Global Variables">→</a>

<div class="page">

<div class="nav-wrapper">

[![](image/logotype.png "Crafting Interpreters")](/)

<div class="expandable">

### [Hash Tables<span class="small">20</span>](#top)

- [<span class="small">20.1</span> An Array of
  Buckets](#an-array-of-buckets)
- [<span class="small">20.2</span> Collision
  Resolution](#collision-resolution)
- [<span class="small">20.3</span> Hash Functions](#hash-functions)
- [<span class="small">20.4</span> Building a Hash
  Table](#building-a-hash-table)
- [<span class="small">20.5</span> String Interning](#string-interning)
- 
- [Challenges](#challenges)

<div class="prev-next">

<a href="strings.html" class="left" title="Strings">← Previous</a>
[↑ Up](a-bytecode-virtual-machine.html "A Bytecode Virtual Machine")
<a href="global-variables.html" class="right"
title="Global Variables">Next →</a>

</div>

</div>

<span id="expand-nav">≡</span>

</div>

<div class="number">

20

</div>

# Hash Tables

> Hash, x. There is no definition for this
> word<span class="em">—</span>nobody knows what hash is.
>
> Ambrose Bierce, *The Unabridged Devil’s Dictionary*

Before we can add variables to our burgeoning virtual machine, we need
some way to look up a value given a variable’s name. Later, when we add
classes, we’ll also need a way to store fields on instances. The perfect
data structure for these problems and others is a hash table.

You probably already know what a hash table is, even if you don’t know
it by that name. If you’re a Java programmer, you call them “HashMaps”.
C# and Python users call them “dictionaries”. In C++, it’s an “unordered
map”. “Objects” in JavaScript and “tables” in Lua are hash tables under
the hood, which is what gives them their flexibility.

A hash table, whatever your language calls it, associates a set of
**keys** with a set of **values**. Each key/value pair is an **entry**
in the table. Given a key, you can look up its corresponding value. You
can add new key/value pairs and remove entries by key. If you add a new
value for an existing key, it replaces the previous entry.

Hash tables appear in so many languages because they are incredibly
powerful. Much of this power comes from one metric: given a key, a hash
table returns the corresponding value in <span id="constant">constant
time</span>, *regardless of how many keys are in the hash table*.

More specifically, the *average-case* lookup time is constant.
Worst-case performance can be, well, worse. In practice, it’s easy to
avoid degenerate behavior and stay on the happy path.

That’s pretty remarkable when you think about it. Imagine you’ve got a
big stack of business cards and I ask you to find a certain person. The
bigger the pile is, the longer it will take. Even if the pile is nicely
sorted and you’ve got the manual dexterity to do a binary search by
hand, you’re still talking *O(log n)*. But with a
<span id="rolodex">hash table</span>, it takes the same time to find
that business card when the stack has ten cards as when it has a
million.

Stuff all those cards in a Rolodex<span class="em">—</span>does anyone
even remember those things anymore?<span class="em">—</span>with
dividers for each letter, and you improve your speed dramatically. As
we’ll see, that’s not too far from the trick a hash table uses.

## <a href="#an-array-of-buckets" id="an-array-of-buckets"><span
class="small">20 . 1</span>An Array of Buckets</a>

A complete, fast hash table has a couple of moving parts. I’ll introduce
them one at a time by working through a couple of toy problems and their
solutions. Eventually, we’ll build up to a data structure that can
associate any set of names with their values.

For now, imagine if Lox was a *lot* more restricted in variable names.
What if a variable’s name could only be a <span id="basic">single</span>
lowercase letter. How could we very efficiently represent a set of
variable names and their values?

This limitation isn’t *too* far-fetched. The initial versions of BASIC
out of Dartmouth allowed variable names to be only a single letter
followed by one optional digit.

With only 26 possible variables (27 if you consider underscore a
“letter”, I guess), the answer is easy. Declare a fixed-size array with
26 elements. We’ll follow tradition and call each element a **bucket**.
Each represents a variable with `a` starting at index zero. If there’s a
value in the array at some letter’s index, then that key is present with
that value. Otherwise, the bucket is empty and that key/value pair isn’t
in the data structure.

![A row of buckets, each labeled with a letter of the
alphabet.](image/hash-tables/bucket-array.png)

Memory usage is great<span class="em">—</span>just a single, reasonably
sized <span id="bucket">array</span>. There’s some waste from the empty
buckets, but it’s not huge. There’s no overhead for node pointers,
padding, or other stuff you’d get with something like a linked list or
tree.

Performance is even better. Given a variable
name<span class="em">—</span>its character<span class="em">—</span>you
can subtract the ASCII value of `a` and use the result to index directly
into the array. Then you can either look up the existing value or store
a new value directly into that slot. It doesn’t get much faster than
that.

This is sort of our Platonic ideal data structure. Lightning fast, dead
simple, and compact in memory. As we add support for more complex keys,
we’ll have to make some concessions, but this is what we’re aiming for.
Even once you add in hash functions, dynamic resizing, and collision
resolution, this is still the core of every hash table out
there<span class="em">—</span>a contiguous array of buckets that you
index directly into.

### <a href="#load-factor-and-wrapped-keys"
id="load-factor-and-wrapped-keys"><span
class="small">20 . 1 . 1</span>Load factor and wrapped keys</a>

Confining Lox to single-letter variables would make our job as
implementers easier, but it’s probably no fun programming in a language
that gives you only 26 storage locations. What if we loosened it a
little and allowed variables up to <span id="six">eight</span>
characters long?

Again, this restriction isn’t so crazy. Early linkers for C treated only
the first six characters of external identifiers as meaningful.
Everything after that was ignored. If you’ve ever wondered why the C
standard library is so enamored of
abbreviation<span class="em">—</span>looking at you,
`strncmp()`<span class="em">—</span>it turns out it wasn’t entirely
because of the small screens (or teletypes!) of the day.

That’s small enough that we can pack all eight characters into a 64-bit
integer and easily turn the string into a number. We can then use it as
an array index. Or, at least, we could if we could somehow allocate a
295,148 *petabyte* array. Memory’s gotten cheaper over time, but not
quite *that* cheap. Even if we could make an array that big, it would be
heinously wasteful. Almost every bucket would be empty unless users
started writing way bigger Lox programs than we’ve anticipated.

Even though our variable keys cover the full 64-bit numeric range, we
clearly don’t need an array that large. Instead, we allocate an array
with more than enough capacity for the entries we need, but not
unreasonably large. We map the full 64-bit keys down to that smaller
range by taking the value modulo the size of the array. Doing that
essentially folds the larger numeric range onto itself until it fits the
smaller range of array elements.

For example, say we want to store “bagel”. We allocate an array with
eight elements, plenty enough to store it and more later. We treat the
key string as a 64-bit integer. On a little-endian machine like Intel,
packing those characters into a 64-bit word puts the first letter, “b”
(ASCII value 98), in the least-significant byte. We take that integer
modulo the array size (<span id="power-of-two">8</span>) to fit it in
the bounds and get a bucket index, 2. Then we store the value there as
usual.

I’m using powers of two for the array sizes here, but they don’t need to
be. Some styles of hash tables work best with powers of two, including
the one we’ll build in this book. Others prefer prime number array sizes
or have other rules.

Using the array size as a modulus lets us map the key’s numeric range
down to fit an array of any size. We can thus control the number of
buckets independently of the key range. That solves our waste problem,
but introduces a new one. Any two variables whose key number has the
same remainder when divided by the array size will end up in the same
bucket. Keys can **collide**. For example, if we try to add “jam”, it
also ends up in bucket 2.

!['Bagel' and 'jam' both end up in bucket index
2.](image/hash-tables/collision.png)

We have some control over this by tuning the array size. The bigger the
array, the fewer the indexes that get mapped to the same bucket and the
fewer the collisions that are likely to occur. Hash table implementers
track this collision likelihood by measuring the table’s **load
factor**. It’s defined as the number of entries divided by the number of
buckets. So a hash table with five entries and an array of 16 elements
has a load factor of 0.3125. The higher the load factor, the greater the
chance of collisions.

One way we mitigate collisions is by resizing the array. Just like the
dynamic arrays we implemented earlier, we reallocate and grow the hash
table’s array as it fills up. Unlike a regular dynamic array, though, we
won’t wait until the array is *full*. Instead, we pick a desired load
factor and grow the array when it goes over that.

## <a href="#collision-resolution" id="collision-resolution"><span
class="small">20 . 2</span>Collision Resolution</a>

Even with a very low load factor, collisions can still occur. The
[*birthday paradox*](https://en.wikipedia.org/wiki/Birthday_problem)
tells us that as the number of entries in the hash table increases, the
chance of collision increases very quickly. We can pick a large array
size to reduce that, but it’s a losing game. Say we wanted to store a
hundred items in a hash table. To keep the chance of collision below a
still-pretty-high 10%, we need an array with at least 47,015 elements.
To get the chance below 1% requires an array with 492,555 elements, over
4,000 empty buckets for each one in use.

A low load factor can make collisions <span id="pigeon">rarer</span>,
but the [*pigeonhole
principle*](https://en.wikipedia.org/wiki/Pigeonhole_principle) tells us
we can never eliminate them entirely. If you’ve got five pet pigeons and
four holes to put them in, at least one hole is going to end up with
more than one pigeon. With 18,446,744,073,709,551,616 different variable
names, any reasonably sized array can potentially end up with multiple
keys in the same bucket.

Thus we still have to handle collisions gracefully when they occur.
Users don’t like it when their programming language can look up
variables correctly only *most* of the time.

Put these two funny-named mathematical rules together and you get this
observation: Take a birdhouse containing 365 pigeonholes, and use each
pigeon’s birthday to assign it to a pigeonhole. You’ll need only about
26 randomly chosen pigeons before you get a greater than 50% chance of
two pigeons in the same box.

![Two pigeons in the same hole.](image/hash-tables/pigeons.png)

### <a href="#separate-chaining" id="separate-chaining"><span
class="small">20 . 2 . 1</span>Separate chaining</a>

Techniques for resolving collisions fall into two broad categories. The
first is **separate chaining**. Instead of each bucket containing a
single entry, we let it contain a collection of them. In the classic
implementation, each bucket points to a linked list of entries. To look
up an entry, you find its bucket and then walk the list until you find
an entry with the matching key.

![An array with eight buckets. Bucket 2 links to a chain of two nodes.
Bucket 5 links to a single node.](image/hash-tables/chaining.png)

In catastrophically bad cases where every entry collides in the same
bucket, the data structure degrades into a single unsorted linked list
with *O(n)* lookup. In practice, it’s easy to avoid that by controlling
the load factor and how entries get scattered across buckets. In typical
separate-chained hash tables, it’s rare for a bucket to have more than
one or two entries.

Separate chaining is conceptually simple<span class="em">—</span>it’s
literally an array of linked lists. Most operations are straightforward
to implement, even deletion which, as we’ll see, can be a pain. But it’s
not a great fit for modern CPUs. It has a lot of overhead from pointers
and tends to scatter little linked list <span id="node">nodes</span>
around in memory which isn’t great for cache usage.

There are a few tricks to optimize this. Many implementations store the
first entry right in the bucket so that in the common case where there’s
only one, no extra pointer indirection is needed. You can also make each
linked list node store a few entries to reduce the pointer overhead.

### <a href="#open-addressing" id="open-addressing"><span
class="small">20 . 2 . 2</span>Open addressing</a>

The other technique is <span id="open">called</span> **open addressing**
or (confusingly) **closed hashing**. With this technique, all entries
live directly in the bucket array, with one entry per bucket. If two
entries collide in the same bucket, we find a different empty bucket to
use instead.

It’s called “open” addressing because the entry may end up at an address
(bucket) outside of its preferred one. It’s called “closed” hashing
because all of the entries stay inside the array of buckets.

Storing all entries in a single, big, contiguous array is great for
keeping the memory representation simple and fast. But it makes all of
the operations on the hash table more complex. When inserting an entry,
its bucket may be full, sending us to look at another bucket. That
bucket itself may be occupied and so on. This process of finding an
available bucket is called **probing**, and the order that you examine
buckets is a **probe sequence**.

There are a <span id="probe">number</span> of algorithms for determining
which buckets to probe and how to decide which entry goes in which
bucket. There’s been a ton of research here because even slight tweaks
can have a large performance impact. And, on a data structure as heavily
used as hash tables, that performance impact touches a very large number
of real-world programs across a range of hardware capabilities.

If you’d like to learn more (and you should, because some of these are
really cool), look into “double hashing”, “cuckoo hashing”, “Robin Hood
hashing”, and anything those lead you to.

As usual in this book, we’ll pick the simplest one that gets the job
done efficiently. That’s good old **linear probing**. When looking for
an entry, we look in the first bucket its key maps to. If it’s not in
there, we look in the very next element in the array, and so on. If we
reach the end, we wrap back around to the beginning.

The good thing about linear probing is that it’s cache friendly. Since
you walk the array directly in memory order, it keeps the CPU’s cache
lines full and happy. The bad thing is that it’s prone to
**clustering**. If you have a lot of entries with numerically similar
key values, you can end up with a lot of colliding, overflowing buckets
right next to each other.

Compared to separate chaining, open addressing can be harder to wrap
your head around. I think of open addressing as similar to separate
chaining except that the “list” of nodes is threaded through the bucket
array itself. Instead of storing the links between them in pointers, the
connections are calculated implicitly by the order that you look through
the buckets.

The tricky part is that more than one of these implicit lists may be
interleaved together. Let’s walk through an example that covers all the
interesting cases. We’ll ignore values for now and just worry about a
set of keys. We start with an empty array of 8 buckets.

<img src="image/hash-tables/insert-1.png" class="wide"
alt="An array with eight empty buckets." />

We decide to insert “bagel”. The first letter, “b” (ASCII value 98),
modulo the array size (8) puts it in bucket 2.

<img src="image/hash-tables/insert-2.png" class="wide"
alt="Bagel goes into bucket 2." />

Next, we insert “jam”. That also wants to go in bucket 2 (106 mod 8 =
2), but that bucket’s taken. We keep probing to the next bucket. It’s
empty, so we put it there.

<img src="image/hash-tables/insert-3.png" class="wide"
alt="Jam goes into bucket 3, since 2 is full." />

We insert “fruit”, which happily lands in bucket 6.

<img src="image/hash-tables/insert-4.png" class="wide"
alt="Fruit goes into bucket 6." />

Likewise, “migas” can go in its preferred bucket 5.

<img src="image/hash-tables/insert-5.png" class="wide"
alt="Migas goes into bucket 5." />

When we try to insert “eggs”, it also wants to be in bucket 5. That’s
full, so we skip to 6. Bucket 6 is also full. Note that the entry in
there is *not* part of the same probe sequence. “Fruit” is in its
preferred bucket, 6. So the 5 and 6 sequences have collided and are
interleaved. We skip over that and finally put “eggs” in bucket 7.

<img src="image/hash-tables/insert-6.png" class="wide"
alt="Eggs goes into bucket 7 because 5 and 6 are full." />

We run into a similar problem with “nuts”. It can’t land in 6 like it
wants to. Nor can it go into 7. So we keep going. But we’ve reached the
end of the array, so we wrap back around to 0 and put it there.

<img src="image/hash-tables/insert-7.png" class="wide"
alt="Nuts wraps around to bucket 0 because 6 and 7 are full." />

In practice, the interleaving turns out to not be much of a problem.
Even in separate chaining, we need to walk the list to check each
entry’s key because multiple keys can reduce to the same bucket. With
open addressing, we need to do that same check, and that also covers the
case where you are stepping over entries that “belong” to a different
original bucket.

## <a href="#hash-functions" id="hash-functions"><span
class="small">20 . 3</span>Hash Functions</a>

We can now build ourselves a reasonably efficient table for storing
variable names up to eight characters long, but that limitation is still
annoying. In order to relax the last constraint, we need a way to take a
string of any length and convert it to a fixed-size integer.

Finally, we get to the “hash” part of “hash table”. A **hash function**
takes some larger blob of data and “hashes” it to produce a fixed-size
integer **hash code** whose value depends on all of the bits of the
original data. A <span id="crypto">good</span> hash function has three
main goals:

Hash functions are also used for cryptography. In that domain, “good”
has a *much* more stringent definition to avoid exposing details about
the data being hashed. We, thankfully, don’t need to worry about those
concerns for this book.

- **It must be *deterministic*.** The same input must always hash to the
  same number. If the same variable ends up in different buckets at
  different points in time, it’s gonna get really hard to find it.

- **It must be *uniform*.** Given a typical set of inputs, it should
  produce a wide and evenly distributed range of output numbers, with as
  few clumps or patterns as possible. We want it to
  <span id="scatter">scatter</span> values across the whole numeric
  range to minimize collisions and clustering.

- **It must be *fast*.** Every operation on the hash table requires us
  to hash the key first. If hashing is slow, it can potentially cancel
  out the speed of the underlying array storage.

One of the original names for a hash table was “scatter table” because
it takes the entries and scatters them throughout the array. The word
“hash” came from the idea that a hash function takes the input data,
chops it up, and tosses it all together into a pile to come up with a
single number from all of those bits.

There is a veritable pile of hash functions out there. Some are old and
optimized for architectures no one uses anymore. Some are designed to be
fast, others cryptographically secure. Some take advantage of vector
instructions and cache sizes for specific chips, others aim to maximize
portability.

There are people out there for whom designing and evaluating hash
functions is, like, their *jam*. I admire them, but I’m not
mathematically astute enough to *be* one. So for clox, I picked a
simple, well-worn hash function called
[FNV-1a](http://www.isthe.com/chongo/tech/comp/fnv/) that’s served me
fine over the years. Consider <span id="thing">trying</span> out
different ones in your code and see if they make a difference.

Who knows, maybe hash functions could turn out to be your thing too?

OK, that’s a quick run through of buckets, load factors, open
addressing, collision resolution, and hash functions. That’s an awful
lot of text and not a lot of real code. Don’t worry if it still seems
vague. Once we’re done coding it up, it will all click into place.

## <a href="#building-a-hash-table" id="building-a-hash-table"><span
class="small">20 . 4</span>Building a Hash Table</a>

The great thing about hash tables compared to other classic techniques
like balanced search trees is that the actual data structure is so
simple. Ours goes into a new module.

<div class="codehilite">

<div class="source-file">

*table.h*  
create new file

</div>

    #ifndef clox_table_h
    #define clox_table_h

    #include "common.h"
    #include "value.h"

    typedef struct {
      int count;
      int capacity;
      Entry* entries;
    } Table;

    #endif

</div>

<div class="source-file-narrow">

*table.h*, create new file

</div>

A hash table is an array of entries. As in our dynamic array earlier, we
keep track of both the allocated size of the array (`capacity`) and the
number of key/value pairs currently stored in it (`count`). The ratio of
count to capacity is exactly the load factor of the hash table.

Each entry is one of these:

<div class="codehilite">

``` insert-before
#include "value.h"
```

<div class="source-file">

*table.h*

</div>

``` insert

typedef struct {
  ObjString* key;
  Value value;
} Entry;
```

``` insert-after

typedef struct {
```

</div>

<div class="source-file-narrow">

*table.h*

</div>

It’s a simple key/value pair. Since the key is always a
<span id="string">string</span>, we store the ObjString pointer directly
instead of wrapping it in a Value. It’s a little faster and smaller this
way.

In clox, we only need to support keys that are strings. Handling other
types of keys doesn’t add much complexity. As long as you can compare
two objects for equality and reduce them to sequences of bits, it’s easy
to use them as hash keys.

To create a new, empty hash table, we declare a constructor-like
function.

<div class="codehilite">

``` insert-before
} Table;
```

<div class="source-file">

*table.h*  
add after struct *Table*

</div>

``` insert
void initTable(Table* table);
```

``` insert-after
#endif
```

</div>

<div class="source-file-narrow">

*table.h*, add after struct *Table*

</div>

We need a new implementation file to define that. While we’re at it,
let’s get all of the pesky includes out of the way.

<div class="codehilite">

<div class="source-file">

*table.c*  
create new file

</div>

    #include <stdlib.h>
    #include <string.h>

    #include "memory.h"
    #include "object.h"
    #include "table.h"
    #include "value.h"

    void initTable(Table* table) {
      table->count = 0;
      table->capacity = 0;
      table->entries = NULL;
    }

</div>

<div class="source-file-narrow">

*table.c*, create new file

</div>

As in our dynamic value array type, a hash table initially starts with
zero capacity and a `NULL` array. We don’t allocate anything until
needed. Assuming we do eventually allocate something, we need to be able
to free it too.

<div class="codehilite">

``` insert-before
void initTable(Table* table);
```

<div class="source-file">

*table.h*  
add after *initTable*()

</div>

``` insert
void freeTable(Table* table);
```

``` insert-after

#endif
```

</div>

<div class="source-file-narrow">

*table.h*, add after *initTable*()

</div>

And its glorious implementation:

<div class="codehilite">

<div class="source-file">

*table.c*  
add after *initTable*()

</div>

    void freeTable(Table* table) {
      FREE_ARRAY(Entry, table->entries, table->capacity);
      initTable(table);
    }

</div>

<div class="source-file-narrow">

*table.c*, add after *initTable*()

</div>

Again, it looks just like a dynamic array. In fact, you can think of a
hash table as basically a dynamic array with a really strange policy for
inserting items. We don’t need to check for `NULL` here since
`FREE_ARRAY()` already handles that gracefully.

### <a href="#hashing-strings" id="hashing-strings"><span
class="small">20 . 4 . 1</span>Hashing strings</a>

Before we can start putting entries in the table, we need to, well, hash
them. To ensure that the entries get distributed uniformly throughout
the array, we want a good hash function that looks at all of the bits of
the key string. If it looked at, say, only the first few characters,
then a series of strings that all shared the same prefix would end up
colliding in the same bucket.

On the other hand, walking the entire string to calculate the hash is
kind of slow. We’d lose some of the performance benefit of the hash
table if we had to walk the string every time we looked for a key in the
table. So we’ll do the obvious thing: cache it.

Over in the “object” module in ObjString, we add:

<div class="codehilite">

``` insert-before
  char* chars;
```

<div class="source-file">

*object.h*  
in struct *ObjString*

</div>

``` insert
  uint32_t hash;
```

``` insert-after
};
```

</div>

<div class="source-file-narrow">

*object.h*, in struct *ObjString*

</div>

Each ObjString stores the hash code for its string. Since strings are
immutable in Lox, we can calculate the hash code once up front and be
certain that it will never get invalidated. Caching it eagerly makes a
kind of sense: allocating the string and copying its characters over is
already an *O(n)* operation, so it’s a good time to also do the *O(n)*
calculation of the string’s hash.

Whenever we call the internal function to allocate a string, we pass in
its hash code.

<div class="codehilite">

<div class="source-file">

*object.c*  
function *allocateString*()  
replace 1 line

</div>

``` insert
static ObjString* allocateString(char* chars, int length,
                                 uint32_t hash) {
```

``` insert-after
  ObjString* string = ALLOCATE_OBJ(ObjString, OBJ_STRING);
```

</div>

<div class="source-file-narrow">

*object.c*, function *allocateString*(), replace 1 line

</div>

That function simply stores the hash in the struct.

<div class="codehilite">

``` insert-before
  string->chars = chars;
```

<div class="source-file">

*object.c*  
in *allocateString*()

</div>

``` insert
  string->hash = hash;
```

``` insert-after
  return string;
}
```

</div>

<div class="source-file-narrow">

*object.c*, in *allocateString*()

</div>

The fun happens over at the callers. `allocateString()` is called from
two places: the function that copies a string and the one that takes
ownership of an existing dynamically allocated string. We’ll start with
the first.

<div class="codehilite">

``` insert-before
ObjString* copyString(const char* chars, int length) {
```

<div class="source-file">

*object.c*  
in *copyString*()

</div>

``` insert
  uint32_t hash = hashString(chars, length);
```

``` insert-after
  char* heapChars = ALLOCATE(char, length + 1);
```

</div>

<div class="source-file-narrow">

*object.c*, in *copyString*()

</div>

No magic here. We calculate the hash code and then pass it along.

<div class="codehilite">

``` insert-before
  memcpy(heapChars, chars, length);
  heapChars[length] = '\0';
```

<div class="source-file">

*object.c*  
in *copyString*()  
replace 1 line

</div>

``` insert
  return allocateString(heapChars, length, hash);
```

``` insert-after
}
```

</div>

<div class="source-file-narrow">

*object.c*, in *copyString*(), replace 1 line

</div>

The other string function is similar.

<div class="codehilite">

``` insert-before
ObjString* takeString(char* chars, int length) {
```

<div class="source-file">

*object.c*  
in *takeString*()  
replace 1 line

</div>

``` insert
  uint32_t hash = hashString(chars, length);
  return allocateString(chars, length, hash);
```

``` insert-after
}
```

</div>

<div class="source-file-narrow">

*object.c*, in *takeString*(), replace 1 line

</div>

The interesting code is over here:

<div class="codehilite">

<div class="source-file">

*object.c*  
add after *allocateString*()

</div>

    static uint32_t hashString(const char* key, int length) {
      uint32_t hash = 2166136261u;
      for (int i = 0; i < length; i++) {
        hash ^= (uint8_t)key[i];
        hash *= 16777619;
      }
      return hash;
    }

</div>

<div class="source-file-narrow">

*object.c*, add after *allocateString*()

</div>

This is the actual bona fide “hash function” in clox. The algorithm is
called “FNV-1a”, and is the shortest decent hash function I know.
Brevity is certainly a virtue in a book that aims to show you every line
of code.

The basic idea is pretty simple, and many hash functions follow the same
pattern. You start with some initial hash value, usually a constant with
certain carefully chosen mathematical properties. Then you walk the data
to be hashed. For each byte (or sometimes word), you mix the bits into
the hash value somehow, and then scramble the resulting bits around
some.

What it means to “mix” and “scramble” can get pretty sophisticated.
Ultimately, though, the basic goal is
*uniformity*<span class="em">—</span>we want the resulting hash values
to be as widely scattered around the numeric range as possible to avoid
collisions and clustering.

### <a href="#inserting-entries" id="inserting-entries"><span
class="small">20 . 4 . 2</span>Inserting entries</a>

Now that string objects know their hash code, we can start putting them
into hash tables.

<div class="codehilite">

``` insert-before
void freeTable(Table* table);
```

<div class="source-file">

*table.h*  
add after *freeTable*()

</div>

``` insert
bool tableSet(Table* table, ObjString* key, Value value);
```

``` insert-after

#endif
```

</div>

<div class="source-file-narrow">

*table.h*, add after *freeTable*()

</div>

This function adds the given key/value pair to the given hash table. If
an entry for that key is already present, the new value overwrites the
old value. The function returns `true` if a new entry was added. Here’s
the implementation:

<div class="codehilite">

<div class="source-file">

*table.c*  
add after *freeTable*()

</div>

    bool tableSet(Table* table, ObjString* key, Value value) {
      Entry* entry = findEntry(table->entries, table->capacity, key);
      bool isNewKey = entry->key == NULL;
      if (isNewKey) table->count++;

      entry->key = key;
      entry->value = value;
      return isNewKey;
    }

</div>

<div class="source-file-narrow">

*table.c*, add after *freeTable*()

</div>

Most of the interesting logic is in `findEntry()` which we’ll get to
soon. That function’s job is to take a key and figure out which bucket
in the array it should go in. It returns a pointer to that
bucket<span class="em">—</span>the address of the Entry in the array.

Once we have a bucket, inserting is straightforward. We update the hash
table’s size, taking care to not increase the count if we overwrote the
value for an already-present key. Then we copy the key and value into
the corresponding fields in the Entry.

We’re missing a little something here, though. We haven’t actually
allocated the Entry array yet. Oops! Before we can insert anything, we
need to make sure we have an array, and that it’s big enough.

<div class="codehilite">

``` insert-before
bool tableSet(Table* table, ObjString* key, Value value) {
```

<div class="source-file">

*table.c*  
in *tableSet*()

</div>

``` insert
  if (table->count + 1 > table->capacity * TABLE_MAX_LOAD) {
    int capacity = GROW_CAPACITY(table->capacity);
    adjustCapacity(table, capacity);
  }
```

``` insert-after
  Entry* entry = findEntry(table->entries, table->capacity, key);
```

</div>

<div class="source-file-narrow">

*table.c*, in *tableSet*()

</div>

This is similar to the code we wrote a while back for growing a dynamic
array. If we don’t have enough capacity to insert an item, we reallocate
and grow the array. The `GROW_CAPACITY()` macro takes an existing
capacity and grows it by a multiple to ensure that we get amortized
constant performance over a series of inserts.

The interesting difference here is that `TABLE_MAX_LOAD` constant.

<div class="codehilite">

``` insert-before
#include "value.h"
```

<div class="source-file">

*table.c*

</div>

``` insert
#define TABLE_MAX_LOAD 0.75
```

``` insert-after
void initTable(Table* table) {
```

</div>

<div class="source-file-narrow">

*table.c*

</div>

This is how we manage the table’s <span id="75">load</span> factor. We
don’t grow when the capacity is completely full. Instead, we grow the
array before then, when the array becomes at least 75% full.

Ideal max load factor varies based on the hash function,
collision-handling strategy, and typical keysets you’ll see. Since a toy
language like Lox doesn’t have “real world” data sets, it’s hard to
optimize this, and I picked 75% somewhat arbitrarily. When you build
your own hash tables, benchmark and tune this.

We’ll get to the implementation of `adjustCapacity()` soon. First, let’s
look at that `findEntry()` function you’ve been wondering about.

<div class="codehilite">

<div class="source-file">

*table.c*  
add after *freeTable*()

</div>

    static Entry* findEntry(Entry* entries, int capacity,
                            ObjString* key) {
      uint32_t index = key->hash % capacity;
      for (;;) {
        Entry* entry = &entries[index];
        if (entry->key == key || entry->key == NULL) {
          return entry;
        }

        index = (index + 1) % capacity;
      }
    }

</div>

<div class="source-file-narrow">

*table.c*, add after *freeTable*()

</div>

This function is the real core of the hash table. It’s responsible for
taking a key and an array of buckets, and figuring out which bucket the
entry belongs in. This function is also where linear probing and
collision handling come into play. We’ll use `findEntry()` both to look
up existing entries in the hash table and to decide where to insert new
ones.

For all that, there isn’t much to it. First, we use modulo to map the
key’s hash code to an index within the array’s bounds. That gives us a
bucket index where, ideally, we’ll be able to find or place the entry.

There are a few cases to check for:

- If the key for the Entry at that array index is `NULL`, then the
  bucket is empty. If we’re using `findEntry()` to look up something in
  the hash table, this means it isn’t there. If we’re using it to
  insert, it means we’ve found a place to add the new entry.

- If the key in the bucket is <span id="equal">equal</span> to the key
  we’re looking for, then that key is already present in the table. If
  we’re doing a lookup, that’s good<span class="em">—</span>we’ve found
  the key we seek. If we’re doing an insert, this means we’ll be
  replacing the value for that key instead of adding a new entry.

It looks like we’re using `==` to see if two strings are equal. That
doesn’t work, does it? There could be two copies of the same string at
different places in memory. Fear not, astute reader. We’ll solve this
further on. And, strangely enough, it’s a hash table that provides the
tool we need.

- Otherwise, the bucket has an entry in it, but with a different key.
  This is a collision. In that case, we start probing. That’s what that
  `for` loop does. We start at the bucket where the entry would ideally
  go. If that bucket is empty or has the same key, we’re done.
  Otherwise, we advance to the next element<span class="em">—</span>this
  is the *linear* part of “linear probing”<span class="em">—</span>and
  check there. If we go past the end of the array, that second modulo
  operator wraps us back around to the beginning.

We exit the loop when we find either an empty bucket or a bucket with
the same key as the one we’re looking for. You might be wondering about
an infinite loop. What if we collide with *every* bucket? Fortunately,
that can’t happen thanks to our load factor. Because we grow the array
as soon as it gets close to being full, we know there will always be
empty buckets.

We return directly from within the loop, yielding a pointer to the found
Entry so the caller can either insert something into it or read from it.
Way back in `tableSet()`, the function that first kicked this off, we
store the new entry in that returned bucket and we’re done.

### <a href="#allocating-and-resizing" id="allocating-and-resizing"><span
class="small">20 . 4 . 3</span>Allocating and resizing</a>

Before we can put entries in the hash table, we do need a place to
actually store them. We need to allocate an array of buckets. That
happens in this function:

<div class="codehilite">

<div class="source-file">

*table.c*  
add after *findEntry*()

</div>

    static void adjustCapacity(Table* table, int capacity) {
      Entry* entries = ALLOCATE(Entry, capacity);
      for (int i = 0; i < capacity; i++) {
        entries[i].key = NULL;
        entries[i].value = NIL_VAL;
      }

      table->entries = entries;
      table->capacity = capacity;
    }

</div>

<div class="source-file-narrow">

*table.c*, add after *findEntry*()

</div>

We create a bucket array with `capacity` entries. After we allocate the
array, we initialize every element to be an empty bucket and then store
the array (and its capacity) in the hash table’s main struct. This code
is fine for when we insert the very first entry into the table, and we
require the first allocation of the array. But what about when we
already have one and we need to grow it?

Back when we were doing a dynamic array, we could just use `realloc()`
and let the C standard library copy everything over. That doesn’t work
for a hash table. Remember that to choose the bucket for each entry, we
take its hash key *modulo the array size*. That means that when the
array size changes, entries may end up in different buckets.

Those new buckets may have new collisions that we need to deal with. So
the simplest way to get every entry where it belongs is to rebuild the
table from scratch by re-inserting every entry into the new empty array.

<div class="codehilite">

``` insert-before
    entries[i].value = NIL_VAL;
  }
```

<div class="source-file">

*table.c*  
in *adjustCapacity*()

</div>

``` insert

  for (int i = 0; i < table->capacity; i++) {
    Entry* entry = &table->entries[i];
    if (entry->key == NULL) continue;

    Entry* dest = findEntry(entries, capacity, entry->key);
    dest->key = entry->key;
    dest->value = entry->value;
  }
```

``` insert-after

  table->entries = entries;
```

</div>

<div class="source-file-narrow">

*table.c*, in *adjustCapacity*()

</div>

We walk through the old array front to back. Any time we find a
non-empty bucket, we insert that entry into the new array. We use
`findEntry()`, passing in the *new* array instead of the one currently
stored in the Table. (This is why `findEntry()` takes a pointer directly
to an Entry array and not the whole `Table` struct. That way, we can
pass the new array and capacity before we’ve stored those in the
struct.)

After that’s done, we can release the memory for the old array.

<div class="codehilite">

``` insert-before
    dest->value = entry->value;
  }
```

<div class="source-file">

*table.c*  
in *adjustCapacity*()

</div>

``` insert
  FREE_ARRAY(Entry, table->entries, table->capacity);
```

``` insert-after
  table->entries = entries;
```

</div>

<div class="source-file-narrow">

*table.c*, in *adjustCapacity*()

</div>

With that, we have a hash table that we can stuff as many entries into
as we like. It handles overwriting existing keys and growing itself as
needed to maintain the desired load capacity.

While we’re at it, let’s also define a helper function for copying all
of the entries of one hash table into another.

<div class="codehilite">

``` insert-before
bool tableSet(Table* table, ObjString* key, Value value);
```

<div class="source-file">

*table.h*  
add after *tableSet*()

</div>

``` insert
void tableAddAll(Table* from, Table* to);
```

``` insert-after

#endif
```

</div>

<div class="source-file-narrow">

*table.h*, add after *tableSet*()

</div>

We won’t need this until much later when we support method inheritance,
but we may as well implement it now while we’ve got all the hash table
stuff fresh in our minds.

<div class="codehilite">

<div class="source-file">

*table.c*  
add after *tableSet*()

</div>

    void tableAddAll(Table* from, Table* to) {
      for (int i = 0; i < from->capacity; i++) {
        Entry* entry = &from->entries[i];
        if (entry->key != NULL) {
          tableSet(to, entry->key, entry->value);
        }
      }
    }

</div>

<div class="source-file-narrow">

*table.c*, add after *tableSet*()

</div>

There’s not much to say about this. It walks the bucket array of the
source hash table. Whenever it finds a non-empty bucket, it adds the
entry to the destination hash table using the `tableSet()` function we
recently defined.

### <a href="#retrieving-values" id="retrieving-values"><span
class="small">20 . 4 . 4</span>Retrieving values</a>

Now that our hash table contains some stuff, let’s start pulling things
back out. Given a key, we can look up the corresponding value, if there
is one, with this function:

<div class="codehilite">

``` insert-before
void freeTable(Table* table);
```

<div class="source-file">

*table.h*  
add after *freeTable*()

</div>

``` insert
bool tableGet(Table* table, ObjString* key, Value* value);
```

``` insert-after
bool tableSet(Table* table, ObjString* key, Value value);
```

</div>

<div class="source-file-narrow">

*table.h*, add after *freeTable*()

</div>

You pass in a table and a key. If it finds an entry with that key, it
returns `true`, otherwise it returns `false`. If the entry exists, the
`value` output parameter points to the resulting value.

Since `findEntry()` already does the hard work, the implementation isn’t
bad.

<div class="codehilite">

<div class="source-file">

*table.c*  
add after *findEntry*()

</div>

    bool tableGet(Table* table, ObjString* key, Value* value) {
      if (table->count == 0) return false;

      Entry* entry = findEntry(table->entries, table->capacity, key);
      if (entry->key == NULL) return false;

      *value = entry->value;
      return true;
    }

</div>

<div class="source-file-narrow">

*table.c*, add after *findEntry*()

</div>

If the table is completely empty, we definitely won’t find the entry, so
we check for that first. This isn’t just an
optimization<span class="em">—</span>it also ensures that we don’t try
to access the bucket array when the array is `NULL`. Otherwise, we let
`findEntry()` work its magic. That returns a pointer to a bucket. If the
bucket is empty, which we detect by seeing if the key is `NULL`, then we
didn’t find an Entry with our key. If `findEntry()` does return a
non-empty Entry, then that’s our match. We take the Entry’s value and
copy it to the output parameter so the caller can get it. Piece of cake.

### <a href="#deleting-entries" id="deleting-entries"><span
class="small">20 . 4 . 5</span>Deleting entries</a>

There is one more fundamental operation a full-featured hash table needs
to support: removing an entry. This seems pretty obvious, if you can add
things, you should be able to *un*-add them, right? But you’d be
surprised how many tutorials on hash tables omit this.

I could have taken that route too. In fact, we use deletion in clox only
in a tiny edge case in the VM. But if you want to actually understand
how to completely implement a hash table, this feels important. I can
sympathize with their desire to overlook it. As we’ll see, deleting from
a hash table that uses <span id="delete">open</span> addressing is
tricky.

With separate chaining, deleting is as easy as removing a node from a
linked list.

At least the declaration is simple.

<div class="codehilite">

``` insert-before
bool tableSet(Table* table, ObjString* key, Value value);
```

<div class="source-file">

*table.h*  
add after *tableSet*()

</div>

``` insert
bool tableDelete(Table* table, ObjString* key);
```

``` insert-after
void tableAddAll(Table* from, Table* to);
```

</div>

<div class="source-file-narrow">

*table.h*, add after *tableSet*()

</div>

The obvious approach is to mirror insertion. Use `findEntry()` to look
up the entry’s bucket. Then clear out the bucket. Done!

In cases where there are no collisions, that works fine. But if a
collision has occurred, then the bucket where the entry lives may be
part of one or more implicit probe sequences. For example, here’s a hash
table containing three keys all with the same preferred bucket, 2:

![A hash table containing 'bagel' in bucket 2, 'biscuit' in bucket 3,
and 'jam' in bucket 4.](image/hash-tables/delete-1.png)

Remember that when we’re walking a probe sequence to find an entry, we
know we’ve reached the end of a sequence and that the entry isn’t
present when we hit an empty bucket. It’s like the probe sequence is a
list of entries and an empty entry terminates that list.

If we delete “biscuit” by simply clearing the Entry, then we break that
probe sequence in the middle, leaving the trailing entries orphaned and
unreachable. Sort of like removing a node from a linked list without
relinking the pointer from the previous node to the next one.

If we later try to look for “jam”, we’d start at “bagel”, stop at the
next empty Entry, and never find it.

![The 'biscuit' entry has been deleted from the hash table, breaking the
chain.](image/hash-tables/delete-2.png)

To solve this, most implementations use a trick called
<span id="tombstone">**tombstones**</span>. Instead of clearing the
entry on deletion, we replace it with a special sentinel entry called a
“tombstone”. When we are following a probe sequence during a lookup, and
we hit a tombstone, we *don’t* treat it like an empty slot and stop
iterating. Instead, we keep going so that deleting an entry doesn’t
break any implicit collision chains and we can still find entries after
it.

![Instead of deleting 'biscuit', it's replaced with a
tombstone.](image/hash-tables/delete-3.png)

The code looks like this:

<div class="codehilite">

<div class="source-file">

*table.c*  
add after *tableSet*()

</div>

    bool tableDelete(Table* table, ObjString* key) {
      if (table->count == 0) return false;

      // Find the entry.
      Entry* entry = findEntry(table->entries, table->capacity, key);
      if (entry->key == NULL) return false;

      // Place a tombstone in the entry.
      entry->key = NULL;
      entry->value = BOOL_VAL(true);
      return true;
    }

</div>

<div class="source-file-narrow">

*table.c*, add after *tableSet*()

</div>

First, we find the bucket containing the entry we want to delete. (If we
don’t find it, there’s nothing to delete, so we bail out.) We replace
the entry with a tombstone. In clox, we use a `NULL` key and a `true`
value to represent that, but any representation that can’t be confused
with an empty bucket or a valid entry works.

![A tombstone enscribed 'Here lies entry biscuit → 3.75, gone but not
deleted'.](image/hash-tables/tombstone.png)

That’s all we need to do to delete an entry. Simple and fast. But all of
the other operations need to correctly handle tombstones too. A
tombstone is a sort of “half” entry. It has some of the characteristics
of a present entry, and some of the characteristics of an empty one.

When we are following a probe sequence during a lookup, and we hit a
tombstone, we note it and keep going.

<div class="codehilite">

``` insert-before
  for (;;) {
    Entry* entry = &entries[index];
```

<div class="source-file">

*table.c*  
in *findEntry*()  
replace 3 lines

</div>

``` insert
    if (entry->key == NULL) {
      if (IS_NIL(entry->value)) {
        // Empty entry.
        return tombstone != NULL ? tombstone : entry;
      } else {
        // We found a tombstone.
        if (tombstone == NULL) tombstone = entry;
      }
    } else if (entry->key == key) {
      // We found the key.
      return entry;
    }
```

``` insert-after

    index = (index + 1) % capacity;
```

</div>

<div class="source-file-narrow">

*table.c*, in *findEntry*(), replace 3 lines

</div>

The first time we pass a tombstone, we store it in this local variable:

<div class="codehilite">

``` insert-before
  uint32_t index = key->hash % capacity;
```

<div class="source-file">

*table.c*  
in *findEntry*()

</div>

``` insert
  Entry* tombstone = NULL;
```

``` insert-after
  for (;;) {
```

</div>

<div class="source-file-narrow">

*table.c*, in *findEntry*()

</div>

If we reach a truly empty entry, then the key isn’t present. In that
case, if we have passed a tombstone, we return its bucket instead of the
later empty one. If we’re calling `findEntry()` in order to insert a
node, that lets us treat the tombstone bucket as empty and reuse it for
the new entry.

Reusing tombstone slots automatically like this helps reduce the number
of tombstones wasting space in the bucket array. In typical use cases
where there is a mixture of insertions and deletions, the number of
tombstones grows for a while and then tends to stabilize.

Even so, there’s no guarantee that a large number of deletes won’t cause
the array to be full of tombstones. In the very worst case, we could end
up with *no* empty buckets. That would be bad because, remember, the
only thing preventing an infinite loop in `findEntry()` is the
assumption that we’ll eventually hit an empty bucket.

So we need to be thoughtful about how tombstones interact with the
table’s load factor and resizing. The key question is, when calculating
the load factor, should we treat tombstones like full buckets or empty
ones?

### <a href="#counting-tombstones" id="counting-tombstones"><span
class="small">20 . 4 . 6</span>Counting tombstones</a>

If we treat tombstones like full buckets, then we may end up with a
bigger array than we probably need because it artificially inflates the
load factor. There are tombstones we could reuse, but they aren’t
treated as unused so we end up growing the array prematurely.

But if we treat tombstones like empty buckets and *don’t* include them
in the load factor, then we run the risk of ending up with *no* actual
empty buckets to terminate a lookup. An infinite loop is a much worse
problem than a few extra array slots, so for load factor, we consider
tombstones to be full buckets.

That’s why we don’t reduce the count when deleting an entry in the
previous code. The count is no longer the number of entries in the hash
table, it’s the number of entries plus tombstones. That implies that we
increment the count during insertion only if the new entry goes into an
entirely empty bucket.

<div class="codehilite">

``` insert-before
  bool isNewKey = entry->key == NULL;
```

<div class="source-file">

*table.c*  
in *tableSet*()  
replace 1 line

</div>

``` insert
  if (isNewKey && IS_NIL(entry->value)) table->count++;
```

``` insert-after

  entry->key = key;
```

</div>

<div class="source-file-narrow">

*table.c*, in *tableSet*(), replace 1 line

</div>

If we are replacing a tombstone with a new entry, the bucket has already
been accounted for and the count doesn’t change.

When we resize the array, we allocate a new array and re-insert all of
the existing entries into it. During that process, we *don’t* copy the
tombstones over. They don’t add any value since we’re rebuilding the
probe sequences anyway, and would just slow down lookups. That means we
need to recalculate the count since it may change during a resize. So we
clear it out:

<div class="codehilite">

``` insert-before
  }
```

<div class="source-file">

*table.c*  
in *adjustCapacity*()

</div>

``` insert
  table->count = 0;
```

``` insert-after
  for (int i = 0; i < table->capacity; i++) {
```

</div>

<div class="source-file-narrow">

*table.c*, in *adjustCapacity*()

</div>

Then each time we find a non-tombstone entry, we increment it.

<div class="codehilite">

``` insert-before
    dest->value = entry->value;
```

<div class="source-file">

*table.c*  
in *adjustCapacity*()

</div>

``` insert
    table->count++;
```

``` insert-after
  }
```

</div>

<div class="source-file-narrow">

*table.c*, in *adjustCapacity*()

</div>

This means that when we grow the capacity, we may end up with *fewer*
entries in the resulting larger array because all of the tombstones get
discarded. That’s a little wasteful, but not a huge practical problem.

I find it interesting that much of the work to support deleting entries
is in `findEntry()` and `adjustCapacity()`. The actual delete logic is
quite simple and fast. In practice, deletions tend to be rare, so you’d
expect a hash table to do as much work as it can in the delete function
and leave the other functions alone to keep them faster. With our
tombstone approach, deletes are fast, but lookups get penalized.

I did a little benchmarking to test this out in a few different deletion
scenarios. I was surprised to discover that tombstones did end up being
faster overall compared to doing all the work during deletion to
reinsert the affected entries.

But if you think about it, it’s not that the tombstone approach pushes
the work of fully deleting an entry to other operations, it’s more that
it makes deleting *lazy*. At first, it does the minimal work to turn the
entry into a tombstone. That can cause a penalty when later lookups have
to skip over it. But it also allows that tombstone bucket to be reused
by a later insert too. That reuse is a very efficient way to avoid the
cost of rearranging all of the following affected entries. You basically
recycle a node in the chain of probed entries. It’s a neat trick.

## <a href="#string-interning" id="string-interning"><span
class="small">20 . 5</span>String Interning</a>

We’ve got ourselves a hash table that mostly works, though it has a
critical flaw in its center. Also, we aren’t using it for anything yet.
It’s time to address both of those and, in the process, learn a classic
technique used by interpreters.

The reason the hash table doesn’t totally work is that when
`findEntry()` checks to see if an existing key matches the one it’s
looking for, it uses `==` to compare two strings for equality. That only
returns true if the two keys are the exact same string in memory. Two
separate strings with the same characters should be considered equal,
but aren’t.

Remember, back when we added strings in the last chapter, we added
[explicit support to compare the strings
character-by-character](strings.html#operations-on-strings) in order to
get true value equality. We could do that in `findEntry()`, but that’s
<span id="hash-collision">slow</span>.

In practice, we would first compare the hash codes of the two strings.
That quickly detects almost all different
strings<span class="em">—</span>it wouldn’t be a very good hash function
if it didn’t. But when the two hashes are the same, we still have to
compare characters to make sure we didn’t have a hash collision on
different strings.

Instead, we’ll use a technique called **string interning**. The core
problem is that it’s possible to have different strings in memory with
the same characters. Those need to behave like equivalent values even
though they are distinct objects. They’re essentially duplicates, and we
have to compare all of their bytes to detect that.

<span id="intern">String interning</span> is a process of deduplication.
We create a collection of “interned” strings. Any string in that
collection is guaranteed to be textually distinct from all others. When
you intern a string, you look for a matching string in the collection.
If found, you use that original one. Otherwise, the string you have is
unique, so you add it to the collection.

I’m guessing “intern” is short for “internal”. I think the idea is that
the language’s runtime keeps its own “internal” collection of these
strings, whereas other strings could be user created and floating around
in memory. When you intern a string, you ask the runtime to add the
string to that internal collection and return a pointer to it.

Languages vary in how much string interning they do and how it’s exposed
to the user. Lua interns *all* strings, which is what clox will do too.
Lisp, Scheme, Smalltalk, Ruby and others have a separate string-like
type called “symbol” that is implicitly interned. (This is why they say
symbols are “faster” in Ruby.) Java interns constant strings by default,
and provides an API to let you explicitly intern any string you give it.

In this way, you know that each sequence of characters is represented by
only one string in memory. This makes value equality trivial. If two
strings point to the same address in memory, they are obviously the same
string and must be equal. And, because we know strings are unique, if
two strings point to different addresses, they must be distinct strings.

Thus, pointer equality exactly matches value equality. Which in turn
means that our existing `==` in `findEntry()` does the right thing. Or,
at least, it will once we intern all the strings. In order to reliably
deduplicate all strings, the VM needs to be able to find every string
that’s created. We do that by giving it a hash table to store them all.

<div class="codehilite">

``` insert-before
  Value* stackTop;
```

<div class="source-file">

*vm.h*  
in struct *VM*

</div>

``` insert
  Table strings;
```

``` insert-after
  Obj* objects;
```

</div>

<div class="source-file-narrow">

*vm.h*, in struct *VM*

</div>

As usual, we need an include.

<div class="codehilite">

``` insert-before
#include "chunk.h"
```

<div class="source-file">

*vm.h*

</div>

``` insert
#include "table.h"
```

``` insert-after
#include "value.h"
```

</div>

<div class="source-file-narrow">

*vm.h*

</div>

When we spin up a new VM, the string table is empty.

<div class="codehilite">

``` insert-before
  vm.objects = NULL;
```

<div class="source-file">

*vm.c*  
in *initVM*()

</div>

``` insert
  initTable(&vm.strings);
```

``` insert-after
}
```

</div>

<div class="source-file-narrow">

*vm.c*, in *initVM*()

</div>

And when we shut down the VM, we clean up any resources used by the
table.

<div class="codehilite">

``` insert-before
void freeVM() {
```

<div class="source-file">

*vm.c*  
in *freeVM*()

</div>

``` insert
  freeTable(&vm.strings);
```

``` insert-after
  freeObjects();
```

</div>

<div class="source-file-narrow">

*vm.c*, in *freeVM*()

</div>

Some languages have a separate type or an explicit step to intern a
string. For clox, we’ll automatically intern every one. That means
whenever we create a new unique string, we add it to the table.

<div class="codehilite">

``` insert-before
  string->hash = hash;
```

<div class="source-file">

*object.c*  
in *allocateString*()

</div>

``` insert
  tableSet(&vm.strings, string, NIL_VAL);
```

``` insert-after
  return string;
```

</div>

<div class="source-file-narrow">

*object.c*, in *allocateString*()

</div>

We’re using the table more like a hash *set* than a hash *table*. The
keys are the strings and those are all we care about, so we just use
`nil` for the values.

This gets a string into the table assuming that it’s unique, but we need
to actually check for duplication before we get here. We do that in the
two higher-level functions that call `allocateString()`. Here’s one:

<div class="codehilite">

``` insert-before
  uint32_t hash = hashString(chars, length);
```

<div class="source-file">

*object.c*  
in *copyString*()

</div>

``` insert
  ObjString* interned = tableFindString(&vm.strings, chars, length,
                                        hash);
  if (interned != NULL) return interned;
```

``` insert-after
  char* heapChars = ALLOCATE(char, length + 1);
```

</div>

<div class="source-file-narrow">

*object.c*, in *copyString*()

</div>

When copying a string into a new LoxString, we look it up in the string
table first. If we find it, instead of “copying”, we just return a
reference to that string. Otherwise, we fall through, allocate a new
string, and store it in the string table.

Taking ownership of a string is a little different.

<div class="codehilite">

``` insert-before
  uint32_t hash = hashString(chars, length);
```

<div class="source-file">

*object.c*  
in *takeString*()

</div>

``` insert
  ObjString* interned = tableFindString(&vm.strings, chars, length,
                                        hash);
  if (interned != NULL) {
    FREE_ARRAY(char, chars, length + 1);
    return interned;
  }
```

``` insert-after
  return allocateString(chars, length, hash);
```

</div>

<div class="source-file-narrow">

*object.c*, in *takeString*()

</div>

Again, we look up the string in the string table first. If we find it,
before we return it, we free the memory for the string that was passed
in. Since ownership is being passed to this function and we no longer
need the duplicate string, it’s up to us to free it.

Before we get to the new function we need to write, there’s one more
include.

<div class="codehilite">

``` insert-before
#include "object.h"
```

<div class="source-file">

*object.c*

</div>

``` insert
#include "table.h"
```

``` insert-after
#include "value.h"
```

</div>

<div class="source-file-narrow">

*object.c*

</div>

To look for a string in the table, we can’t use the normal `tableGet()`
function because that calls `findEntry()`, which has the exact problem
with duplicate strings that we’re trying to fix right now. Instead, we
use this new function:

<div class="codehilite">

``` insert-before
void tableAddAll(Table* from, Table* to);
```

<div class="source-file">

*table.h*  
add after *tableAddAll*()

</div>

``` insert
ObjString* tableFindString(Table* table, const char* chars,
                           int length, uint32_t hash);
```

``` insert-after

#endif
```

</div>

<div class="source-file-narrow">

*table.h*, add after *tableAddAll*()

</div>

The implementation looks like so:

<div class="codehilite">

<div class="source-file">

*table.c*  
add after *tableAddAll*()

</div>

    ObjString* tableFindString(Table* table, const char* chars,
                               int length, uint32_t hash) {
      if (table->count == 0) return NULL;

      uint32_t index = hash % table->capacity;
      for (;;) {
        Entry* entry = &table->entries[index];
        if (entry->key == NULL) {
          // Stop if we find an empty non-tombstone entry.
          if (IS_NIL(entry->value)) return NULL;
        } else if (entry->key->length == length &&
            entry->key->hash == hash &&
            memcmp(entry->key->chars, chars, length) == 0) {
          // We found it.
          return entry->key;
        }

        index = (index + 1) % table->capacity;
      }
    }

</div>

<div class="source-file-narrow">

*table.c*, add after *tableAddAll*()

</div>

It appears we have copy-pasted `findEntry()`. There is a lot of
redundancy, but also a couple of key differences. First, we pass in the
raw character array of the key we’re looking for instead of an
ObjString. At the point that we call this, we haven’t created an
ObjString yet.

Second, when checking to see if we found the key, we look at the actual
strings. We first see if they have matching lengths and hashes. Those
are quick to check and if they aren’t equal, the strings definitely
aren’t the same.

If there is a hash collision, we do an actual character-by-character
string comparison. This is the one place in the VM where we actually
test strings for textual equality. We do it here to deduplicate strings
and then the rest of the VM can take for granted that any two strings at
different addresses in memory must have different contents.

In fact, now that we’ve interned all the strings, we can take advantage
of it in the bytecode interpreter. When a user does `==` on two objects
that happen to be strings, we don’t need to test the characters any
more.

<div class="codehilite">

``` insert-before
    case VAL_NUMBER: return AS_NUMBER(a) == AS_NUMBER(b);
```

<div class="source-file">

*value.c*  
in *valuesEqual*()  
replace 7 lines

</div>

``` insert
    case VAL_OBJ:    return AS_OBJ(a) == AS_OBJ(b);
```

``` insert-after
    default:         return false; // Unreachable.
```

</div>

<div class="source-file-narrow">

*value.c*, in *valuesEqual*(), replace 7 lines

</div>

We’ve added a little overhead when creating strings to intern them. But
in return, at runtime, the equality operator on strings is much faster.
With that, we have a full-featured hash table ready for us to use for
tracking variables, instances, or any other key-value pairs that might
show up.

We also sped up testing strings for equality. This is nice for when the
user does `==` on strings. But it’s even more critical in a dynamically
typed language like Lox where method calls and instance fields are
looked up by name at runtime. If testing a string for equality is slow,
then that means looking up a method by name is slow. And if *that’s*
slow in your object-oriented language, then *everything* is slow.

<div class="challenges">

## <a href="#challenges" id="challenges">Challenges</a>

1.  In clox, we happen to only need keys that are strings, so the hash
    table we built is hardcoded for that key type. If we exposed hash
    tables to Lox users as a first-class collection, it would be useful
    to support different kinds of keys.

    Add support for keys of the other primitive types: numbers,
    Booleans, and `nil`. Later, clox will support user-defined classes.
    If we want to support keys that are instances of those classes, what
    kind of complexity does that add?

2.  Hash tables have a lot of knobs you can tweak that affect their
    performance. You decide whether to use separate chaining or open
    addressing. Depending on which fork in that road you take, you can
    tune how many entries are stored in each node, or the probing
    strategy you use. You control the hash function, load factor, and
    growth rate.

    All of this variety wasn’t created just to give CS doctoral
    candidates something to <span id="publish">publish</span> theses on:
    each has its uses in the many varied domains and hardware scenarios
    where hashing comes into play. Look up a few hash table
    implementations in different open source systems, research the
    choices they made, and try to figure out why they did things that
    way.

    Well, at least that wasn’t the *only* reason they were created.
    Whether that was the *main* reason is up for debate.

3.  Benchmarking a hash table is notoriously difficult. A hash table
    implementation may perform well with some keysets and poorly with
    others. It may work well at small sizes but degrade as it grows, or
    vice versa. It may choke when deletions are common, but fly when
    they aren’t. Creating benchmarks that accurately represent how your
    users will use the hash table is a challenge.

    Write a handful of different benchmark programs to validate our hash
    table implementation. How does the performance vary between them?
    Why did you choose the specific test cases you chose?

</div>

<a href="global-variables.html" class="next">Next Chapter: “Global
Variables” →</a> Handcrafted by Robert Nystrom — <a
href="https://github.com/munificent/craftinginterpreters/blob/master/LICENSE"
target="_blank">© 2015 – 2021</a>

</div>

[![](image/logotype.png "Crafting Interpreters")](/)

<div class="contents">

### [Global Variables<span class="small">21</span>](#top)

- [<span class="small">21.1</span> Statements](#statements)
- [<span class="small">21.2</span> Variable
  Declarations](#variable-declarations)
- [<span class="small">21.3</span> Reading
  Variables](#reading-variables)
- [<span class="small">21.4</span> Assignment](#assignment)
- 
- [Challenges](#challenges)

<div class="prev-next">

<a href="hash-tables.html" class="left"
title="Hash Tables">← Previous</a>
[↑ Up](a-bytecode-virtual-machine.html "A Bytecode Virtual Machine")
<a href="local-variables.html" class="right"
title="Local Variables">Next →</a>

</div>

</div>

[![](image/logotype.png "Crafting Interpreters")](/)
<a href="hash-tables.html" class="prev" title="Hash Tables">←</a>
<a href="local-variables.html" class="next"
title="Local Variables">→</a>

<div class="page">

<div class="nav-wrapper">

[![](image/logotype.png "Crafting Interpreters")](/)

<div class="expandable">

### [Global Variables<span class="small">21</span>](#top)

- [<span class="small">21.1</span> Statements](#statements)
- [<span class="small">21.2</span> Variable
  Declarations](#variable-declarations)
- [<span class="small">21.3</span> Reading
  Variables](#reading-variables)
- [<span class="small">21.4</span> Assignment](#assignment)
- 
- [Challenges](#challenges)

<div class="prev-next">

<a href="hash-tables.html" class="left"
title="Hash Tables">← Previous</a>
[↑ Up](a-bytecode-virtual-machine.html "A Bytecode Virtual Machine")
<a href="local-variables.html" class="right"
title="Local Variables">Next →</a>

</div>

</div>

<span id="expand-nav">≡</span>

</div>

<div class="number">

21

</div>

# Global Variables

> If only there could be an invention that bottled up a memory, like
> scent. And it never faded, and it never got stale. And then, when one
> wanted it, the bottle could be uncorked, and it would be like living
> the moment all over again.
>
> Daphne du Maurier, *Rebecca*

The [previous chapter](hash-tables.html) was a long exploration of one
big, deep, fundamental computer science data structure. Heavy on theory
and concept. There may have been some discussion of big-O notation and
algorithms. This chapter has fewer intellectual pretensions. There are
no large ideas to learn. Instead, it’s a handful of straightforward
engineering tasks. Once we’ve completed them, our virtual machine will
support variables.

Actually, it will support only *global* variables. Locals are coming in
the [next chapter](local-variables.html). In jlox, we managed to cram
them both into a single chapter because we used the same implementation
technique for all variables. We built a chain of environments, one for
each scope, all the way up to the top. That was a simple, clean way to
learn how to manage state.

But it’s also *slow*. Allocating a new hash table each time you enter a
block or call a function is not the road to a fast VM. Given how much
code is concerned with using variables, if variables go slow, everything
goes slow. For clox, we’ll improve that by using a much more efficient
strategy for <span id="different">local</span> variables, but globals
aren’t as easily optimized.

This is a common meta-strategy in sophisticated language
implementations. Often, the same language feature will have multiple
implementation techniques, each tuned for different use patterns. For
example, JavaScript VMs often have a faster representation for objects
that are used more like instances of classes compared to other objects
whose set of properties is more freely modified. C and C++ compilers
usually have a variety of ways to compile `switch` statements based on
the number of cases and how densely packed the case values are.

A quick refresher on Lox semantics: Global variables in Lox are “late
bound”, or resolved dynamically. This means you can compile a chunk of
code that refers to a global variable before it’s defined. As long as
the code doesn’t *execute* before the definition happens, everything is
fine. In practice, that means you can refer to later variables inside
the body of functions.

<div class="codehilite">

    fun showVariable() {
      print global;
    }

    var global = "after";
    showVariable();

</div>

Code like this might seem odd, but it’s handy for defining mutually
recursive functions. It also plays nicer with the REPL. You can write a
little function in one line, then define the variable it uses in the
next.

Local variables work differently. Since a local variable’s declaration
*always* occurs before it is used, the VM can resolve them at compile
time, even in a simple single-pass compiler. That will let us use a
smarter representation for locals. But that’s for the next chapter.
Right now, let’s just worry about globals.

## <a href="#statements" id="statements"><span
class="small">21 . 1</span>Statements</a>

Variables come into being using variable declarations, which means now
is also the time to add support for statements to our compiler. If you
recall, Lox splits statements into two categories. “Declarations” are
those statements that bind a new name to a value. The other kinds of
statements<span class="em">—</span>control flow, print,
etc.<span class="em">—</span>are just called “statements”. We disallow
declarations directly inside control flow statements, like this:

<div class="codehilite">

    if (monday) var croissant = "yes"; // Error.

</div>

Allowing it would raise confusing questions around the scope of the
variable. So, like other languages, we prohibit it syntactically by
having a separate grammar rule for the subset of statements that *are*
allowed inside a control flow body.

<div class="codehilite">

    statement      → exprStmt
                   | forStmt
                   | ifStmt
                   | printStmt
                   | returnStmt
                   | whileStmt
                   | block ;

</div>

Then we use a separate rule for the top level of a script and inside a
block.

<div class="codehilite">

    declaration    → classDecl
                   | funDecl
                   | varDecl
                   | statement ;

</div>

The `declaration` rule contains the statements that declare names, and
also includes `statement` so that all statement types are allowed. Since
`block` itself is in `statement`, you can put declarations
<span id="parens">inside</span> a control flow construct by nesting them
inside a block.

Blocks work sort of like parentheses do for expressions. A block lets
you put the “lower-precedence” declaration statements in places where
only a “higher-precedence” non-declaring statement is allowed.

In this chapter, we’ll cover only a couple of statements and one
declaration.

<div class="codehilite">

    statement      → exprStmt
                   | printStmt ;

    declaration    → varDecl
                   | statement ;

</div>

Up to now, our VM considered a “program” to be a single expression since
that’s all we could parse and compile. In a full Lox implementation, a
program is a sequence of declarations. We’re ready to support that now.

<div class="codehilite">

``` insert-before
  advance();
```

<div class="source-file">

*compiler.c*  
in *compile*()  
replace 2 lines

</div>

``` insert

  while (!match(TOKEN_EOF)) {
    declaration();
  }
```

``` insert-after
  endCompiler();
```

</div>

<div class="source-file-narrow">

*compiler.c*, in *compile*(), replace 2 lines

</div>

We keep compiling declarations until we hit the end of the source file.
We compile a single declaration using this:

<div class="codehilite">

<div class="source-file">

*compiler.c*  
add after *expression*()

</div>

    static void declaration() {
      statement();
    }

</div>

<div class="source-file-narrow">

*compiler.c*, add after *expression*()

</div>

We’ll get to variable declarations later in the chapter, so for now, we
simply forward to `statement()`.

<div class="codehilite">

<div class="source-file">

*compiler.c*  
add after *declaration*()

</div>

    static void statement() {
      if (match(TOKEN_PRINT)) {
        printStatement();
      }
    }

</div>

<div class="source-file-narrow">

*compiler.c*, add after *declaration*()

</div>

Blocks can contain declarations, and control flow statements can contain
other statements. That means these two functions will eventually be
recursive. We may as well write out the forward declarations now.

<div class="codehilite">

``` insert-before
static void expression();
```

<div class="source-file">

*compiler.c*  
add after *expression*()

</div>

``` insert
static void statement();
static void declaration();
```

``` insert-after
static ParseRule* getRule(TokenType type);
```

</div>

<div class="source-file-narrow">

*compiler.c*, add after *expression*()

</div>

### <a href="#print-statements" id="print-statements"><span
class="small">21 . 1 . 1</span>Print statements</a>

We have two statement types to support in this chapter. Let’s start with
`print` statements, which begin, naturally enough, with a `print` token.
We detect that using this helper function:

<div class="codehilite">

<div class="source-file">

*compiler.c*  
add after *consume*()

</div>

    static bool match(TokenType type) {
      if (!check(type)) return false;
      advance();
      return true;
    }

</div>

<div class="source-file-narrow">

*compiler.c*, add after *consume*()

</div>

You may recognize it from jlox. If the current token has the given type,
we consume the token and return `true`. Otherwise we leave the token
alone and return `false`. This <span id="turtles">helper</span> function
is implemented in terms of this other helper:

It’s helpers all the way down!

<div class="codehilite">

<div class="source-file">

*compiler.c*  
add after *consume*()

</div>

    static bool check(TokenType type) {
      return parser.current.type == type;
    }

</div>

<div class="source-file-narrow">

*compiler.c*, add after *consume*()

</div>

The `check()` function returns `true` if the current token has the given
type. It seems a little <span id="read">silly</span> to wrap this in a
function, but we’ll use it more later, and I think short verb-named
functions like this make the parser easier to read.

This sounds trivial, but handwritten parsers for non-toy languages get
pretty big. When you have thousands of lines of code, a utility function
that turns two lines into one and makes the result a little more
readable easily earns its keep.

If we did match the `print` token, then we compile the rest of the
statement here:

<div class="codehilite">

<div class="source-file">

*compiler.c*  
add after *expression*()

</div>

    static void printStatement() {
      expression();
      consume(TOKEN_SEMICOLON, "Expect ';' after value.");
      emitByte(OP_PRINT);
    }

</div>

<div class="source-file-narrow">

*compiler.c*, add after *expression*()

</div>

A `print` statement evaluates an expression and prints the result, so we
first parse and compile that expression. The grammar expects a semicolon
after that, so we consume it. Finally, we emit a new instruction to
print the result.

<div class="codehilite">

``` insert-before
  OP_NEGATE,
```

<div class="source-file">

*chunk.h*  
in enum *OpCode*

</div>

``` insert
  OP_PRINT,
```

``` insert-after
  OP_RETURN,
```

</div>

<div class="source-file-narrow">

*chunk.h*, in enum *OpCode*

</div>

At runtime, we execute this instruction like so:

<div class="codehilite">

``` insert-before
        break;
```

<div class="source-file">

*vm.c*  
in *run*()

</div>

``` insert
      case OP_PRINT: {
        printValue(pop());
        printf("\n");
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

When the interpreter reaches this instruction, it has already executed
the code for the expression, leaving the result value on top of the
stack. Now we simply pop and print it.

Note that we don’t push anything else after that. This is a key
difference between expressions and statements in the VM. Every bytecode
instruction has a <span id="effect">**stack effect**</span> that
describes how the instruction modifies the stack. For example, `OP_ADD`
pops two values and pushes one, leaving the stack one element smaller
than before.

The stack is one element shorter after an `OP_ADD`, so its effect is -1:

![The stack effect of an OP_ADD
instruction.](image/global-variables/stack-effect.png)

You can sum the stack effects of a series of instructions to get their
total effect. When you add the stack effects of the series of
instructions compiled from any complete expression, it will total one.
Each expression leaves one result value on the stack.

The bytecode for an entire statement has a total stack effect of zero.
Since a statement produces no values, it ultimately leaves the stack
unchanged, though it of course uses the stack while it’s doing its
thing. This is important because when we get to control flow and
looping, a program might execute a long series of statements. If each
statement grew or shrank the stack, it might eventually overflow or
underflow.

While we’re in the interpreter loop, we should delete a bit of code.

<div class="codehilite">

``` insert-before
      case OP_RETURN: {
```

<div class="source-file">

*vm.c*  
in *run*()  
replace 2 lines

</div>

``` insert
        // Exit interpreter.
```

``` insert-after
        return INTERPRET_OK;
```

</div>

<div class="source-file-narrow">

*vm.c*, in *run*(), replace 2 lines

</div>

When the VM only compiled and evaluated a single expression, we had some
temporary code in `OP_RETURN` to output the value. Now that we have
statements and `print`, we don’t need that anymore. We’re one
<span id="return">step</span> closer to the complete implementation of
clox.

We’re only one step closer, though. We will revisit `OP_RETURN` again
when we add functions. Right now, it exits the entire interpreter loop.

As usual, a new instruction needs support in the disassembler.

<div class="codehilite">

``` insert-before
      return simpleInstruction("OP_NEGATE", offset);
```

<div class="source-file">

*debug.c*  
in *disassembleInstruction*()

</div>

``` insert
    case OP_PRINT:
      return simpleInstruction("OP_PRINT", offset);
```

``` insert-after
    case OP_RETURN:
```

</div>

<div class="source-file-narrow">

*debug.c*, in *disassembleInstruction*()

</div>

That’s our `print` statement. If you want, give it a whirl:

<div class="codehilite">

    print 1 + 2;
    print 3 * 4;

</div>

Exciting! OK, maybe not thrilling, but we can build scripts that contain
as many statements as we want now, which feels like progress.

### <a href="#expression-statements" id="expression-statements"><span
class="small">21 . 1 . 2</span>Expression statements</a>

Wait until you see the next statement. If we *don’t* see a `print`
keyword, then we must be looking at an expression statement.

<div class="codehilite">

``` insert-before
    printStatement();
```

<div class="source-file">

*compiler.c*  
in *statement*()

</div>

``` insert
  } else {
    expressionStatement();
```

``` insert-after
  }
```

</div>

<div class="source-file-narrow">

*compiler.c*, in *statement*()

</div>

It’s parsed like so:

<div class="codehilite">

<div class="source-file">

*compiler.c*  
add after *expression*()

</div>

    static void expressionStatement() {
      expression();
      consume(TOKEN_SEMICOLON, "Expect ';' after expression.");
      emitByte(OP_POP);
    }

</div>

<div class="source-file-narrow">

*compiler.c*, add after *expression*()

</div>

An “expression statement” is simply an expression followed by a
semicolon. They’re how you write an expression in a context where a
statement is expected. Usually, it’s so that you can call a function or
evaluate an assignment for its side effect, like this:

<div class="codehilite">

    brunch = "quiche";
    eat(brunch);

</div>

Semantically, an expression statement evaluates the expression and
discards the result. The compiler directly encodes that behavior. It
compiles the expression, and then emits an `OP_POP` instruction.

<div class="codehilite">

``` insert-before
  OP_FALSE,
```

<div class="source-file">

*chunk.h*  
in enum *OpCode*

</div>

``` insert
  OP_POP,
```

``` insert-after
  OP_EQUAL,
```

</div>

<div class="source-file-narrow">

*chunk.h*, in enum *OpCode*

</div>

As the name implies, that instruction pops the top value off the stack
and forgets it.

<div class="codehilite">

``` insert-before
      case OP_FALSE: push(BOOL_VAL(false)); break;
```

<div class="source-file">

*vm.c*  
in *run*()

</div>

``` insert
      case OP_POP: pop(); break;
```

``` insert-after
      case OP_EQUAL: {
```

</div>

<div class="source-file-narrow">

*vm.c*, in *run*()

</div>

We can disassemble it too.

<div class="codehilite">

``` insert-before
      return simpleInstruction("OP_FALSE", offset);
```

<div class="source-file">

*debug.c*  
in *disassembleInstruction*()

</div>

``` insert
    case OP_POP:
      return simpleInstruction("OP_POP", offset);
```

``` insert-after
    case OP_EQUAL:
```

</div>

<div class="source-file-narrow">

*debug.c*, in *disassembleInstruction*()

</div>

Expression statements aren’t very useful yet since we can’t create any
expressions that have side effects, but they’ll be essential when we
[add functions later](calls-and-functions.html). The
<span id="majority">majority</span> of statements in real-world code in
languages like C are expression statements.

By my count, 80 of the 149 statements, in the version of “compiler.c”
that we have at the end of this chapter are expression statements.

### <a href="#error-synchronization" id="error-synchronization"><span
class="small">21 . 1 . 3</span>Error synchronization</a>

While we’re getting this initial work done in the compiler, we can tie
off a loose end we left [several chapters
back](compiling-expressions.html#handling-syntax-errors). Like jlox,
clox uses panic mode error recovery to minimize the number of cascaded
compile errors that it reports. The compiler exits panic mode when it
reaches a synchronization point. For Lox, we chose statement boundaries
as that point. Now that we have statements, we can implement
synchronization.

<div class="codehilite">

``` insert-before
  statement();
```

<div class="source-file">

*compiler.c*  
in *declaration*()

</div>

``` insert

  if (parser.panicMode) synchronize();
```

``` insert-after
}
```

</div>

<div class="source-file-narrow">

*compiler.c*, in *declaration*()

</div>

If we hit a compile error while parsing the previous statement, we enter
panic mode. When that happens, after the statement we start
synchronizing.

<div class="codehilite">

<div class="source-file">

*compiler.c*  
add after *printStatement*()

</div>

    static void synchronize() {
      parser.panicMode = false;

      while (parser.current.type != TOKEN_EOF) {
        if (parser.previous.type == TOKEN_SEMICOLON) return;
        switch (parser.current.type) {
          case TOKEN_CLASS:
          case TOKEN_FUN:
          case TOKEN_VAR:
          case TOKEN_FOR:
          case TOKEN_IF:
          case TOKEN_WHILE:
          case TOKEN_PRINT:
          case TOKEN_RETURN:
            return;

          default:
            ; // Do nothing.
        }

        advance();
      }
    }

</div>

<div class="source-file-narrow">

*compiler.c*, add after *printStatement*()

</div>

We skip tokens indiscriminately until we reach something that looks like
a statement boundary. We recognize the boundary by looking for a
preceding token that can end a statement, like a semicolon. Or we’ll
look for a subsequent token that begins a statement, usually one of the
control flow or declaration keywords.

## <a href="#variable-declarations" id="variable-declarations"><span
class="small">21 . 2</span>Variable Declarations</a>

Merely being able to *print* doesn’t win your language any prizes at the
programming language <span id="fair">fair</span>, so let’s move on to
something a little more ambitious and get variables going. There are
three operations we need to support:

I can’t help but imagine a “language fair” like some country 4H thing.
Rows of straw-lined stalls full of baby languages *moo*ing and *baa*ing
at each other.

- Declaring a new variable using a `var` statement.
- Accessing the value of a variable using an identifier expression.
- Storing a new value in an existing variable using an assignment
  expression.

We can’t do either of the last two until we have some variables, so we
start with declarations.

<div class="codehilite">

``` insert-before
static void declaration() {
```

<div class="source-file">

*compiler.c*  
in *declaration*()  
replace 1 line

</div>

``` insert
  if (match(TOKEN_VAR)) {
    varDeclaration();
  } else {
    statement();
  }
```

``` insert-after

  if (parser.panicMode) synchronize();
```

</div>

<div class="source-file-narrow">

*compiler.c*, in *declaration*(), replace 1 line

</div>

The placeholder parsing function we sketched out for the declaration
grammar rule has an actual production now. If we match a `var` token, we
jump here:

<div class="codehilite">

<div class="source-file">

*compiler.c*  
add after *expression*()

</div>

    static void varDeclaration() {
      uint8_t global = parseVariable("Expect variable name.");

      if (match(TOKEN_EQUAL)) {
        expression();
      } else {
        emitByte(OP_NIL);
      }
      consume(TOKEN_SEMICOLON,
              "Expect ';' after variable declaration.");

      defineVariable(global);
    }

</div>

<div class="source-file-narrow">

*compiler.c*, add after *expression*()

</div>

The keyword is followed by the variable name. That’s compiled by
`parseVariable()`, which we’ll get to in a second. Then we look for an
`=` followed by an initializer expression. If the user doesn’t
initialize the variable, the compiler implicitly initializes it to
<span id="nil">`nil`</span> by emitting an `OP_NIL` instruction. Either
way, we expect the statement to be terminated with a semicolon.

Essentially, the compiler desugars a variable declaration like:

<div class="codehilite">

    var a;

</div>

into:

<div class="codehilite">

    var a = nil;

</div>

The code it generates for the former is identical to what it produces
for the latter.

There are two new functions here for working with variables and
identifiers. Here is the first:

<div class="codehilite">

``` insert-before
static void parsePrecedence(Precedence precedence);
```

<div class="source-file">

*compiler.c*  
add after *parsePrecedence*()

</div>

``` insert
static uint8_t parseVariable(const char* errorMessage) {
  consume(TOKEN_IDENTIFIER, errorMessage);
  return identifierConstant(&parser.previous);
}
```

</div>

<div class="source-file-narrow">

*compiler.c*, add after *parsePrecedence*()

</div>

It requires the next token to be an identifier, which it consumes and
sends here:

<div class="codehilite">

``` insert-before
static void parsePrecedence(Precedence precedence);
```

<div class="source-file">

*compiler.c*  
add after *parsePrecedence*()

</div>

``` insert
static uint8_t identifierConstant(Token* name) {
  return makeConstant(OBJ_VAL(copyString(name->start,
                                         name->length)));
}
```

</div>

<div class="source-file-narrow">

*compiler.c*, add after *parsePrecedence*()

</div>

This function takes the given token and adds its lexeme to the chunk’s
constant table as a string. It then returns the index of that constant
in the constant table.

Global variables are looked up *by name* at runtime. That means the
VM<span class="em">—</span>the bytecode interpreter
loop<span class="em">—</span>needs access to the name. A whole string is
too big to stuff into the bytecode stream as an operand. Instead, we
store the string in the constant table and the instruction then refers
to the name by its index in the table.

This function returns that index all the way to `varDeclaration()` which
later hands it over to here:

<div class="codehilite">

<div class="source-file">

*compiler.c*  
add after *parseVariable*()

</div>

    static void defineVariable(uint8_t global) {
      emitBytes(OP_DEFINE_GLOBAL, global);
    }

</div>

<div class="source-file-narrow">

*compiler.c*, add after *parseVariable*()

</div>

<span id="helper">This</span> outputs the bytecode instruction that
defines the new variable and stores its initial value. The index of the
variable’s name in the constant table is the instruction’s operand. As
usual in a stack-based VM, we emit this instruction last. At runtime, we
execute the code for the variable’s initializer first. That leaves the
value on the stack. Then this instruction takes that value and stores it
away for later.

I know some of these functions seem pretty pointless right now. But
we’ll get more mileage out of them as we add more language features for
working with names. Function and class declarations both declare new
variables, and variable and assignment expressions access them.

Over in the runtime, we begin with this new instruction:

<div class="codehilite">

``` insert-before
  OP_POP,
```

<div class="source-file">

*chunk.h*  
in enum *OpCode*

</div>

``` insert
  OP_DEFINE_GLOBAL,
```

``` insert-after
  OP_EQUAL,
```

</div>

<div class="source-file-narrow">

*chunk.h*, in enum *OpCode*

</div>

Thanks to our handy-dandy hash table, the implementation isn’t too hard.

<div class="codehilite">

``` insert-before
      case OP_POP: pop(); break;
```

<div class="source-file">

*vm.c*  
in *run*()

</div>

``` insert
      case OP_DEFINE_GLOBAL: {
        ObjString* name = READ_STRING();
        tableSet(&vm.globals, name, peek(0));
        pop();
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

We get the name of the variable from the constant table. Then we
<span id="pop">take</span> the value from the top of the stack and store
it in a hash table with that name as the key.

Note that we don’t *pop* the value until *after* we add it to the hash
table. That ensures the VM can still find the value if a garbage
collection is triggered right in the middle of adding it to the hash
table. That’s a distinct possibility since the hash table requires
dynamic allocation when it resizes.

This code doesn’t check to see if the key is already in the table. Lox
is pretty lax with global variables and lets you redefine them without
error. That’s useful in a REPL session, so the VM supports that by
simply overwriting the value if the key happens to already be in the
hash table.

There’s another little helper macro:

<div class="codehilite">

``` insert-before
#define READ_CONSTANT() (vm.chunk->constants.values[READ_BYTE()])
```

<div class="source-file">

*vm.c*  
in *run*()

</div>

``` insert
#define READ_STRING() AS_STRING(READ_CONSTANT())
```

``` insert-after
#define BINARY_OP(valueType, op) \
```

</div>

<div class="source-file-narrow">

*vm.c*, in *run*()

</div>

It reads a one-byte operand from the bytecode chunk. It treats that as
an index into the chunk’s constant table and returns the string at that
index. It doesn’t check that the value *is* a
string<span class="em">—</span>it just indiscriminately casts it. That’s
safe because the compiler never emits an instruction that refers to a
non-string constant.

Because we care about lexical hygiene, we also undefine this macro at
the end of the interpret function.

<div class="codehilite">

``` insert-before
#undef READ_CONSTANT
```

<div class="source-file">

*vm.c*  
in *run*()

</div>

``` insert
#undef READ_STRING
```

``` insert-after
#undef BINARY_OP
```

</div>

<div class="source-file-narrow">

*vm.c*, in *run*()

</div>

I keep saying “the hash table”, but we don’t actually have one yet. We
need a place to store these globals. Since we want them to persist as
long as clox is running, we store them right in the VM.

<div class="codehilite">

``` insert-before
  Value* stackTop;
```

<div class="source-file">

*vm.h*  
in struct *VM*

</div>

``` insert
  Table globals;
```

``` insert-after
  Table strings;
```

</div>

<div class="source-file-narrow">

*vm.h*, in struct *VM*

</div>

As we did with the string table, we need to initialize the hash table to
a valid state when the VM boots up.

<div class="codehilite">

``` insert-before
  vm.objects = NULL;
```

<div class="source-file">

*vm.c*  
in *initVM*()

</div>

``` insert

  initTable(&vm.globals);
```

``` insert-after
  initTable(&vm.strings);
```

</div>

<div class="source-file-narrow">

*vm.c*, in *initVM*()

</div>

And we <span id="tear">tear</span> it down when we exit.

The process will free everything on exit, but it feels undignified to
require the operating system to clean up our mess.

<div class="codehilite">

``` insert-before
void freeVM() {
```

<div class="source-file">

*vm.c*  
in *freeVM*()

</div>

``` insert
  freeTable(&vm.globals);
```

``` insert-after
  freeTable(&vm.strings);
```

</div>

<div class="source-file-narrow">

*vm.c*, in *freeVM*()

</div>

As usual, we want to be able to disassemble the new instruction too.

<div class="codehilite">

``` insert-before
      return simpleInstruction("OP_POP", offset);
```

<div class="source-file">

*debug.c*  
in *disassembleInstruction*()

</div>

``` insert
    case OP_DEFINE_GLOBAL:
      return constantInstruction("OP_DEFINE_GLOBAL", chunk,
                                 offset);
```

``` insert-after
    case OP_EQUAL:
```

</div>

<div class="source-file-narrow">

*debug.c*, in *disassembleInstruction*()

</div>

And with that, we can define global variables. Not that users can *tell*
that they’ve done so, because they can’t actually *use* them. So let’s
fix that next.

## <a href="#reading-variables" id="reading-variables"><span
class="small">21 . 3</span>Reading Variables</a>

As in every programming language ever, we access a variable’s value
using its name. We hook up identifier tokens to the expression parser
here:

<div class="codehilite">

``` insert-before
  [TOKEN_LESS_EQUAL]    = {NULL,     binary, PREC_COMPARISON},
```

<div class="source-file">

*compiler.c*  
replace 1 line

</div>

``` insert
  [TOKEN_IDENTIFIER]    = {variable, NULL,   PREC_NONE},
```

``` insert-after
  [TOKEN_STRING]        = {string,   NULL,   PREC_NONE},
```

</div>

<div class="source-file-narrow">

*compiler.c*, replace 1 line

</div>

That calls this new parser function:

<div class="codehilite">

<div class="source-file">

*compiler.c*  
add after *string*()

</div>

    static void variable() {
      namedVariable(parser.previous);
    }

</div>

<div class="source-file-narrow">

*compiler.c*, add after *string*()

</div>

Like with declarations, there are a couple of tiny helper functions that
seem pointless now but will become more useful in later chapters. I
promise.

<div class="codehilite">

<div class="source-file">

*compiler.c*  
add after *string*()

</div>

    static void namedVariable(Token name) {
      uint8_t arg = identifierConstant(&name);
      emitBytes(OP_GET_GLOBAL, arg);
    }

</div>

<div class="source-file-narrow">

*compiler.c*, add after *string*()

</div>

This calls the same `identifierConstant()` function from before to take
the given identifier token and add its lexeme to the chunk’s constant
table as a string. All that remains is to emit an instruction that loads
the global variable with that name. Here’s the instruction:

<div class="codehilite">

``` insert-before
  OP_POP,
```

<div class="source-file">

*chunk.h*  
in enum *OpCode*

</div>

``` insert
  OP_GET_GLOBAL,
```

``` insert-after
  OP_DEFINE_GLOBAL,
```

</div>

<div class="source-file-narrow">

*chunk.h*, in enum *OpCode*

</div>

Over in the interpreter, the implementation mirrors `OP_DEFINE_GLOBAL`.

<div class="codehilite">

``` insert-before
      case OP_POP: pop(); break;
```

<div class="source-file">

*vm.c*  
in *run*()

</div>

``` insert
      case OP_GET_GLOBAL: {
        ObjString* name = READ_STRING();
        Value value;
        if (!tableGet(&vm.globals, name, &value)) {
          runtimeError("Undefined variable '%s'.", name->chars);
          return INTERPRET_RUNTIME_ERROR;
        }
        push(value);
        break;
      }
```

``` insert-after
      case OP_DEFINE_GLOBAL: {
```

</div>

<div class="source-file-narrow">

*vm.c*, in *run*()

</div>

We pull the constant table index from the instruction’s operand and get
the variable name. Then we use that as a key to look up the variable’s
value in the globals hash table.

If the key isn’t present in the hash table, it means that global
variable has never been defined. That’s a runtime error in Lox, so we
report it and exit the interpreter loop if that happens. Otherwise, we
take the value and push it onto the stack.

<div class="codehilite">

``` insert-before
      return simpleInstruction("OP_POP", offset);
```

<div class="source-file">

*debug.c*  
in *disassembleInstruction*()

</div>

``` insert
    case OP_GET_GLOBAL:
      return constantInstruction("OP_GET_GLOBAL", chunk, offset);
```

``` insert-after
    case OP_DEFINE_GLOBAL:
```

</div>

<div class="source-file-narrow">

*debug.c*, in *disassembleInstruction*()

</div>

A little bit of disassembling, and we’re done. Our interpreter is now
able to run code like this:

<div class="codehilite">

    var beverage = "cafe au lait";
    var breakfast = "beignets with " + beverage;
    print breakfast;

</div>

There’s only one operation left.

## <a href="#assignment" id="assignment"><span
class="small">21 . 4</span>Assignment</a>

Throughout this book, I’ve tried to keep you on a fairly safe and easy
path. I don’t avoid hard *problems*, but I try to not make the
*solutions* more complex than they need to be. Alas, other design
choices in our <span id="jlox">bytecode</span> compiler make assignment
annoying to implement.

If you recall, assignment was pretty easy in jlox.

Our bytecode VM uses a single-pass compiler. It parses and generates
bytecode on the fly without any intermediate AST. As soon as it
recognizes a piece of syntax, it emits code for it. Assignment doesn’t
naturally fit that. Consider:

<div class="codehilite">

    menu.brunch(sunday).beverage = "mimosa";

</div>

In this code, the parser doesn’t realize `menu.brunch(sunday).beverage`
is the target of an assignment and not a normal expression until it
reaches `=`, many tokens after the first `menu`. By then, the compiler
has already emitted bytecode for the whole thing.

The problem is not as dire as it might seem, though. Look at how the
parser sees that example:

![The 'menu.brunch(sunday).beverage = "mimosa"' statement, showing that
'menu.brunch(sunday)' is an
expression.](image/global-variables/setter.png)

Even though the `.beverage` part must not be compiled as a get
expression, everything to the left of the `.` is an expression, with the
normal expression semantics. The `menu.brunch(sunday)` part can be
compiled and executed as usual.

Fortunately for us, the only semantic differences on the left side of an
assignment appear at the very right-most end of the tokens, immediately
preceding the `=`. Even though the receiver of a setter may be an
arbitrarily long expression, the part whose behavior differs from a get
expression is only the trailing identifier, which is right before the
`=`. We don’t need much lookahead to realize `beverage` should be
compiled as a set expression and not a getter.

Variables are even easier since they are just a single bare identifier
before an `=`. The idea then is that right *before* compiling an
expression that can also be used as an assignment target, we look for a
subsequent `=` token. If we see one, we compile it as an assignment or
setter instead of a variable access or getter.

We don’t have setters to worry about yet, so all we need to handle are
variables.

<div class="codehilite">

``` insert-before
  uint8_t arg = identifierConstant(&name);
```

<div class="source-file">

*compiler.c*  
in *namedVariable*()  
replace 1 line

</div>

``` insert

  if (match(TOKEN_EQUAL)) {
    expression();
    emitBytes(OP_SET_GLOBAL, arg);
  } else {
    emitBytes(OP_GET_GLOBAL, arg);
  }
```

``` insert-after
}
```

</div>

<div class="source-file-narrow">

*compiler.c*, in *namedVariable*(), replace 1 line

</div>

In the parse function for identifier expressions, we look for an equals
sign after the identifier. If we find one, instead of emitting code for
a variable access, we compile the assigned value and then emit an
assignment instruction.

That’s the last instruction we need to add in this chapter.

<div class="codehilite">

``` insert-before
  OP_DEFINE_GLOBAL,
```

<div class="source-file">

*chunk.h*  
in enum *OpCode*

</div>

``` insert
  OP_SET_GLOBAL,
```

``` insert-after
  OP_EQUAL,
```

</div>

<div class="source-file-narrow">

*chunk.h*, in enum *OpCode*

</div>

As you’d expect, its runtime behavior is similar to defining a new
variable.

<div class="codehilite">

``` insert-before
      }
```

<div class="source-file">

*vm.c*  
in *run*()

</div>

``` insert
      case OP_SET_GLOBAL: {
        ObjString* name = READ_STRING();
        if (tableSet(&vm.globals, name, peek(0))) {
          tableDelete(&vm.globals, name); 
          runtimeError("Undefined variable '%s'.", name->chars);
          return INTERPRET_RUNTIME_ERROR;
        }
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

The main difference is what happens when the key doesn’t already exist
in the globals hash table. If the variable hasn’t been defined yet, it’s
a runtime error to try to assign to it. Lox [doesn’t do implicit
variable declaration](statements-and-state.html#design-note).

The call to `tableSet()` stores the value in the global variable table
even if the variable wasn’t previously defined. That fact is visible in
a REPL session, since it keeps running even after the runtime error is
reported. So we also take care to delete that zombie value from the
table.

The other difference is that setting a variable doesn’t pop the value
off the stack. Remember, assignment is an expression, so it needs to
leave that value there in case the assignment is nested inside some
larger expression.

Add a dash of disassembly:

<div class="codehilite">

``` insert-before
      return constantInstruction("OP_DEFINE_GLOBAL", chunk,
                                 offset);
```

<div class="source-file">

*debug.c*  
in *disassembleInstruction*()

</div>

``` insert
    case OP_SET_GLOBAL:
      return constantInstruction("OP_SET_GLOBAL", chunk, offset);
```

``` insert-after
    case OP_EQUAL:
```

</div>

<div class="source-file-narrow">

*debug.c*, in *disassembleInstruction*()

</div>

So we’re done, right? Well<span class="ellipse"> . . . </span>not quite.
We’ve made a mistake! Take a gander at:

<div class="codehilite">

    a * b = c + d;

</div>

According to Lox’s grammar, `=` has the lowest precedence, so this
should be parsed roughly like:

![The expected parse, like '(a \* b) = (c +
d)'.](image/global-variables/ast-good.png)

Obviously, `a * b` isn’t a <span id="do">valid</span> assignment target,
so this should be a syntax error. But here’s what our parser does:

Wouldn’t it be wild if `a * b` *was* a valid assignment target, though?
You could imagine some algebra-like language that tried to divide the
assigned value up in some reasonable way and distribute it to `a` and
`b`<span class="ellipse"> . . . </span>that’s probably a terrible idea.

1.  First, `parsePrecedence()` parses `a` using the `variable()` prefix
    parser.
2.  After that, it enters the infix parsing loop.
3.  It reaches the `*` and calls `binary()`.
4.  That recursively calls `parsePrecedence()` to parse the right-hand
    operand.
5.  That calls `variable()` again for parsing `b`.
6.  Inside that call to `variable()`, it looks for a trailing `=`. It
    sees one and thus parses the rest of the line as an assignment.

In other words, the parser sees the above code like:

![The actual parse, like 'a \* (b = c +
d)'.](image/global-variables/ast-bad.png)

We’ve messed up the precedence handling because `variable()` doesn’t
take into account the precedence of the surrounding expression that
contains the variable. If the variable happens to be the right-hand side
of an infix operator, or the operand of a unary operator, then that
containing expression is too high precedence to permit the `=`.

To fix this, `variable()` should look for and consume the `=` only if
it’s in the context of a low-precedence expression. The code that knows
the current precedence is, logically enough, `parsePrecedence()`. The
`variable()` function doesn’t need to know the actual level. It just
cares that the precedence is low enough to allow assignment, so we pass
that fact in as a Boolean.

<div class="codehilite">

``` insert-before
    error("Expect expression.");
    return;
  }
```

<div class="source-file">

*compiler.c*  
in *parsePrecedence*()  
replace 1 line

</div>

``` insert
  bool canAssign = precedence <= PREC_ASSIGNMENT;
  prefixRule(canAssign);
```

``` insert-after

  while (precedence <= getRule(parser.current.type)->precedence) {
```

</div>

<div class="source-file-narrow">

*compiler.c*, in *parsePrecedence*(), replace 1 line

</div>

Since assignment is the lowest-precedence expression, the only time we
allow an assignment is when parsing an assignment expression or
top-level expression like in an expression statement. That flag makes
its way to the parser function here:

<div class="codehilite">

<div class="source-file">

*compiler.c*  
function *variable*()  
replace 3 lines

</div>

    static void variable(bool canAssign) {
      namedVariable(parser.previous, canAssign);
    }

</div>

<div class="source-file-narrow">

*compiler.c*, function *variable*(), replace 3 lines

</div>

Which passes it through a new parameter:

<div class="codehilite">

<div class="source-file">

*compiler.c*  
function *namedVariable*()  
replace 1 line

</div>

``` insert
static void namedVariable(Token name, bool canAssign) {
```

``` insert-after
  uint8_t arg = identifierConstant(&name);
```

</div>

<div class="source-file-narrow">

*compiler.c*, function *namedVariable*(), replace 1 line

</div>

And then finally uses it here:

<div class="codehilite">

``` insert-before
  uint8_t arg = identifierConstant(&name);
```

<div class="source-file">

*compiler.c*  
in *namedVariable*()  
replace 1 line

</div>

``` insert
  if (canAssign && match(TOKEN_EQUAL)) {
```

``` insert-after
    expression();
```

</div>

<div class="source-file-narrow">

*compiler.c*, in *namedVariable*(), replace 1 line

</div>

That’s a lot of plumbing to get literally one bit of data to the right
place in the compiler, but arrived it has. If the variable is nested
inside some expression with higher precedence, `canAssign` will be
`false` and this will ignore the `=` even if there is one there. Then
`namedVariable()` returns, and execution eventually makes its way back
to `parsePrecedence()`.

Then what? What does the compiler do with our broken example from
before? Right now, `variable()` won’t consume the `=`, so that will be
the current token. The compiler returns back to `parsePrecedence()` from
the `variable()` prefix parser and then tries to enter the infix parsing
loop. There is no parsing function associated with `=`, so it skips that
loop.

Then `parsePrecedence()` silently returns back to the caller. That also
isn’t right. If the `=` doesn’t get consumed as part of the expression,
nothing else is going to consume it. It’s an error and we should report
it.

<div class="codehilite">

``` insert-before
    infixRule();
  }
```

<div class="source-file">

*compiler.c*  
in *parsePrecedence*()

</div>

``` insert

  if (canAssign && match(TOKEN_EQUAL)) {
    error("Invalid assignment target.");
  }
```

``` insert-after
}
```

</div>

<div class="source-file-narrow">

*compiler.c*, in *parsePrecedence*()

</div>

With that, the previous bad program correctly gets an error at compile
time. OK, *now* are we done? Still not quite. See, we’re passing an
argument to one of the parse functions. But those functions are stored
in a table of function pointers, so all of the parse functions need to
have the same type. Even though most parse functions don’t support being
used as an assignment target<span class="em">—</span>setters are the
<span id="index">only</span> other one<span class="em">—</span>our
friendly C compiler requires them *all* to accept the parameter.

If Lox had arrays and subscript operators like `array[index]` then an
infix `[` would also allow assignment to support `array[index] = value`.

So we’re going to finish off this chapter with some grunt work. First,
let’s go ahead and pass the flag to the infix parse functions.

<div class="codehilite">

``` insert-before
    ParseFn infixRule = getRule(parser.previous.type)->infix;
```

<div class="source-file">

*compiler.c*  
in *parsePrecedence*()  
replace 1 line

</div>

``` insert
    infixRule(canAssign);
```

``` insert-after
  }
```

</div>

<div class="source-file-narrow">

*compiler.c*, in *parsePrecedence*(), replace 1 line

</div>

We’ll need that for setters eventually. Then we’ll fix the typedef for
the function type.

<div class="codehilite">

``` insert-before
} Precedence;
```

<div class="source-file">

*compiler.c*  
add after enum *Precedence*  
replace 1 line

</div>

``` insert
typedef void (*ParseFn)(bool canAssign);
```

``` insert-after

typedef struct {
```

</div>

<div class="source-file-narrow">

*compiler.c*, add after enum *Precedence*, replace 1 line

</div>

And some completely tedious code to accept this parameter in all of our
existing parse functions. Here:

<div class="codehilite">

<div class="source-file">

*compiler.c*  
function *binary*()  
replace 1 line

</div>

``` insert
static void binary(bool canAssign) {
```

``` insert-after
  TokenType operatorType = parser.previous.type;
```

</div>

<div class="source-file-narrow">

*compiler.c*, function *binary*(), replace 1 line

</div>

And here:

<div class="codehilite">

<div class="source-file">

*compiler.c*  
function *literal*()  
replace 1 line

</div>

``` insert
static void literal(bool canAssign) {
```

``` insert-after
  switch (parser.previous.type) {
```

</div>

<div class="source-file-narrow">

*compiler.c*, function *literal*(), replace 1 line

</div>

And here:

<div class="codehilite">

<div class="source-file">

*compiler.c*  
function *grouping*()  
replace 1 line

</div>

``` insert
static void grouping(bool canAssign) {
```

``` insert-after
  expression();
```

</div>

<div class="source-file-narrow">

*compiler.c*, function *grouping*(), replace 1 line

</div>

And here:

<div class="codehilite">

<div class="source-file">

*compiler.c*  
function *number*()  
replace 1 line

</div>

``` insert
static void number(bool canAssign) {
```

``` insert-after
  double value = strtod(parser.previous.start, NULL);
```

</div>

<div class="source-file-narrow">

*compiler.c*, function *number*(), replace 1 line

</div>

And here too:

<div class="codehilite">

<div class="source-file">

*compiler.c*  
function *string*()  
replace 1 line

</div>

``` insert
static void string(bool canAssign) {
```

``` insert-after
  emitConstant(OBJ_VAL(copyString(parser.previous.start + 1,
```

</div>

<div class="source-file-narrow">

*compiler.c*, function *string*(), replace 1 line

</div>

And, finally:

<div class="codehilite">

<div class="source-file">

*compiler.c*  
function *unary*()  
replace 1 line

</div>

``` insert
static void unary(bool canAssign) {
```

``` insert-after
  TokenType operatorType = parser.previous.type;
```

</div>

<div class="source-file-narrow">

*compiler.c*, function *unary*(), replace 1 line

</div>

Phew! We’re back to a C program we can compile. Fire it up and now you
can run this:

<div class="codehilite">

    var breakfast = "beignets";
    var beverage = "cafe au lait";
    breakfast = "beignets with " + beverage;

    print breakfast;

</div>

It’s starting to look like real code for an actual language!

<div class="challenges">

## <a href="#challenges" id="challenges">Challenges</a>

1.  The compiler adds a global variable’s name to the constant table as
    a string every time an identifier is encountered. It creates a new
    constant each time, even if that variable name is already in a
    previous slot in the constant table. That’s wasteful in cases where
    the same variable is referenced multiple times by the same function.
    That, in turn, increases the odds of filling up the constant table
    and running out of slots since we allow only 256 constants in a
    single chunk.

    Optimize this. How does your optimization affect the performance of
    the compiler compared to the runtime? Is this the right trade-off?

2.  Looking up a global variable by name in a hash table each time it is
    used is pretty slow, even with a good hash table. Can you come up
    with a more efficient way to store and access global variables
    without changing the semantics?

3.  When running in the REPL, a user might write a function that
    references an unknown global variable. Then, in the next line, they
    declare the variable. Lox should handle this gracefully by not
    reporting an “unknown variable” compile error when the function is
    first defined.

    But when a user runs a Lox *script*, the compiler has access to the
    full text of the entire program before any code is run. Consider
    this program:

    <div class="codehilite">

        fun useVar() {
          print oops;
        }

        var ooops = "too many o's!";

    </div>

    Here, we can tell statically that `oops` will not be defined because
    there is *no* declaration of that global anywhere in the program.
    Note that `useVar()` is never called either, so even though the
    variable isn’t defined, no runtime error will occur because it’s
    never used either.

    We could report mistakes like this as compile errors, at least when
    running from a script. Do you think we should? Justify your answer.
    What do other scripting languages you know do?

</div>

<a href="local-variables.html" class="next">Next Chapter: “Local
Variables” →</a> Handcrafted by Robert Nystrom — <a
href="https://github.com/munificent/craftinginterpreters/blob/master/LICENSE"
target="_blank">© 2015 – 2021</a>

</div>

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

[![](image/logotype.png "Crafting Interpreters")](/)

<div class="contents">

### [Jumping Back and Forth<span class="small">23</span>](#top)

- [<span class="small">23.1</span> If Statements](#if-statements)
- [<span class="small">23.2</span> Logical
  Operators](#logical-operators)
- [<span class="small">23.3</span> While Statements](#while-statements)
- [<span class="small">23.4</span> For Statements](#for-statements)
- 
- [Challenges](#challenges)
- [<span class="small">note</span>Considering Goto
  Harmful](#design-note)

<div class="prev-next">

<a href="local-variables.html" class="left"
title="Local Variables">← Previous</a>
[↑ Up](a-bytecode-virtual-machine.html "A Bytecode Virtual Machine")
<a href="calls-and-functions.html" class="right"
title="Calls and Functions">Next →</a>

</div>

</div>

[![](image/logotype.png "Crafting Interpreters")](/)
<a href="local-variables.html" class="prev"
title="Local Variables">←</a>
<a href="calls-and-functions.html" class="next"
title="Calls and Functions">→</a>

<div class="page">

<div class="nav-wrapper">

[![](image/logotype.png "Crafting Interpreters")](/)

<div class="expandable">

### [Jumping Back and Forth<span class="small">23</span>](#top)

- [<span class="small">23.1</span> If Statements](#if-statements)
- [<span class="small">23.2</span> Logical
  Operators](#logical-operators)
- [<span class="small">23.3</span> While Statements](#while-statements)
- [<span class="small">23.4</span> For Statements](#for-statements)
- 
- [Challenges](#challenges)
- [<span class="small">note</span>Considering Goto
  Harmful](#design-note)

<div class="prev-next">

<a href="local-variables.html" class="left"
title="Local Variables">← Previous</a>
[↑ Up](a-bytecode-virtual-machine.html "A Bytecode Virtual Machine")
<a href="calls-and-functions.html" class="right"
title="Calls and Functions">Next →</a>

</div>

</div>

<span id="expand-nav">≡</span>

</div>

<div class="number">

23

</div>

# Jumping Back and Forth

> The order that our mind imagines is like a net, or like a ladder,
> built to attain something. But afterward you must throw the ladder
> away, because you discover that, even if it was useful, it was
> meaningless.
>
> Umberto Eco, *The Name of the Rose*

It’s taken a while to get here, but we’re finally ready to add control
flow to our virtual machine. In the tree-walk interpreter we built for
jlox, we implemented Lox’s control flow in terms of Java’s. To execute a
Lox `if` statement, we used a Java `if` statement to run the chosen
branch. That works, but isn’t entirely satisfying. By what magic does
the *JVM itself* or a native CPU implement `if` statements? Now that we
have our own bytecode VM to hack on, we can answer that.

When we talk about “control flow”, what are we referring to? By “flow”
we mean the way execution moves through the text of the program. Almost
like there is a little robot inside the computer wandering through our
code, executing bits and pieces here and there. Flow is the path that
robot takes, and by *controlling* the robot, we drive which pieces of
code it executes.

In jlox, the robot’s locus of attention<span class="em">—</span>the
*current* bit of code<span class="em">—</span>was implicit based on
which AST nodes were stored in various Java variables and what Java code
we were in the middle of running. In clox, it is much more explicit. The
VM’s `ip` field stores the address of the current bytecode instruction.
The value of that field is exactly “where we are” in the program.

Execution proceeds normally by incrementing the `ip`. But we can mutate
that variable however we want to. In order to implement control flow,
all that’s necessary is to change the `ip` in more interesting ways. The
simplest control flow construct is an `if` statement with no `else`
clause:

<div class="codehilite">

    if (condition) print("condition was truthy");

</div>

The VM evaluates the bytecode for the condition expression. If the
result is truthy, then it continues along and executes the `print`
statement in the body. The interesting case is when the condition is
falsey. When that happens, execution skips over the then branch and
proceeds to the next statement.

To skip over a chunk of code, we simply set the `ip` field to the
address of the bytecode instruction following that code. To
*conditionally* skip over some code, we need an instruction that looks
at the value on top of the stack. If it’s falsey, it adds a given offset
to the `ip` to jump over a range of instructions. Otherwise, it does
nothing and lets execution proceed to the next instruction as usual.

When we compile to bytecode, the explicit nested block structure of the
code evaporates, leaving only a flat series of instructions behind. Lox
is a [structured
programming](https://en.wikipedia.org/wiki/Structured_programming)
language, but clox bytecode isn’t. The right<span class="em">—</span>or
wrong, depending on how you look at it<span class="em">—</span>set of
bytecode instructions could jump into the middle of a block, or from one
scope into another.

The VM will happily execute that, even if the result leaves the stack in
an unknown, inconsistent state. So even though the bytecode is
unstructured, we’ll take care to ensure that our compiler only generates
clean code that maintains the same structure and nesting that Lox itself
does.

This is exactly how real CPUs behave. Even though we might program them
using higher-level languages that mandate structured control flow, the
compiler lowers that down to raw jumps. At the bottom, it turns out goto
is the only real control flow.

Anyway, I didn’t mean to get all philosophical. The important bit is
that if we have that one conditional jump instruction, that’s enough to
implement Lox’s `if` statement, as long as it doesn’t have an `else`
clause. So let’s go ahead and get started with that.

## <a href="#if-statements" id="if-statements"><span
class="small">23 . 1</span>If Statements</a>

This many chapters in, you know the drill. Any new feature starts in the
front end and works its way through the pipeline. An `if` statement is,
well, a statement, so that’s where we hook it into the parser.

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
  } else if (match(TOKEN_IF)) {
    ifStatement();
```

``` insert-after
  } else if (match(TOKEN_LEFT_BRACE)) {
```

</div>

<div class="source-file-narrow">

*compiler.c*, in *statement*()

</div>

When we see an `if` keyword, we hand off compilation to this function:

<div class="codehilite">

<div class="source-file">

*compiler.c*  
add after *expressionStatement*()

</div>

    static void ifStatement() {
      consume(TOKEN_LEFT_PAREN, "Expect '(' after 'if'.");
      expression();
      consume(TOKEN_RIGHT_PAREN, "Expect ')' after condition."); 

      int thenJump = emitJump(OP_JUMP_IF_FALSE);
      statement();

      patchJump(thenJump);
    }

</div>

<div class="source-file-narrow">

*compiler.c*, add after *expressionStatement*()

</div>

Have you ever noticed that the `(` after the `if` keyword doesn’t
actually do anything useful? The language would be just as unambiguous
and easy to parse without it, like:

<div class="codehilite">

    if condition) print("looks weird");

</div>

The closing `)` is useful because it separates the condition expression
from the body. Some languages use a `then` keyword instead. But the
opening `(` doesn’t do anything. It’s just there because unmatched
parentheses look bad to us humans.

First we compile the condition expression, bracketed by parentheses. At
runtime, that will leave the condition value on top of the stack. We’ll
use that to determine whether to execute the then branch or skip it.

Then we emit a new `OP_JUMP_IF_FALSE` instruction. It has an operand for
how much to offset the `ip`<span class="em">—</span>how many bytes of
code to skip. If the condition is falsey, it adjusts the `ip` by that
amount. Something like this:

The boxes with the torn edges here represent the blob of bytecode
generated by compiling some sub-clause of a control flow construct. So
the “condition expression” box is all of the instructions emitted when
we compiled that expression.

<span id="legend"></span>

![Flowchart of the compiled bytecode of an if
statement.](image/jumping-back-and-forth/if-without-else.png)

But we have a problem. When we’re writing the `OP_JUMP_IF_FALSE`
instruction’s operand, how do we know how far to jump? We haven’t
compiled the then branch yet, so we don’t know how much bytecode it
contains.

To fix that, we use a classic trick called **backpatching**. We emit the
jump instruction first with a placeholder offset operand. We keep track
of where that half-finished instruction is. Next, we compile the then
body. Once that’s done, we know how far to jump. So we go back and
replace that placeholder offset with the real one now that we can
calculate it. Sort of like sewing a patch onto the existing fabric of
the compiled code.

![A patch containing a number being sewn onto a sheet of
bytecode.](image/jumping-back-and-forth/patch.png)

We encode this trick into two helper functions.

<div class="codehilite">

<div class="source-file">

*compiler.c*  
add after *emitBytes*()

</div>

    static int emitJump(uint8_t instruction) {
      emitByte(instruction);
      emitByte(0xff);
      emitByte(0xff);
      return currentChunk()->count - 2;
    }

</div>

<div class="source-file-narrow">

*compiler.c*, add after *emitBytes*()

</div>

The first emits a bytecode instruction and writes a placeholder operand
for the jump offset. We pass in the opcode as an argument because later
we’ll have two different instructions that use this helper. We use two
bytes for the jump offset operand. A 16-bit
<span id="offset">offset</span> lets us jump over up to 65,535 bytes of
code, which should be plenty for our needs.

Some instruction sets have separate “long” jump instructions that take
larger operands for when you need to jump a greater distance.

The function returns the offset of the emitted instruction in the chunk.
After compiling the then branch, we take that offset and pass it to
this:

<div class="codehilite">

<div class="source-file">

*compiler.c*  
add after *emitConstant*()

</div>

    static void patchJump(int offset) {
      // -2 to adjust for the bytecode for the jump offset itself.
      int jump = currentChunk()->count - offset - 2;

      if (jump > UINT16_MAX) {
        error("Too much code to jump over.");
      }

      currentChunk()->code[offset] = (jump >> 8) & 0xff;
      currentChunk()->code[offset + 1] = jump & 0xff;
    }

</div>

<div class="source-file-narrow">

*compiler.c*, add after *emitConstant*()

</div>

This goes back into the bytecode and replaces the operand at the given
location with the calculated jump offset. We call `patchJump()` right
before we emit the next instruction that we want the jump to land on, so
it uses the current bytecode count to determine how far to jump. In the
case of an `if` statement, that means right after we compile the then
branch and before we compile the next statement.

That’s all we need at compile time. Let’s define the new instruction.

<div class="codehilite">

``` insert-before
  OP_PRINT,
```

<div class="source-file">

*chunk.h*  
in enum *OpCode*

</div>

``` insert
  OP_JUMP_IF_FALSE,
```

``` insert-after
  OP_RETURN,
```

</div>

<div class="source-file-narrow">

*chunk.h*, in enum *OpCode*

</div>

Over in the VM, we get it working like so:

<div class="codehilite">

``` insert-before
        break;
      }
```

<div class="source-file">

*vm.c*  
in *run*()

</div>

``` insert
      case OP_JUMP_IF_FALSE: {
        uint16_t offset = READ_SHORT();
        if (isFalsey(peek(0))) vm.ip += offset;
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

This is the first instruction we’ve added that takes a 16-bit operand.
To read that from the chunk, we use a new macro.

<div class="codehilite">

``` insert-before
#define READ_CONSTANT() (vm.chunk->constants.values[READ_BYTE()])
```

<div class="source-file">

*vm.c*  
in *run*()

</div>

``` insert
#define READ_SHORT() \
    (vm.ip += 2, (uint16_t)((vm.ip[-2] << 8) | vm.ip[-1]))
```

``` insert-after
#define READ_STRING() AS_STRING(READ_CONSTANT())
```

</div>

<div class="source-file-narrow">

*vm.c*, in *run*()

</div>

It yanks the next two bytes from the chunk and builds a 16-bit unsigned
integer out of them. As usual, we clean up our macro when we’re done
with it.

<div class="codehilite">

``` insert-before
#undef READ_BYTE
```

<div class="source-file">

*vm.c*  
in *run*()

</div>

``` insert
#undef READ_SHORT
```

``` insert-after
#undef READ_CONSTANT
```

</div>

<div class="source-file-narrow">

*vm.c*, in *run*()

</div>

After reading the offset, we check the condition value on top of the
stack. <span id="if">If</span> it’s falsey, we apply this jump offset to
the `ip`. Otherwise, we leave the `ip` alone and execution will
automatically proceed to the next instruction following the jump
instruction.

In the case where the condition is falsey, we don’t need to do any other
work. We’ve offset the `ip`, so when the outer instruction dispatch loop
turns again, it will pick up execution at that new instruction, past all
of the code in the then branch.

I said we wouldn’t use C’s `if` statement to implement Lox’s control
flow, but we do use one here to determine whether or not to offset the
instruction pointer. But we aren’t really using C for *control flow*. If
we wanted to, we could do the same thing purely arithmetically. Let’s
assume we have a function `falsey()` that takes a Lox Value and returns
1 if it’s falsey or 0 otherwise. Then we could implement the jump
instruction like:

<div class="codehilite">

    case OP_JUMP_IF_FALSE: {
      uint16_t offset = READ_SHORT();
      vm.ip += falsey() * offset;
      break;
    }

</div>

The `falsey()` function would probably use some control flow to handle
the different value types, but that’s an implementation detail of that
function and doesn’t affect how our VM does its own control flow.

Note that the jump instruction doesn’t pop the condition value off the
stack. So we aren’t totally done here, since this leaves an extra value
floating around on the stack. We’ll clean that up soon. Ignoring that
for the moment, we do have a working `if` statement in Lox now, with
only one little instruction required to support it at runtime in the VM.

### <a href="#else-clauses" id="else-clauses"><span
class="small">23 . 1 . 1</span>Else clauses</a>

An `if` statement without support for `else` clauses is like Morticia
Addams without Gomez. So, after we compile the then branch, we look for
an `else` keyword. If we find one, we compile the else branch.

<div class="codehilite">

``` insert-before
  patchJump(thenJump);
```

<div class="source-file">

*compiler.c*  
in *ifStatement*()

</div>

``` insert

  if (match(TOKEN_ELSE)) statement();
```

``` insert-after
}
```

</div>

<div class="source-file-narrow">

*compiler.c*, in *ifStatement*()

</div>

When the condition is falsey, we’ll jump over the then branch. If
there’s an else branch, the `ip` will land right at the beginning of its
code. But that’s not enough, though. Here’s the flow that leads to:

![Flowchart of the compiled bytecode with the then branch incorrectly
falling through to the else
branch.](image/jumping-back-and-forth/bad-else.png)

If the condition is truthy, we execute the then branch like we want. But
after that, execution rolls right on through into the else branch. Oops!
When the condition is true, after we run the then branch, we need to
jump over the else branch. That way, in either case, we only execute a
single branch, like this:

![Flowchart of the compiled bytecode for an if with an else
clause.](image/jumping-back-and-forth/if-else.png)

To implement that, we need another jump from the end of the then branch.

<div class="codehilite">

``` insert-before
  statement();
```

<div class="source-file">

*compiler.c*  
in *ifStatement*()

</div>

``` insert
  int elseJump = emitJump(OP_JUMP);
```

``` insert-after
  patchJump(thenJump);
```

</div>

<div class="source-file-narrow">

*compiler.c*, in *ifStatement*()

</div>

We patch that offset after the end of the else body.

<div class="codehilite">

``` insert-before
  if (match(TOKEN_ELSE)) statement();
```

<div class="source-file">

*compiler.c*  
in *ifStatement*()

</div>

``` insert
  patchJump(elseJump);
```

``` insert-after
}
```

</div>

<div class="source-file-narrow">

*compiler.c*, in *ifStatement*()

</div>

After executing the then branch, this jumps to the next statement after
the else branch. Unlike the other jump, this jump is unconditional. We
always take it, so we need another instruction that expresses that.

<div class="codehilite">

``` insert-before
  OP_PRINT,
```

<div class="source-file">

*chunk.h*  
in enum *OpCode*

</div>

``` insert
  OP_JUMP,
```

``` insert-after
  OP_JUMP_IF_FALSE,
```

</div>

<div class="source-file-narrow">

*chunk.h*, in enum *OpCode*

</div>

We interpret it like so:

<div class="codehilite">

``` insert-before
        break;
      }
```

<div class="source-file">

*vm.c*  
in *run*()

</div>

``` insert
      case OP_JUMP: {
        uint16_t offset = READ_SHORT();
        vm.ip += offset;
        break;
      }
```

``` insert-after
      case OP_JUMP_IF_FALSE: {
```

</div>

<div class="source-file-narrow">

*vm.c*, in *run*()

</div>

Nothing too surprising here<span class="em">—</span>the only difference
is that it doesn’t check a condition and always applies the offset.

We have then and else branches working now, so we’re close. The last bit
is to clean up that condition value we left on the stack. Remember, each
statement is required to have zero stack
effect<span class="em">—</span>after the statement is finished
executing, the stack should be as tall as it was before.

We could have the `OP_JUMP_IF_FALSE` instruction pop the condition
itself, but soon we’ll use that same instruction for the logical
operators where we don’t want the condition popped. Instead, we’ll have
the compiler emit a couple of explicit `OP_POP` instructions when
compiling an `if` statement. We need to take care that every execution
path through the generated code pops the condition.

When the condition is truthy, we pop it right before the code inside the
then branch.

<div class="codehilite">

``` insert-before
  int thenJump = emitJump(OP_JUMP_IF_FALSE);
```

<div class="source-file">

*compiler.c*  
in *ifStatement*()

</div>

``` insert
  emitByte(OP_POP);
```

``` insert-after
  statement();
```

</div>

<div class="source-file-narrow">

*compiler.c*, in *ifStatement*()

</div>

Otherwise, we pop it at the beginning of the else branch.

<div class="codehilite">

``` insert-before
  patchJump(thenJump);
```

<div class="source-file">

*compiler.c*  
in *ifStatement*()

</div>

``` insert
  emitByte(OP_POP);
```

``` insert-after

  if (match(TOKEN_ELSE)) statement();
```

</div>

<div class="source-file-narrow">

*compiler.c*, in *ifStatement*()

</div>

This little instruction here also means that every `if` statement has an
implicit else branch even if the user didn’t write an `else` clause. In
the case where they left it off, all the branch does is discard the
condition value.

The full correct flow looks like this:

![Flowchart of the compiled bytecode including necessary pop
instructions.](image/jumping-back-and-forth/full-if-else.png)

If you trace through, you can see that it always executes a single
branch and ensures the condition is popped first. All that remains is a
little disassembler support.

<div class="codehilite">

``` insert-before
      return simpleInstruction("OP_PRINT", offset);
```

<div class="source-file">

*debug.c*  
in *disassembleInstruction*()

</div>

``` insert
    case OP_JUMP:
      return jumpInstruction("OP_JUMP", 1, chunk, offset);
    case OP_JUMP_IF_FALSE:
      return jumpInstruction("OP_JUMP_IF_FALSE", 1, chunk, offset);
```

``` insert-after
    case OP_RETURN:
```

</div>

<div class="source-file-narrow">

*debug.c*, in *disassembleInstruction*()

</div>

These two instructions have a new format with a 16-bit operand, so we
add a new utility function to disassemble them.

<div class="codehilite">

<div class="source-file">

*debug.c*  
add after *byteInstruction*()

</div>

    static int jumpInstruction(const char* name, int sign,
                               Chunk* chunk, int offset) {
      uint16_t jump = (uint16_t)(chunk->code[offset + 1] << 8);
      jump |= chunk->code[offset + 2];
      printf("%-16s %4d -> %d\n", name, offset,
             offset + 3 + sign * jump);
      return offset + 3;
    }

</div>

<div class="source-file-narrow">

*debug.c*, add after *byteInstruction*()

</div>

There we go, that’s one complete control flow construct. If this were an
’80s movie, the montage music would kick in and the rest of the control
flow syntax would take care of itself. Alas, the
<span id="80s">’80s</span> are long over, so we’ll have to grind it out
ourselves.

My enduring love of Depeche Mode notwithstanding.

## <a href="#logical-operators" id="logical-operators"><span
class="small">23 . 2</span>Logical Operators</a>

You probably remember this from jlox, but the logical operators `and`
and `or` aren’t just another pair of binary operators like `+` and `-`.
Because they short-circuit and may not evaluate their right operand
depending on the value of the left one, they work more like control flow
expressions.

They’re basically a little variation on an `if` statement with an `else`
clause. The easiest way to explain them is to just show you the compiler
code and the control flow it produces in the resulting bytecode.
Starting with `and`, we hook it into the expression parsing table here:

<div class="codehilite">

``` insert-before
  [TOKEN_NUMBER]        = {number,   NULL,   PREC_NONE},
```

<div class="source-file">

*compiler.c*  
replace 1 line

</div>

``` insert
  [TOKEN_AND]           = {NULL,     and_,   PREC_AND},
```

``` insert-after
  [TOKEN_CLASS]         = {NULL,     NULL,   PREC_NONE},
```

</div>

<div class="source-file-narrow">

*compiler.c*, replace 1 line

</div>

That hands off to a new parser function.

<div class="codehilite">

<div class="source-file">

*compiler.c*  
add after *defineVariable*()

</div>

    static void and_(bool canAssign) {
      int endJump = emitJump(OP_JUMP_IF_FALSE);

      emitByte(OP_POP);
      parsePrecedence(PREC_AND);

      patchJump(endJump);
    }

</div>

<div class="source-file-narrow">

*compiler.c*, add after *defineVariable*()

</div>

At the point this is called, the left-hand side expression has already
been compiled. That means at runtime, its value will be on top of the
stack. If that value is falsey, then we know the entire `and` must be
false, so we skip the right operand and leave the left-hand side value
as the result of the entire expression. Otherwise, we discard the
left-hand value and evaluate the right operand which becomes the result
of the whole `and` expression.

Those four lines of code right there produce exactly that. The flow
looks like this:

![Flowchart of the compiled bytecode of an 'and'
expression.](image/jumping-back-and-forth/and.png)

Now you can see why `OP_JUMP_IF_FALSE` <span id="instr">leaves</span>
the value on top of the stack. When the left-hand side of the `and` is
falsey, that value sticks around to become the result of the entire
expression.

We’ve got plenty of space left in our opcode range, so we could have
separate instructions for conditional jumps that implicitly pop and
those that don’t, I suppose. But I’m trying to keep things minimal for
the book. In your bytecode VM, it’s worth exploring adding more
specialized instructions and seeing how they affect performance.

### <a href="#logical-or-operator" id="logical-or-operator"><span
class="small">23 . 2 . 1</span>Logical or operator</a>

The `or` operator is a little more complex. First we add it to the parse
table.

<div class="codehilite">

``` insert-before
  [TOKEN_NIL]           = {literal,  NULL,   PREC_NONE},
```

<div class="source-file">

*compiler.c*  
replace 1 line

</div>

``` insert
  [TOKEN_OR]            = {NULL,     or_,    PREC_OR},
```

``` insert-after
  [TOKEN_PRINT]         = {NULL,     NULL,   PREC_NONE},
```

</div>

<div class="source-file-narrow">

*compiler.c*, replace 1 line

</div>

When that parser consumes an infix `or` token, it calls this:

<div class="codehilite">

<div class="source-file">

*compiler.c*  
add after *number*()

</div>

    static void or_(bool canAssign) {
      int elseJump = emitJump(OP_JUMP_IF_FALSE);
      int endJump = emitJump(OP_JUMP);

      patchJump(elseJump);
      emitByte(OP_POP);

      parsePrecedence(PREC_OR);
      patchJump(endJump);
    }

</div>

<div class="source-file-narrow">

*compiler.c*, add after *number*()

</div>

In an `or` expression, if the left-hand side is *truthy*, then we skip
over the right operand. Thus we need to jump when a value is truthy. We
could add a separate instruction, but just to show how our compiler is
free to map the language’s semantics to whatever instruction sequence it
wants, I implemented it in terms of the jump instructions we already
have.

When the left-hand side is falsey, it does a tiny jump over the next
statement. That statement is an unconditional jump over the code for the
right operand. This little dance effectively does a jump when the value
is truthy. The flow looks like this:

![Flowchart of the compiled bytecode of a logical or
expression.](image/jumping-back-and-forth/or.png)

If I’m honest with you, this isn’t the best way to do this. There are
more instructions to dispatch and more overhead. There’s no good reason
why `or` should be slower than `and`. But it is kind of fun to see that
it’s possible to implement both operators without adding any new
instructions. Forgive me my indulgences.

OK, those are the three *branching* constructs in Lox. By that, I mean,
these are the control flow features that only jump *forward* over code.
Other languages often have some kind of multi-way branching statement
like `switch` and maybe a conditional expression like `?:`, but Lox
keeps it simple.

## <a href="#while-statements" id="while-statements"><span
class="small">23 . 3</span>While Statements</a>

That takes us to the *looping* statements, which jump *backward* so that
code can be executed more than once. Lox only has two loop constructs,
`while` and `for`. A `while` loop is (much) simpler, so we start the
party there.

<div class="codehilite">

``` insert-before
    ifStatement();
```

<div class="source-file">

*compiler.c*  
in *statement*()

</div>

``` insert
  } else if (match(TOKEN_WHILE)) {
    whileStatement();
```

``` insert-after
  } else if (match(TOKEN_LEFT_BRACE)) {
```

</div>

<div class="source-file-narrow">

*compiler.c*, in *statement*()

</div>

When we reach a `while` token, we call:

<div class="codehilite">

<div class="source-file">

*compiler.c*  
add after *printStatement*()

</div>

    static void whileStatement() {
      consume(TOKEN_LEFT_PAREN, "Expect '(' after 'while'.");
      expression();
      consume(TOKEN_RIGHT_PAREN, "Expect ')' after condition.");

      int exitJump = emitJump(OP_JUMP_IF_FALSE);
      emitByte(OP_POP);
      statement();

      patchJump(exitJump);
      emitByte(OP_POP);
    }

</div>

<div class="source-file-narrow">

*compiler.c*, add after *printStatement*()

</div>

Most of this mirrors `if` statements<span class="em">—</span>we compile
the condition expression, surrounded by mandatory parentheses. That’s
followed by a jump instruction that skips over the subsequent body
statement if the condition is falsey.

We patch the jump after compiling the body and take care to
<span id="pop">pop</span> the condition value from the stack on either
path. The only difference from an `if` statement is the loop. That looks
like this:

Really starting to second-guess my decision to use the same jump
instructions for the logical operators.

<div class="codehilite">

``` insert-before
  statement();
```

<div class="source-file">

*compiler.c*  
in *whileStatement*()

</div>

``` insert
  emitLoop(loopStart);
```

``` insert-after

  patchJump(exitJump);
```

</div>

<div class="source-file-narrow">

*compiler.c*, in *whileStatement*()

</div>

After the body, we call this function to emit a “loop” instruction. That
instruction needs to know how far back to jump. When jumping forward, we
had to emit the instruction in two stages since we didn’t know how far
we were going to jump until after we emitted the jump instruction. We
don’t have that problem now. We’ve already compiled the point in code
that we want to jump back to<span class="em">—</span>it’s right before
the condition expression.

All we need to do is capture that location as we compile it.

<div class="codehilite">

``` insert-before
static void whileStatement() {
```

<div class="source-file">

*compiler.c*  
in *whileStatement*()

</div>

``` insert
  int loopStart = currentChunk()->count;
```

``` insert-after
  consume(TOKEN_LEFT_PAREN, "Expect '(' after 'while'.");
```

</div>

<div class="source-file-narrow">

*compiler.c*, in *whileStatement*()

</div>

After executing the body of a `while` loop, we jump all the way back to
before the condition. That way, we re-evaluate the condition expression
on each iteration. We store the chunk’s current instruction count in
`loopStart` to record the offset in the bytecode right before the
condition expression we’re about to compile. Then we pass that into this
helper function:

<div class="codehilite">

<div class="source-file">

*compiler.c*  
add after *emitBytes*()

</div>

    static void emitLoop(int loopStart) {
      emitByte(OP_LOOP);

      int offset = currentChunk()->count - loopStart + 2;
      if (offset > UINT16_MAX) error("Loop body too large.");

      emitByte((offset >> 8) & 0xff);
      emitByte(offset & 0xff);
    }

</div>

<div class="source-file-narrow">

*compiler.c*, add after *emitBytes*()

</div>

It’s a bit like `emitJump()` and `patchJump()` combined. It emits a new
loop instruction, which unconditionally jumps *backwards* by a given
offset. Like the jump instructions, after that we have a 16-bit operand.
We calculate the offset from the instruction we’re currently at to the
`loopStart` point that we want to jump back to. The `+ 2` is to take
into account the size of the `OP_LOOP` instruction’s own operands which
we also need to jump over.

From the VM’s perspective, there really is no semantic difference
between `OP_LOOP` and `OP_JUMP`. Both just add an offset to the `ip`. We
could have used a single instruction for both and given it a signed
offset operand. But I figured it was a little easier to sidestep the
annoying bit twiddling required to manually pack a signed 16-bit integer
into two bytes, and we’ve got the opcode space available, so why not use
it?

The new instruction is here:

<div class="codehilite">

``` insert-before
  OP_JUMP_IF_FALSE,
```

<div class="source-file">

*chunk.h*  
in enum *OpCode*

</div>

``` insert
  OP_LOOP,
```

``` insert-after
  OP_RETURN,
```

</div>

<div class="source-file-narrow">

*chunk.h*, in enum *OpCode*

</div>

And in the VM, we implement it thusly:

<div class="codehilite">

``` insert-before
      }
```

<div class="source-file">

*vm.c*  
in *run*()

</div>

``` insert
      case OP_LOOP: {
        uint16_t offset = READ_SHORT();
        vm.ip -= offset;
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

The only difference from `OP_JUMP` is a subtraction instead of an
addition. Disassembly is similar too.

<div class="codehilite">

``` insert-before
      return jumpInstruction("OP_JUMP_IF_FALSE", 1, chunk, offset);
```

<div class="source-file">

*debug.c*  
in *disassembleInstruction*()

</div>

``` insert
    case OP_LOOP:
      return jumpInstruction("OP_LOOP", -1, chunk, offset);
```

``` insert-after
    case OP_RETURN:
```

</div>

<div class="source-file-narrow">

*debug.c*, in *disassembleInstruction*()

</div>

That’s our `while` statement. It contains two
jumps<span class="em">—</span>a conditional forward one to escape the
loop when the condition is not met, and an unconditional loop backward
after we have executed the body. The flow looks like this:

![Flowchart of the compiled bytecode of a while
statement.](image/jumping-back-and-forth/while.png)

## <a href="#for-statements" id="for-statements"><span
class="small">23 . 4</span>For Statements</a>

The other looping statement in Lox is the venerable `for` loop,
inherited from C. It’s got a lot more going on with it compared to a
`while` loop. It has three clauses, all of which are optional:

<span id="detail"></span>

- The initializer can be a variable declaration or an expression. It
  runs once at the beginning of the statement.

- The condition clause is an expression. Like in a `while` loop, we exit
  the loop when it evaluates to something falsey.

- The increment expression runs once at the end of each loop iteration.

If you want a refresher, the corresponding chapter in part II goes
through the semantics [in more detail](control-flow.html#for-loops).

In jlox, the parser desugared a `for` loop to a synthesized AST for a
`while` loop with some extra stuff before it and at the end of the body.
We’ll do something similar, though we won’t go through anything like an
AST. Instead, our bytecode compiler will use the jump and loop
instructions we already have.

We’ll work our way through the implementation a piece at a time,
starting with the `for` keyword.

<div class="codehilite">

``` insert-before
    printStatement();
```

<div class="source-file">

*compiler.c*  
in *statement*()

</div>

``` insert
  } else if (match(TOKEN_FOR)) {
    forStatement();
```

``` insert-after
  } else if (match(TOKEN_IF)) {
```

</div>

<div class="source-file-narrow">

*compiler.c*, in *statement*()

</div>

It calls a helper function. If we only supported `for` loops with empty
clauses like `for (;;)`, then we could implement it like this:

<div class="codehilite">

<div class="source-file">

*compiler.c*  
add after *expressionStatement*()

</div>

    static void forStatement() {
      consume(TOKEN_LEFT_PAREN, "Expect '(' after 'for'.");
      consume(TOKEN_SEMICOLON, "Expect ';'.");

      int loopStart = currentChunk()->count;
      consume(TOKEN_SEMICOLON, "Expect ';'.");
      consume(TOKEN_RIGHT_PAREN, "Expect ')' after for clauses.");

      statement();
      emitLoop(loopStart);
    }

</div>

<div class="source-file-narrow">

*compiler.c*, add after *expressionStatement*()

</div>

There’s a bunch of mandatory punctuation at the top. Then we compile the
body. Like we did for `while` loops, we record the bytecode offset at
the top of the body and emit a loop to jump back to that point after it.
We’ve got a working implementation of
<span id="infinite">infinite</span> loops now.

Alas, without `return` statements, there isn’t any way to terminate it
short of a runtime error.

### <a href="#initializer-clause" id="initializer-clause"><span
class="small">23 . 4 . 1</span>Initializer clause</a>

Now we’ll add the first clause, the initializer. It executes only once,
before the body, so compiling is straightforward.

<div class="codehilite">

``` insert-before
  consume(TOKEN_LEFT_PAREN, "Expect '(' after 'for'.");
```

<div class="source-file">

*compiler.c*  
in *forStatement*()  
replace 1 line

</div>

``` insert
  if (match(TOKEN_SEMICOLON)) {
    // No initializer.
  } else if (match(TOKEN_VAR)) {
    varDeclaration();
  } else {
    expressionStatement();
  }
```

``` insert-after

  int loopStart = currentChunk()->count;
```

</div>

<div class="source-file-narrow">

*compiler.c*, in *forStatement*(), replace 1 line

</div>

The syntax is a little complex since we allow either a variable
declaration or an expression. We use the presence of the `var` keyword
to tell which we have. For the expression case, we call
`expressionStatement()` instead of `expression()`. That looks for a
semicolon, which we need here too, and also emits an `OP_POP`
instruction to discard the value. We don’t want the initializer to leave
anything on the stack.

If a `for` statement declares a variable, that variable should be scoped
to the loop body. We ensure that by wrapping the whole statement in a
scope.

<div class="codehilite">

``` insert-before
static void forStatement() {
```

<div class="source-file">

*compiler.c*  
in *forStatement*()

</div>

``` insert
  beginScope();
```

``` insert-after
  consume(TOKEN_LEFT_PAREN, "Expect '(' after 'for'.");
```

</div>

<div class="source-file-narrow">

*compiler.c*, in *forStatement*()

</div>

Then we close it at the end.

<div class="codehilite">

``` insert-before
  emitLoop(loopStart);
```

<div class="source-file">

*compiler.c*  
in *forStatement*()

</div>

``` insert
  endScope();
```

``` insert-after
}
```

</div>

<div class="source-file-narrow">

*compiler.c*, in *forStatement*()

</div>

### <a href="#condition-clause" id="condition-clause"><span
class="small">23 . 4 . 2</span>Condition clause</a>

Next, is the condition expression that can be used to exit the loop.

<div class="codehilite">

``` insert-before
  int loopStart = currentChunk()->count;
```

<div class="source-file">

*compiler.c*  
in *forStatement*()  
replace 1 line

</div>

``` insert
  int exitJump = -1;
  if (!match(TOKEN_SEMICOLON)) {
    expression();
    consume(TOKEN_SEMICOLON, "Expect ';' after loop condition.");

    // Jump out of the loop if the condition is false.
    exitJump = emitJump(OP_JUMP_IF_FALSE);
    emitByte(OP_POP); // Condition.
  }
```

``` insert-after
  consume(TOKEN_RIGHT_PAREN, "Expect ')' after for clauses.");
```

</div>

<div class="source-file-narrow">

*compiler.c*, in *forStatement*(), replace 1 line

</div>

Since the clause is optional, we need to see if it’s actually present.
If the clause is omitted, the next token must be a semicolon, so we look
for that to tell. If there isn’t a semicolon, there must be a condition
expression.

In that case, we compile it. Then, just like with while, we emit a
conditional jump that exits the loop if the condition is falsey. Since
the jump leaves the value on the stack, we pop it before executing the
body. That ensures we discard the value when the condition is true.

After the loop body, we need to patch that jump.

<div class="codehilite">

``` insert-before
  emitLoop(loopStart);
```

<div class="source-file">

*compiler.c*  
in *forStatement*()

</div>

``` insert

  if (exitJump != -1) {
    patchJump(exitJump);
    emitByte(OP_POP); // Condition.
  }
```

``` insert-after
  endScope();
}
```

</div>

<div class="source-file-narrow">

*compiler.c*, in *forStatement*()

</div>

We do this only when there is a condition clause. If there isn’t,
there’s no jump to patch and no condition value on the stack to pop.

### <a href="#increment-clause" id="increment-clause"><span
class="small">23 . 4 . 3</span>Increment clause</a>

I’ve saved the best for last, the increment clause. It’s pretty
convoluted. It appears textually before the body, but executes *after*
it. If we parsed to an AST and generated code in a separate pass, we
could simply traverse into and compile the `for` statement AST’s body
field before its increment clause.

Unfortunately, we can’t compile the increment clause later, since our
compiler only makes a single pass over the code. Instead, we’ll *jump
over* the increment, run the body, jump *back* up to the increment, run
it, and then go to the next iteration.

I know, a little weird, but hey, it beats manually managing ASTs in
memory in C, right? Here’s the code:

<div class="codehilite">

``` insert-before
  }
```

<div class="source-file">

*compiler.c*  
in *forStatement*()  
replace 1 line

</div>

``` insert
  if (!match(TOKEN_RIGHT_PAREN)) {
    int bodyJump = emitJump(OP_JUMP);
    int incrementStart = currentChunk()->count;
    expression();
    emitByte(OP_POP);
    consume(TOKEN_RIGHT_PAREN, "Expect ')' after for clauses.");

    emitLoop(loopStart);
    loopStart = incrementStart;
    patchJump(bodyJump);
  }
```

``` insert-after

  statement();
```

</div>

<div class="source-file-narrow">

*compiler.c*, in *forStatement*(), replace 1 line

</div>

Again, it’s optional. Since this is the last clause, when omitted, the
next token will be the closing parenthesis. When an increment is
present, we need to compile it now, but it shouldn’t execute yet. So,
first, we emit an unconditional jump that hops over the increment
clause’s code to the body of the loop.

Next, we compile the increment expression itself. This is usually an
assignment. Whatever it is, we only execute it for its side effect, so
we also emit a pop to discard its value.

The last part is a little tricky. First, we emit a loop instruction.
This is the main loop that takes us back to the top of the `for`
loop<span class="em">—</span>right before the condition expression if
there is one. That loop happens right after the increment, since the
increment executes at the end of each loop iteration.

Then we change `loopStart` to point to the offset where the increment
expression begins. Later, when we emit the loop instruction after the
body statement, this will cause it to jump up to the *increment*
expression instead of the top of the loop like it does when there is no
increment. This is how we weave the increment in to run after the body.

It’s convoluted, but it all works out. A complete loop with all the
clauses compiles to a flow like this:

![Flowchart of the compiled bytecode of a for
statement.](image/jumping-back-and-forth/for.png)

As with implementing `for` loops in jlox, we didn’t need to touch the
runtime. It all gets compiled down to primitive control flow operations
the VM already supports. In this chapter, we’ve taken a big
<span id="leap">leap</span> forward<span class="em">—</span>clox is now
Turing complete. We’ve also covered quite a bit of new syntax: three
statements and two expression forms. Even so, it only took three new
simple instructions. That’s a pretty good effort-to-reward ratio for the
architecture of our VM.

I couldn’t resist the pun. I regret nothing.

<div class="challenges">

## <a href="#challenges" id="challenges">Challenges</a>

1.  In addition to `if` statements, most C-family languages have a
    multi-way `switch` statement. Add one to clox. The grammar is:

    <div class="codehilite">

        switchStmt     → "switch" "(" expression ")"
                         "{" switchCase* defaultCase? "}" ;
        switchCase     → "case" expression ":" statement* ;
        defaultCase    → "default" ":" statement* ;

    </div>

    To execute a `switch` statement, first evaluate the parenthesized
    switch value expression. Then walk the cases. For each case,
    evaluate its value expression. If the case value is equal to the
    switch value, execute the statements under the case and then exit
    the `switch` statement. Otherwise, try the next case. If no case
    matches and there is a `default` clause, execute its statements.

    To keep things simpler, we’re omitting fallthrough and `break`
    statements. Each case automatically jumps to the end of the switch
    statement after its statements are done.

2.  In jlox, we had a challenge to add support for `break` statements.
    This time, let’s do `continue`:

    <div class="codehilite">

        continueStmt   → "continue" ";" ;

    </div>

    A `continue` statement jumps directly to the top of the nearest
    enclosing loop, skipping the rest of the loop body. Inside a `for`
    loop, a `continue` jumps to the increment clause, if there is one.
    It’s a compile-time error to have a `continue` statement not
    enclosed in a loop.

    Make sure to think about scope. What should happen to local
    variables declared inside the body of the loop or in blocks nested
    inside the loop when a `continue` is executed?

3.  Control flow constructs have been mostly unchanged since Algol 68.
    Language evolution since then has focused on making code more
    declarative and high level, so imperative control flow hasn’t gotten
    much attention.

    For fun, try to invent a useful novel control flow feature for Lox.
    It can be a refinement of an existing form or something entirely
    new. In practice, it’s hard to come up with something useful enough
    at this low expressiveness level to outweigh the cost of forcing a
    user to learn an unfamiliar notation and behavior, but it’s a good
    chance to practice your design skills.

</div>

<div class="design-note">

## <a href="#design-note" id="design-note">Design Note: Considering Goto
Harmful</a>

Discovering that all of our beautiful structured control flow in Lox is
actually compiled to raw unstructured jumps is like the moment in Scooby
Doo when the monster rips the mask off their face. It was goto all
along! Except in this case, the monster is *under* the mask. We all know
goto is evil. But<span class="ellipse"> . . . </span>why?

It is true that you can write outrageously unmaintainable code using
goto. But I don’t think most programmers around today have seen that
first hand. It’s been a long time since that style was common. These
days, it’s a boogie man we invoke in scary stories around the campfire.

The reason we rarely confront that monster in person is because Edsger
Dijkstra slayed it with his famous letter “Go To Statement Considered
Harmful”, published in *Communications of the ACM* (March, 1968). Debate
around structured programming had been fierce for some time with
adherents on both sides, but I think Dijkstra deserves the most credit
for effectively ending it. Most new languages today have no unstructured
jump statements.

A one-and-a-half page letter that almost single-handedly destroyed a
language feature must be pretty impressive stuff. If you haven’t read
it, I encourage you to do so. It’s a seminal piece of computer science
lore, one of our tribe’s ancestral songs. Also, it’s a nice, short bit
of practice for reading academic CS <span id="style">writing</span>,
which is a useful skill to develop.

That is, if you can get past Dijkstra’s insufferable faux-modest
self-aggrandizing writing style:

> More recently I discovered why the use of the go to statement has such
> disastrous effects. <span class="ellipse"> . . . </span>At that time I
> did not attach too much importance to this discovery; I now submit my
> considerations for publication because in very recent discussions in
> which the subject turned up, I have been urged to do so.

Ah, yet another one of my many discoveries. I couldn’t even be bothered
to write it up until the clamoring masses begged me to.

I’ve read it through a number of times, along with a few critiques,
responses, and commentaries. I ended up with mixed feelings, at best. At
a very high level, I’m with him. His general argument is something like
this:

1.  As programmers, we write programs<span class="em">—</span>static
    text<span class="em">—</span>but what we care about is the actual
    running program<span class="em">—</span>its dynamic behavior.

2.  We’re better at reasoning about static things than dynamic things.
    (He doesn’t provide any evidence to support this claim, but I accept
    it.)

3.  Thus, the more we can make the dynamic execution of the program
    reflect its textual structure, the better.

This is a good start. Drawing our attention to the separation between
the code we write and the code as it runs inside the machine is an
interesting insight. Then he tries to define a “correspondence” between
program text and execution. For someone who spent literally his entire
career advocating greater rigor in programming, his definition is pretty
hand-wavey. He says:

> Let us now consider how we can characterize the progress of a process.
> (You may think about this question in a very concrete manner: suppose
> that a process, considered as a time succession of actions, is stopped
> after an arbitrary action, what data do we have to fix in order that
> we can redo the process until the very same point?)

Imagine it like this. You have two computers with the same program
running on the exact same inputs<span class="em">—</span>so totally
deterministic. You pause one of them at an arbitrary point in its
execution. What data would you need to send to the other computer to be
able to stop it exactly as far along as the first one was?

If your program allows only simple statements like assignment, it’s
easy. You just need to know the point after the last statement you
executed. Basically a breakpoint, the `ip` in our VM, or the line number
in an error message. Adding branching control flow like `if` and
`switch` doesn’t add any more to this. Even if the marker points inside
a branch, we can still tell where we are.

Once you add function calls, you need something more. You could have
paused the first computer in the middle of a function, but that function
may be called from multiple places. To pause the second machine at
exactly the same point in *the entire program’s* execution, you need to
pause it on the *right* call to that function.

So you need to know not just the current statement, but, for function
calls that haven’t returned yet, you need to know the locations of the
callsites. In other words, a call stack, though I don’t think that term
existed when Dijkstra wrote this. Groovy.

He notes that loops make things harder. If you pause in the middle of a
loop body, you don’t know how many iterations have run. So he says you
also need to keep an iteration count. And, since loops can nest, you
need a stack of those (presumably interleaved with the call stack
pointers since you can be in loops in outer calls too).

This is where it gets weird. So we’re really building to something now,
and you expect him to explain how goto breaks all of this. Instead, he
just says:

> The unbridled use of the go to statement has an immediate consequence
> that it becomes terribly hard to find a meaningful set of coordinates
> in which to describe the process progress.

He doesn’t prove that this is hard, or say why. He just says it. He does
say that one approach is unsatisfactory:

> With the go to statement one can, of course, still describe the
> progress uniquely by a counter counting the number of actions
> performed since program start (viz. a kind of normalized clock). The
> difficulty is that such a coordinate, although unique, is utterly
> unhelpful.

But<span class="ellipse"> . . . </span>that’s effectively what loop
counters do, and he was fine with those. It’s not like every loop is a
simple “for every integer from 0 to 10” incrementing count. Many are
`while` loops with complex conditionals.

Taking an example close to home, consider the core bytecode execution
loop at the heart of clox. Dijkstra argues that that loop is tractable
because we can simply count how many times the loop has run to reason
about its progress. But that loop runs once for each executed
instruction in some user’s compiled Lox program. Does knowing that it
executed 6,201 bytecode instructions really tell us VM maintainers
*anything* edifying about the state of the interpreter?

In fact, this particular example points to a deeper truth. Böhm and
Jacopini
[proved](https://en.wikipedia.org/wiki/Structured_program_theorem) that
*any* control flow using goto can be transformed into one using just
sequencing, loops, and branches. Our bytecode interpreter loop is a
living example of that proof: it implements the unstructured control
flow of the clox bytecode instruction set without using any gotos
itself.

That seems to offer a counter-argument to Dijkstra’s claim: you *can*
define a correspondence for a program using gotos by transforming it to
one that doesn’t and then use the correspondence from that program,
which<span class="em">—</span>according to
him<span class="em">—</span>is acceptable because it uses only branches
and loops.

But, honestly, my argument here is also weak. I think both of us are
basically doing pretend math and using fake logic to make what should be
an empirical, human-centered argument. Dijkstra is right that some code
using goto is really bad. Much of that could and should be turned into
clearer code by using structured control flow.

By eliminating goto completely from languages, you’re definitely
prevented from writing bad code using gotos. It may be that forcing
users to use structured control flow and making it an uphill battle to
write goto-like code using those constructs is a net win for all of our
productivity.

But I do wonder sometimes if we threw out the baby with the bathwater.
In the absence of goto, we often resort to more complex structured
patterns. The “switch inside a loop” is a classic one. Another is using
a guard variable to exit out of a series of nested loops:

<span id="break"> </span>

<div class="codehilite">

    // See if the matrix contains a zero.
    bool found = false;
    for (int x = 0; x < xSize; x++) {
      for (int y = 0; y < ySize; y++) {
        for (int z = 0; z < zSize; z++) {
          if (matrix[x][y][z] == 0) {
            printf("found");
            found = true;
            break;
          }
        }
        if (found) break;
      }
      if (found) break;
    }

</div>

Is that really better than:

<div class="codehilite">

    for (int x = 0; x < xSize; x++) {
      for (int y = 0; y < ySize; y++) {
        for (int z = 0; z < zSize; z++) {
          if (matrix[x][y][z] == 0) {
            printf("found");
            goto done;
          }
        }
      }
    }
    done:

</div>

You could do this without `break`
statements<span class="em">—</span>themselves a limited goto-ish
construct<span class="em">—</span>by inserting `!found &&` at the
beginning of the condition clause of each loop.

I guess what I really don’t like is that we’re making language design
and engineering decisions today based on fear. Few people today have any
subtle understanding of the problems and benefits of goto. Instead, we
just think it’s “considered harmful”. Personally, I’ve never found dogma
a good starting place for quality creative work.

</div>

<a href="calls-and-functions.html" class="next">Next Chapter: “Calls and
Functions” →</a> Handcrafted by Robert Nystrom — <a
href="https://github.com/munificent/craftinginterpreters/blob/master/LICENSE"
target="_blank">© 2015 – 2021</a>

</div>

[![](image/logotype.png "Crafting Interpreters")](/)

<div class="contents">

### [Calls and Functions<span class="small">24</span>](#top)

- [<span class="small">24.1</span> Function Objects](#function-objects)
- [<span class="small">24.2</span> Compiling to Function
  Objects](#compiling-to-function-objects)
- [<span class="small">24.3</span> Call Frames](#call-frames)
- [<span class="small">24.4</span> Function
  Declarations](#function-declarations)
- [<span class="small">24.5</span> Function Calls](#function-calls)
- [<span class="small">24.6</span> Return
  Statements](#return-statements)
- [<span class="small">24.7</span> Native Functions](#native-functions)
- 
- [Challenges](#challenges)

<div class="prev-next">

<a href="jumping-back-and-forth.html" class="left"
title="Jumping Back and Forth">← Previous</a>
[↑ Up](a-bytecode-virtual-machine.html "A Bytecode Virtual Machine")
<a href="closures.html" class="right" title="Closures">Next →</a>

</div>

</div>

[![](image/logotype.png "Crafting Interpreters")](/)
<a href="jumping-back-and-forth.html" class="prev"
title="Jumping Back and Forth">←</a>
<a href="closures.html" class="next" title="Closures">→</a>

<div class="page">

<div class="nav-wrapper">

[![](image/logotype.png "Crafting Interpreters")](/)

<div class="expandable">

### [Calls and Functions<span class="small">24</span>](#top)

- [<span class="small">24.1</span> Function Objects](#function-objects)
- [<span class="small">24.2</span> Compiling to Function
  Objects](#compiling-to-function-objects)
- [<span class="small">24.3</span> Call Frames](#call-frames)
- [<span class="small">24.4</span> Function
  Declarations](#function-declarations)
- [<span class="small">24.5</span> Function Calls](#function-calls)
- [<span class="small">24.6</span> Return
  Statements](#return-statements)
- [<span class="small">24.7</span> Native Functions](#native-functions)
- 
- [Challenges](#challenges)

<div class="prev-next">

<a href="jumping-back-and-forth.html" class="left"
title="Jumping Back and Forth">← Previous</a>
[↑ Up](a-bytecode-virtual-machine.html "A Bytecode Virtual Machine")
<a href="closures.html" class="right" title="Closures">Next →</a>

</div>

</div>

<span id="expand-nav">≡</span>

</div>

<div class="number">

24

</div>

# Calls and Functions

> Any problem in computer science can be solved with another level of
> indirection. Except for the problem of too many layers of indirection.
>
> David Wheeler

This chapter is a beast. I try to break features into bite-sized pieces,
but sometimes you gotta swallow the whole <span id="eat">meal</span>.
Our next task is functions. We could start with only function
declarations, but that’s not very useful when you can’t call them. We
could do calls, but there’s nothing to call. And all of the runtime
support needed in the VM to support both of those isn’t very rewarding
if it isn’t hooked up to anything you can see. So we’re going to do it
all. It’s a lot, but we’ll feel good when we’re done.

Eating<span class="em">—</span>consumption<span class="em">—</span>is a
weird metaphor for a creative act. But most of the biological processes
that produce “output” are a little less, ahem, decorous.

## <a href="#function-objects" id="function-objects"><span
class="small">24 . 1</span>Function Objects</a>

The most interesting structural change in the VM is around the stack. We
already *have* a stack for local variables and temporaries, so we’re
partway there. But we have no notion of a *call* stack. Before we can
make much progress, we’ll have to fix that. But first, let’s write some
code. I always feel better once I start moving. We can’t do much without
having some kind of representation for functions, so we’ll start there.
From the VM’s perspective, what is a function?

A function has a body that can be executed, so that means some bytecode.
We could compile the entire program and all of its function declarations
into one big monolithic Chunk. Each function would have a pointer to the
first instruction of its code inside the Chunk.

This is roughly how compilation to native code works where you end up
with one solid blob of machine code. But for our bytecode VM, we can do
something a little higher level. I think a cleaner model is to give each
function its own Chunk. We’ll want some other metadata too, so let’s go
ahead and stuff it all in a struct now.

<div class="codehilite">

``` insert-before
  struct Obj* next;
};
```

<div class="source-file">

*object.h*  
add after struct *Obj*

</div>

``` insert

typedef struct {
  Obj obj;
  int arity;
  Chunk chunk;
  ObjString* name;
} ObjFunction;
```

``` insert-after

struct ObjString {
```

</div>

<div class="source-file-narrow">

*object.h*, add after struct *Obj*

</div>

Functions are first class in Lox, so they need to be actual Lox objects.
Thus ObjFunction has the same Obj header that all object types share.
The `arity` field stores the number of parameters the function expects.
Then, in addition to the chunk, we store the function’s
<span id="name">name</span>. That will be handy for reporting readable
runtime errors.

Humans don’t seem to find numeric bytecode offsets particularly
illuminating in crash dumps.

This is the first time the “object” module has needed to reference
Chunk, so we get an include.

<div class="codehilite">

``` insert-before
#include "common.h"
```

<div class="source-file">

*object.h*

</div>

``` insert
#include "chunk.h"
```

``` insert-after
#include "value.h"
```

</div>

<div class="source-file-narrow">

*object.h*

</div>

Like we did with strings, we define some accessories to make Lox
functions easier to work with in C. Sort of a poor man’s object
orientation. First, we’ll declare a C function to create a new Lox
function.

<div class="codehilite">

``` insert-before
  uint32_t hash;
};
```

<div class="source-file">

*object.h*  
add after struct *ObjString*

</div>

``` insert
ObjFunction* newFunction();
```

``` insert-after
ObjString* takeString(char* chars, int length);
```

</div>

<div class="source-file-narrow">

*object.h*, add after struct *ObjString*

</div>

The implementation is over here:

<div class="codehilite">

<div class="source-file">

*object.c*  
add after *allocateObject*()

</div>

    ObjFunction* newFunction() {
      ObjFunction* function = ALLOCATE_OBJ(ObjFunction, OBJ_FUNCTION);
      function->arity = 0;
      function->name = NULL;
      initChunk(&function->chunk);
      return function;
    }

</div>

<div class="source-file-narrow">

*object.c*, add after *allocateObject*()

</div>

We use our friend `ALLOCATE_OBJ()` to allocate memory and initialize the
object’s header so that the VM knows what type of object it is. Instead
of passing in arguments to initialize the function like we did with
ObjString, we set the function up in a sort of blank
state<span class="em">—</span>zero arity, no name, and no code. That
will get filled in later after the function is created.

Since we have a new kind of object, we need a new object type in the
enum.

<div class="codehilite">

``` insert-before
typedef enum {
```

<div class="source-file">

*object.h*  
in enum *ObjType*

</div>

``` insert
  OBJ_FUNCTION,
```

``` insert-after
  OBJ_STRING,
} ObjType;
```

</div>

<div class="source-file-narrow">

*object.h*, in enum *ObjType*

</div>

When we’re done with a function object, we must return the bits it
borrowed back to the operating system.

<div class="codehilite">

``` insert-before
  switch (object->type) {
```

<div class="source-file">

*memory.c*  
in *freeObject*()

</div>

``` insert
    case OBJ_FUNCTION: {
      ObjFunction* function = (ObjFunction*)object;
      freeChunk(&function->chunk);
      FREE(ObjFunction, object);
      break;
    }
```

``` insert-after
    case OBJ_STRING: {
```

</div>

<div class="source-file-narrow">

*memory.c*, in *freeObject*()

</div>

This switch case is <span id="free-name">responsible</span> for freeing
the ObjFunction itself as well as any other memory it owns. Functions
own their chunk, so we call Chunk’s destructor-like function.

We don’t need to explicitly free the function’s name because it’s an
ObjString. That means we can let the garbage collector manage its
lifetime for us. Or, at least, we’ll be able to once we [implement a
garbage collector](garbage-collection.html).

Lox lets you print any object, and functions are first-class objects, so
we need to handle them too.

<div class="codehilite">

``` insert-before
  switch (OBJ_TYPE(value)) {
```

<div class="source-file">

*object.c*  
in *printObject*()

</div>

``` insert
    case OBJ_FUNCTION:
      printFunction(AS_FUNCTION(value));
      break;
```

``` insert-after
    case OBJ_STRING:
```

</div>

<div class="source-file-narrow">

*object.c*, in *printObject*()

</div>

This calls out to:

<div class="codehilite">

<div class="source-file">

*object.c*  
add after *copyString*()

</div>

    static void printFunction(ObjFunction* function) {
      printf("<fn %s>", function->name->chars);
    }

</div>

<div class="source-file-narrow">

*object.c*, add after *copyString*()

</div>

Since a function knows its name, it may as well say it.

Finally, we have a couple of macros for converting values to functions.
First, make sure your value actually *is* a function.

<div class="codehilite">

``` insert-before
#define OBJ_TYPE(value)        (AS_OBJ(value)->type)
```

<div class="source-file">

*object.h*

</div>

``` insert
#define IS_FUNCTION(value)     isObjType(value, OBJ_FUNCTION)
```

``` insert-after
#define IS_STRING(value)       isObjType(value, OBJ_STRING)
```

</div>

<div class="source-file-narrow">

*object.h*

</div>

Assuming that evaluates to true, you can then safely cast the Value to
an ObjFunction pointer using this:

<div class="codehilite">

``` insert-before
#define IS_STRING(value)       isObjType(value, OBJ_STRING)
```

<div class="source-file">

*object.h*

</div>

``` insert
#define AS_FUNCTION(value)     ((ObjFunction*)AS_OBJ(value))
```

``` insert-after
#define AS_STRING(value)       ((ObjString*)AS_OBJ(value))
```

</div>

<div class="source-file-narrow">

*object.h*

</div>

With that, our object model knows how to represent functions. I’m
feeling warmed up now. You ready for something a little harder?

## <a href="#compiling-to-function-objects"
id="compiling-to-function-objects"><span
class="small">24 . 2</span>Compiling to Function Objects</a>

Right now, our compiler assumes it is always compiling to one single
chunk. With each function’s code living in separate chunks, that gets
more complex. When the compiler reaches a function declaration, it needs
to emit code into the function’s chunk when compiling its body. At the
end of the function body, the compiler needs to return to the previous
chunk it was working with.

That’s fine for code inside function bodies, but what about code that
isn’t? The “top level” of a Lox program is also imperative code and we
need a chunk to compile that into. We can simplify the compiler and VM
by placing that top-level code inside an automatically defined function
too. That way, the compiler is always within some kind of function body,
and the VM always runs code by invoking a function. It’s as if the
entire program is <span id="wrap">wrapped</span> inside an implicit
`main()` function.

One semantic corner where that analogy breaks down is global variables.
They have special scoping rules different from local variables, so in
that way, the top level of a script isn’t like a function body.

Before we get to user-defined functions, then, let’s do the
reorganization to support that implicit top-level function. It starts
with the Compiler struct. Instead of pointing directly to a Chunk that
the compiler writes to, it instead has a reference to the function
object being built.

<div class="codehilite">

``` insert-before
typedef struct {
```

<div class="source-file">

*compiler.c*  
in struct *Compiler*

</div>

``` insert
  ObjFunction* function;
  FunctionType type;
```

``` insert-after
  Local locals[UINT8_COUNT];
```

</div>

<div class="source-file-narrow">

*compiler.c*, in struct *Compiler*

</div>

We also have a little FunctionType enum. This lets the compiler tell
when it’s compiling top-level code versus the body of a function. Most
of the compiler doesn’t care about this<span class="em">—</span>that’s
why it’s a useful abstraction<span class="em">—</span>but in one or two
places the distinction is meaningful. We’ll get to one later.

<div class="codehilite">

<div class="source-file">

*compiler.c*  
add after struct *Local*

</div>

    typedef enum {
      TYPE_FUNCTION,
      TYPE_SCRIPT
    } FunctionType;

</div>

<div class="source-file-narrow">

*compiler.c*, add after struct *Local*

</div>

Every place in the compiler that was writing to the Chunk now needs to
go through that `function` pointer. Fortunately, many
<span id="current">chapters</span> ago, we encapsulated access to the
chunk in the `currentChunk()` function. We only need to fix that and the
rest of the compiler is happy.

It’s almost like I had a crystal ball that could see into the future and
knew we’d need to change the code later. But, really, it’s because I
wrote all the code for the book before any of the text.

<div class="codehilite">

``` insert-before
Compiler* current = NULL;
```

<div class="source-file">

*compiler.c*  
add after variable *current*  
replace 5 lines

</div>

``` insert

static Chunk* currentChunk() {
  return &current->function->chunk;
}
```

``` insert-after

static void errorAt(Token* token, const char* message) {
```

</div>

<div class="source-file-narrow">

*compiler.c*, add after variable *current*, replace 5 lines

</div>

The current chunk is always the chunk owned by the function we’re in the
middle of compiling. Next, we need to actually create that function.
Previously, the VM passed a Chunk to the compiler which filled it with
code. Instead, the compiler will create and return a function that
contains the compiled top-level code<span class="em">—</span>which is
all we support right now<span class="em">—</span>of the user’s program.

### <a href="#creating-functions-at-compile-time"
id="creating-functions-at-compile-time"><span
class="small">24 . 2 . 1</span>Creating functions at compile time</a>

We start threading this through in `compile()`, which is the main entry
point into the compiler.

<div class="codehilite">

``` insert-before
  Compiler compiler;
```

<div class="source-file">

*compiler.c*  
in *compile*()  
replace 2 lines

</div>

``` insert
  initCompiler(&compiler, TYPE_SCRIPT);
```

``` insert-after

  parser.hadError = false;
```

</div>

<div class="source-file-narrow">

*compiler.c*, in *compile*(), replace 2 lines

</div>

There are a bunch of changes in how the compiler is initialized. First,
we initialize the new Compiler fields.

<div class="codehilite">

<div class="source-file">

*compiler.c*  
function *initCompiler*()  
replace 1 line

</div>

``` insert
static void initCompiler(Compiler* compiler, FunctionType type) {
  compiler->function = NULL;
  compiler->type = type;
```

``` insert-after
  compiler->localCount = 0;
```

</div>

<div class="source-file-narrow">

*compiler.c*, function *initCompiler*(), replace 1 line

</div>

Then we allocate a new function object to compile into.

<div class="codehilite">

``` insert-before
  compiler->scopeDepth = 0;
```

<div class="source-file">

*compiler.c*  
in *initCompiler*()

</div>

``` insert
  compiler->function = newFunction();
```

``` insert-after
  current = compiler;
```

</div>

<div class="source-file-narrow">

*compiler.c*, in *initCompiler*()

</div>

<span id="null"></span>

I know, it looks dumb to null the `function` field only to immediately
assign it a value a few lines later. More garbage collection-related
paranoia.

Creating an ObjFunction in the compiler might seem a little strange. A
function object is the *runtime* representation of a function, but here
we are creating it at compile time. The way to think of it is that a
function is similar to a string or number literal. It forms a bridge
between the compile time and runtime worlds. When we get to function
*declarations*, those really *are* literals<span class="em">—</span>they
are a notation that produces values of a built-in type. So the
<span id="closure">compiler</span> creates function objects during
compilation. Then, at runtime, they are simply invoked.

We can create functions at compile time because they contain only data
available at compile time. The function’s code, name, and arity are all
fixed. When we add closures in the [next chapter](closures.html), which
capture variables at runtime, the story gets more complex.

Here is another strange piece of code:

<div class="codehilite">

``` insert-before
  current = compiler;
```

<div class="source-file">

*compiler.c*  
in *initCompiler*()

</div>

``` insert

  Local* local = &current->locals[current->localCount++];
  local->depth = 0;
  local->name.start = "";
  local->name.length = 0;
```

``` insert-after
}
```

</div>

<div class="source-file-narrow">

*compiler.c*, in *initCompiler*()

</div>

Remember that the compiler’s `locals` array keeps track of which stack
slots are associated with which local variables or temporaries. From now
on, the compiler implicitly claims stack slot zero for the VM’s own
internal use. We give it an empty name so that the user can’t write an
identifier that refers to it. I’ll explain what this is about when it
becomes useful.

That’s the initialization side. We also need a couple of changes on the
other end when we finish compiling some code.

<div class="codehilite">

<div class="source-file">

*compiler.c*  
function *endCompiler*()  
replace 1 line

</div>

``` insert
static ObjFunction* endCompiler() {
```

``` insert-after
  emitReturn();
```

</div>

<div class="source-file-narrow">

*compiler.c*, function *endCompiler*(), replace 1 line

</div>

Previously, when `interpret()` called into the compiler, it passed in a
Chunk to be written to. Now that the compiler creates the function
object itself, we return that function. We grab it from the current
compiler here:

<div class="codehilite">

``` insert-before
  emitReturn();
```

<div class="source-file">

*compiler.c*  
in *endCompiler*()

</div>

``` insert
  ObjFunction* function = current->function;
```

``` insert-after
#ifdef DEBUG_PRINT_CODE
```

</div>

<div class="source-file-narrow">

*compiler.c*, in *endCompiler*()

</div>

And then return it to `compile()` like so:

<div class="codehilite">

``` insert-before
#endif
```

<div class="source-file">

*compiler.c*  
in *endCompiler*()

</div>

``` insert

  return function;
```

``` insert-after
}
```

</div>

<div class="source-file-narrow">

*compiler.c*, in *endCompiler*()

</div>

Now is a good time to make another tweak in this function. Earlier, we
added some diagnostic code to have the VM dump the disassembled bytecode
so we could debug the compiler. We should fix that to keep working now
that the generated chunk is wrapped in a function.

<div class="codehilite">

``` insert-before
#ifdef DEBUG_PRINT_CODE
  if (!parser.hadError) {
```

<div class="source-file">

*compiler.c*  
in *endCompiler*()  
replace 1 line

</div>

``` insert
    disassembleChunk(currentChunk(), function->name != NULL
        ? function->name->chars : "<script>");
```

``` insert-after
  }
#endif
```

</div>

<div class="source-file-narrow">

*compiler.c*, in *endCompiler*(), replace 1 line

</div>

Notice the check in here to see if the function’s name is `NULL`?
User-defined functions have names, but the implicit function we create
for the top-level code does not, and we need to handle that gracefully
even in our own diagnostic code. Speaking of which:

<div class="codehilite">

``` insert-before
static void printFunction(ObjFunction* function) {
```

<div class="source-file">

*object.c*  
in *printFunction*()

</div>

``` insert
  if (function->name == NULL) {
    printf("<script>");
    return;
  }
```

``` insert-after
  printf("<fn %s>", function->name->chars);
```

</div>

<div class="source-file-narrow">

*object.c*, in *printFunction*()

</div>

There’s no way for a *user* to get a reference to the top-level function
and try to print it, but our `DEBUG_TRACE_EXECUTION`
<span id="debug">diagnostic</span> code that prints the entire stack can
and does.

It is no fun if the diagnostic code we use to find bugs itself causes
the VM to segfault!

Bumping up a level to `compile()`, we adjust its signature.

<div class="codehilite">

``` insert-before
#include "vm.h"
```

<div class="source-file">

*compiler.h*  
function *compile*()  
replace 1 line

</div>

``` insert
ObjFunction* compile(const char* source);
```

``` insert-after

#endif
```

</div>

<div class="source-file-narrow">

*compiler.h*, function *compile*(), replace 1 line

</div>

Instead of taking a chunk, now it returns a function. Over in the
implementation:

<div class="codehilite">

<div class="source-file">

*compiler.c*  
function *compile*()  
replace 1 line

</div>

``` insert
ObjFunction* compile(const char* source) {
```

``` insert-after
  initScanner(source);
```

</div>

<div class="source-file-narrow">

*compiler.c*, function *compile*(), replace 1 line

</div>

Finally we get to some actual code. We change the very end of the
function to this:

<div class="codehilite">

``` insert-before
  while (!match(TOKEN_EOF)) {
    declaration();
  }
```

<div class="source-file">

*compiler.c*  
in *compile*()  
replace 2 lines

</div>

``` insert
  ObjFunction* function = endCompiler();
  return parser.hadError ? NULL : function;
```

``` insert-after
}
```

</div>

<div class="source-file-narrow">

*compiler.c*, in *compile*(), replace 2 lines

</div>

We get the function object from the compiler. If there were no compile
errors, we return it. Otherwise, we signal an error by returning `NULL`.
This way, the VM doesn’t try to execute a function that may contain
invalid bytecode.

Eventually, we will update `interpret()` to handle the new declaration
of `compile()`, but first we have some other changes to make.

## <a href="#call-frames" id="call-frames"><span
class="small">24 . 3</span>Call Frames</a>

It’s time for a big conceptual leap. Before we can implement function
declarations and calls, we need to get the VM ready to handle them.
There are two main problems we need to worry about:

### <a href="#allocating-local-variables"
id="allocating-local-variables"><span
class="small">24 . 3 . 1</span>Allocating local variables</a>

The compiler allocates stack slots for local variables. How should that
work when the set of local variables in a program is distributed across
multiple functions?

One option would be to keep them totally separate. Each function would
get its own dedicated set of slots in the VM stack that it would own
<span id="static">forever</span>, even when the function isn’t being
called. Each local variable in the entire program would have a bit of
memory in the VM that it keeps to itself.

It’s basically what you’d get if you declared every local variable in a
C program using `static`.

Believe it or not, early programming language implementations worked
this way. The first Fortran compilers statically allocated memory for
each variable. The obvious problem is that it’s really inefficient. Most
functions are not in the middle of being called at any point in time, so
sitting on unused memory for them is wasteful.

The more fundamental problem, though, is recursion. With recursion, you
can be “in” multiple calls to the same function at the same time. Each
needs its <span id="fortran">own</span> memory for its local variables.
In jlox, we solved this by dynamically allocating memory for an
environment each time a function was called or a block entered. In clox,
we don’t want that kind of performance cost on every function call.

Fortran avoided this problem by disallowing recursion entirely.
Recursion was considered an advanced, esoteric feature at the time.

Instead, our solution lies somewhere between Fortran’s static allocation
and jlox’s dynamic approach. The value stack in the VM works on the
observation that local variables and temporaries behave in a last-in
first-out fashion. Fortunately for us, that’s still true even when you
add function calls into the mix. Here’s an example:

<div class="codehilite">

    fun first() {
      var a = 1;
      second();
      var b = 2;
    }

    fun second() {
      var c = 3;
      var d = 4;
    }

    first();

</div>

Step through the program and look at which variables are in memory at
each point in time:

![Tracing through the execution of the previous program, showing the
stack of variables at each step.](image/calls-and-functions/calls.png)

As execution flows through the two calls, every local variable obeys the
principle that any variable declared after it will be discarded before
the first variable needs to be. This is true even across calls. We know
we’ll be done with `c` and `d` before we are done with `a`. It seems we
should be able to allocate local variables on the VM’s value stack.

Ideally, we still determine *where* on the stack each variable will go
at compile time. That keeps the bytecode instructions for working with
variables simple and fast. In the above example, we could
<span id="imagine">imagine</span> doing so in a straightforward way, but
that doesn’t always work out. Consider:

I say “imagine” because the compiler can’t actually figure this out.
Because functions are first class in Lox, we can’t determine which
functions call which others at compile time.

<div class="codehilite">

    fun first() {
      var a = 1;
      second();
      var b = 2;
      second();
    }

    fun second() {
      var c = 3;
      var d = 4;
    }

    first();

</div>

In the first call to `second()`, `c` and `d` would go into slots 1 and
2. But in the second call, we need to have made room for `b`, so `c` and
`d` need to be in slots 2 and 3. Thus the compiler can’t pin down an
exact slot for each local variable across function calls. But *within* a
given function, the *relative* locations of each local variable are
fixed. Variable `d` is always in the slot right after `c`. This is the
key insight.

When a function is called, we don’t know where the top of the stack will
be because it can be called from different contexts. But, wherever that
top happens to be, we do know where all of the function’s local
variables will be relative to that starting point. So, like many
problems, we solve our allocation problem with a level of indirection.

At the beginning of each function call, the VM records the location of
the first slot where that function’s own locals begin. The instructions
for working with local variables access them by a slot index relative to
that, instead of relative to the bottom of the stack like they do today.
At compile time, we calculate those relative slots. At runtime, we
convert that relative slot to an absolute stack index by adding the
function call’s starting slot.

It’s as if the function gets a “window” or “frame” within the larger
stack where it can store its locals. The position of the **call frame**
is determined at runtime, but within and relative to that region, we
know where to find things.

![The stack at the two points when second() is called, with a window
hovering over each one showing the pair of stack slots used by the
function.](image/calls-and-functions/window.png)

The historical name for this recorded location where the function’s
locals start is a **frame pointer** because it points to the beginning
of the function’s call frame. Sometimes you hear **base pointer**,
because it points to the base stack slot on top of which all of the
function’s variables live.

That’s the first piece of data we need to track. Every time we call a
function, the VM determines the first stack slot where that function’s
variables begin.

### <a href="#return-addresses" id="return-addresses"><span
class="small">24 . 3 . 2</span>Return addresses</a>

Right now, the VM works its way through the instruction stream by
incrementing the `ip` field. The only interesting behavior is around
control flow instructions which offset the `ip` by larger amounts.
*Calling* a function is pretty
straightforward<span class="em">—</span>simply set `ip` to point to the
first instruction in that function’s chunk. But what about when the
function is done?

The VM needs to <span id="return">return</span> back to the chunk where
the function was called from and resume execution at the instruction
immediately after the call. Thus, for each function call, we need to
track where we jump back to when the call completes. This is called a
**return address** because it’s the address of the instruction that the
VM returns to after the call.

Again, thanks to recursion, there may be multiple return addresses for a
single function, so this is a property of each *invocation* and not the
function itself.

The authors of early Fortran compilers had a clever trick for
implementing return addresses. Since they *didn’t* support recursion,
any given function needed only a single return address at any point in
time. So when a function was called at runtime, the program would
*modify its own code* to change a jump instruction at the end of the
function to jump back to its caller. Sometimes the line between genius
and madness is hair thin.

### <a href="#the-call-stack" id="the-call-stack"><span
class="small">24 . 3 . 3</span>The call stack</a>

So for each live function invocation<span class="em">—</span>each call
that hasn’t returned yet<span class="em">—</span>we need to track where
on the stack that function’s locals begin, and where the caller should
resume. We’ll put this, along with some other stuff, in a new struct.

<div class="codehilite">

``` insert-before
#define STACK_MAX 256
```

<div class="source-file">

*vm.h*

</div>

``` insert

typedef struct {
  ObjFunction* function;
  uint8_t* ip;
  Value* slots;
} CallFrame;
```

``` insert-after

typedef struct {
```

</div>

<div class="source-file-narrow">

*vm.h*

</div>

A CallFrame represents a single ongoing function call. The `slots` field
points into the VM’s value stack at the first slot that this function
can use. I gave it a plural name because<span class="em">—</span>thanks
to C’s weird “pointers are sort of arrays”
thing<span class="em">—</span>we’ll treat it like an array.

The implementation of return addresses is a little different from what I
described above. Instead of storing the return address in the callee’s
frame, the caller stores its own `ip`. When we return from a function,
the VM will jump to the `ip` of the caller’s CallFrame and resume from
there.

I also stuffed a pointer to the function being called in here. We’ll use
that to look up constants and for a few other things.

Each time a function is called, we create one of these structs. We could
<span id="heap">dynamically</span> allocate them on the heap, but that’s
slow. Function calls are a core operation, so they need to be as fast as
possible. Fortunately, we can make the same observation we made for
variables: function calls have stack semantics. If `first()` calls
`second()`, the call to `second()` will complete before `first()` does.

Many Lisp implementations dynamically allocate stack frames because it
simplifies implementing
[continuations](https://en.wikipedia.org/wiki/Continuation). If your
language supports continuations, then function calls do *not* always
have stack semantics.

So over in the VM, we create an array of these CallFrame structs up
front and treat it as a stack, like we do with the value array.

<div class="codehilite">

``` insert-before
typedef struct {
```

<div class="source-file">

*vm.h*  
in struct *VM*  
replace 2 lines

</div>

``` insert
  CallFrame frames[FRAMES_MAX];
  int frameCount;
```

``` insert-after
  Value stack[STACK_MAX];
```

</div>

<div class="source-file-narrow">

*vm.h*, in struct *VM*, replace 2 lines

</div>

This array replaces the `chunk` and `ip` fields we used to have directly
in the VM. Now each CallFrame has its own `ip` and its own pointer to
the ObjFunction that it’s executing. From there, we can get to the
function’s chunk.

The new `frameCount` field in the VM stores the current height of the
CallFrame stack<span class="em">—</span>the number of ongoing function
calls. To keep clox simple, the array’s capacity is fixed. This means,
as in many language implementations, there is a maximum call depth we
can handle. For clox, it’s defined here:

<div class="codehilite">

``` insert-before
#include "value.h"
```

<div class="source-file">

*vm.h*  
replace 1 line

</div>

``` insert
#define FRAMES_MAX 64
#define STACK_MAX (FRAMES_MAX * UINT8_COUNT)
```

``` insert-after

typedef struct {
```

</div>

<div class="source-file-narrow">

*vm.h*, replace 1 line

</div>

We also redefine the value stack’s <span id="plenty">size</span> in
terms of that to make sure we have plenty of stack slots even in very
deep call trees. When the VM starts up, the CallFrame stack is empty.

It is still possible to overflow the stack if enough function calls use
enough temporaries in addition to locals. A robust implementation would
guard against this, but I’m trying to keep things simple.

<div class="codehilite">

``` insert-before
  vm.stackTop = vm.stack;
```

<div class="source-file">

*vm.c*  
in *resetStack*()

</div>

``` insert
  vm.frameCount = 0;
```

``` insert-after
}
```

</div>

<div class="source-file-narrow">

*vm.c*, in *resetStack*()

</div>

The “vm.h” header needs access to ObjFunction, so we add an include.

<div class="codehilite">

``` insert-before
#define clox_vm_h
```

<div class="source-file">

*vm.h*  
replace 1 line

</div>

``` insert
#include "object.h"
```

``` insert-after
#include "table.h"
```

</div>

<div class="source-file-narrow">

*vm.h*, replace 1 line

</div>

Now we’re ready to move over to the VM’s implementation file. We’ve got
some grunt work ahead of us. We’ve moved `ip` out of the VM struct and
into CallFrame. We need to fix every line of code in the VM that touches
`ip` to handle that. Also, the instructions that access local variables
by stack slot need to be updated to do so relative to the current
CallFrame’s `slots` field.

We’ll start at the top and plow through it.

<div class="codehilite">

``` insert-before
static InterpretResult run() {
```

<div class="source-file">

*vm.c*  
in *run*()  
replace 4 lines

</div>

``` insert
  CallFrame* frame = &vm.frames[vm.frameCount - 1];

#define READ_BYTE() (*frame->ip++)

#define READ_SHORT() \
    (frame->ip += 2, \
    (uint16_t)((frame->ip[-2] << 8) | frame->ip[-1]))

#define READ_CONSTANT() \
    (frame->function->chunk.constants.values[READ_BYTE()])
```

``` insert-after
#define READ_STRING() AS_STRING(READ_CONSTANT())
```

</div>

<div class="source-file-narrow">

*vm.c*, in *run*(), replace 4 lines

</div>

First, we store the current topmost CallFrame in a
<span id="local">local</span> variable inside the main bytecode
execution function. Then we replace the bytecode access macros with
versions that access `ip` through that variable.

We could access the current frame by going through the CallFrame array
every time, but that’s verbose. More importantly, storing the frame in a
local variable encourages the C compiler to keep that pointer in a
register. That speeds up access to the frame’s `ip`. There’s no
*guarantee* that the compiler will do this, but there’s a good chance it
will.

Now onto each instruction that needs a little tender loving care.

<div class="codehilite">

``` insert-before
      case OP_GET_LOCAL: {
        uint8_t slot = READ_BYTE();
```

<div class="source-file">

*vm.c*  
in *run*()  
replace 1 line

</div>

``` insert
        push(frame->slots[slot]);
```

``` insert-after
        break;
```

</div>

<div class="source-file-narrow">

*vm.c*, in *run*(), replace 1 line

</div>

Previously, `OP_GET_LOCAL` read the given local slot directly from the
VM’s stack array, which meant it indexed the slot starting from the
bottom of the stack. Now, it accesses the current frame’s `slots` array,
which means it accesses the given numbered slot relative to the
beginning of that frame.

Setting a local variable works the same way.

<div class="codehilite">

``` insert-before
      case OP_SET_LOCAL: {
        uint8_t slot = READ_BYTE();
```

<div class="source-file">

*vm.c*  
in *run*()  
replace 1 line

</div>

``` insert
        frame->slots[slot] = peek(0);
```

``` insert-after
        break;
```

</div>

<div class="source-file-narrow">

*vm.c*, in *run*(), replace 1 line

</div>

The jump instructions used to modify the VM’s `ip` field. Now, they do
the same for the current frame’s `ip`.

<div class="codehilite">

``` insert-before
      case OP_JUMP: {
        uint16_t offset = READ_SHORT();
```

<div class="source-file">

*vm.c*  
in *run*()  
replace 1 line

</div>

``` insert
        frame->ip += offset;
```

``` insert-after
        break;
```

</div>

<div class="source-file-narrow">

*vm.c*, in *run*(), replace 1 line

</div>

Same with the conditional jump:

<div class="codehilite">

``` insert-before
      case OP_JUMP_IF_FALSE: {
        uint16_t offset = READ_SHORT();
```

<div class="source-file">

*vm.c*  
in *run*()  
replace 1 line

</div>

``` insert
        if (isFalsey(peek(0))) frame->ip += offset;
```

``` insert-after
        break;
```

</div>

<div class="source-file-narrow">

*vm.c*, in *run*(), replace 1 line

</div>

And our backward-jumping loop instruction:

<div class="codehilite">

``` insert-before
      case OP_LOOP: {
        uint16_t offset = READ_SHORT();
```

<div class="source-file">

*vm.c*  
in *run*()  
replace 1 line

</div>

``` insert
        frame->ip -= offset;
```

``` insert-after
        break;
```

</div>

<div class="source-file-narrow">

*vm.c*, in *run*(), replace 1 line

</div>

We have some diagnostic code that prints each instruction as it executes
to help us debug our VM. That needs to work with the new structure too.

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
    disassembleInstruction(&frame->function->chunk,
        (int)(frame->ip - frame->function->chunk.code));
```

``` insert-after
#endif
```

</div>

<div class="source-file-narrow">

*vm.c*, in *run*(), replace 2 lines

</div>

Instead of passing in the VM’s `chunk` and `ip` fields, now we read from
the current CallFrame.

You know, that wasn’t too bad, actually. Most instructions just use the
macros so didn’t need to be touched. Next, we jump up a level to the
code that calls `run()`.

<div class="codehilite">

``` insert-before
InterpretResult interpret(const char* source) {
```

<div class="source-file">

*vm.c*  
in *interpret*()  
replace 10 lines

</div>

``` insert
  ObjFunction* function = compile(source);
  if (function == NULL) return INTERPRET_COMPILE_ERROR;

  push(OBJ_VAL(function));
  CallFrame* frame = &vm.frames[vm.frameCount++];
  frame->function = function;
  frame->ip = function->chunk.code;
  frame->slots = vm.stack;
```

``` insert-after

  InterpretResult result = run();
```

</div>

<div class="source-file-narrow">

*vm.c*, in *interpret*(), replace 10 lines

</div>

We finally get to wire up our earlier compiler changes to the back-end
changes we just made. First, we pass the source code to the compiler. It
returns us a new ObjFunction containing the compiled top-level code. If
we get `NULL` back, it means there was some compile-time error which the
compiler has already reported. In that case, we bail out since we can’t
run anything.

Otherwise, we store the function on the stack and prepare an initial
CallFrame to execute its code. Now you can see why the compiler sets
aside stack slot zero<span class="em">—</span>that stores the function
being called. In the new CallFrame, we point to the function, initialize
its `ip` to point to the beginning of the function’s bytecode, and set
up its stack window to start at the very bottom of the VM’s value stack.

This gets the interpreter ready to start executing code. After
finishing, the VM used to free the hardcoded chunk. Now that the
ObjFunction owns that code, we don’t need to do that anymore, so the end
of `interpret()` is simply this:

<div class="codehilite">

``` insert-before
  frame->slots = vm.stack;
```

<div class="source-file">

*vm.c*  
in *interpret*()  
replace 4 lines

</div>

``` insert
  return run();
```

``` insert-after
}
```

</div>

<div class="source-file-narrow">

*vm.c*, in *interpret*(), replace 4 lines

</div>

The last piece of code referring to the old VM fields is
`runtimeError()`. We’ll revisit that later in the chapter, but for now
let’s change it to this:

<div class="codehilite">

``` insert-before
  fputs("\n", stderr);
```

<div class="source-file">

*vm.c*  
in *runtimeError*()  
replace 2 lines

</div>

``` insert
  CallFrame* frame = &vm.frames[vm.frameCount - 1];
  size_t instruction = frame->ip - frame->function->chunk.code - 1;
  int line = frame->function->chunk.lines[instruction];
```

``` insert-after
  fprintf(stderr, "[line %d] in script\n", line);
```

</div>

<div class="source-file-narrow">

*vm.c*, in *runtimeError*(), replace 2 lines

</div>

Instead of reading the chunk and `ip` directly from the VM, it pulls
those from the topmost CallFrame on the stack. That should get the
function working again and behaving as it did before.

Assuming we did all of that correctly, we got clox back to a runnable
state. Fire it up and it does<span class="ellipse"> . . . </span>exactly
what it did before. We haven’t added any new features yet, so this is
kind of a let down. But all of the infrastructure is there and ready for
us now. Let’s take advantage of it.

## <a href="#function-declarations" id="function-declarations"><span
class="small">24 . 4</span>Function Declarations</a>

Before we can do call expressions, we need something to call, so we’ll
do function declarations first. The <span id="fun">fun</span> starts
with a keyword.

Yes, I am going to make a dumb joke about the `fun` keyword every time
it comes up.

<div class="codehilite">

``` insert-before
static void declaration() {
```

<div class="source-file">

*compiler.c*  
in *declaration*()  
replace 1 line

</div>

``` insert
  if (match(TOKEN_FUN)) {
    funDeclaration();
  } else if (match(TOKEN_VAR)) {
```

``` insert-after
    varDeclaration();
```

</div>

<div class="source-file-narrow">

*compiler.c*, in *declaration*(), replace 1 line

</div>

That passes control to here:

<div class="codehilite">

<div class="source-file">

*compiler.c*  
add after *block*()

</div>

    static void funDeclaration() {
      uint8_t global = parseVariable("Expect function name.");
      markInitialized();
      function(TYPE_FUNCTION);
      defineVariable(global);
    }

</div>

<div class="source-file-narrow">

*compiler.c*, add after *block*()

</div>

Functions are first-class values, and a function declaration simply
creates and stores one in a newly declared variable. So we parse the
name just like any other variable declaration. A function declaration at
the top level will bind the function to a global variable. Inside a
block or other function, a function declaration creates a local
variable.

In an earlier chapter, I explained how variables [get defined in two
stages](local-variables.html#another-scope-edge-case). This ensures you
can’t access a variable’s value inside the variable’s own initializer.
That would be bad because the variable doesn’t *have* a value yet.

Functions don’t suffer from this problem. It’s safe for a function to
refer to its own name inside its body. You can’t *call* the function and
execute the body until after it’s fully defined, so you’ll never see the
variable in an uninitialized state. Practically speaking, it’s useful to
allow this in order to support recursive local functions.

To make that work, we mark the function declaration’s variable
“initialized” as soon as we compile the name, before we compile the
body. That way the name can be referenced inside the body without
generating an error.

We do need one check, though.

<div class="codehilite">

``` insert-before
static void markInitialized() {
```

<div class="source-file">

*compiler.c*  
in *markInitialized*()

</div>

``` insert
  if (current->scopeDepth == 0) return;
```

``` insert-after
  current->locals[current->localCount - 1].depth =
```

</div>

<div class="source-file-narrow">

*compiler.c*, in *markInitialized*()

</div>

Before, we called `markInitialized()` only when we already knew we were
in a local scope. Now, a top-level function declaration will also call
this function. When that happens, there is no local variable to mark
initialized<span class="em">—</span>the function is bound to a global
variable.

Next, we compile the function itself<span class="em">—</span>its
parameter list and block body. For that, we use a separate helper
function. That helper generates code that leaves the resulting function
object on top of the stack. After that, we call `defineVariable()` to
store that function back into the variable we declared for it.

I split out the code to compile the parameters and body because we’ll
reuse it later for parsing method declarations inside classes. Let’s
build it incrementally, starting with this:

<div class="codehilite">

<div class="source-file">

*compiler.c*  
add after *block*()

</div>

    static void function(FunctionType type) {
      Compiler compiler;
      initCompiler(&compiler, type);
      beginScope(); 

      consume(TOKEN_LEFT_PAREN, "Expect '(' after function name.");
      consume(TOKEN_RIGHT_PAREN, "Expect ')' after parameters.");
      consume(TOKEN_LEFT_BRACE, "Expect '{' before function body.");
      block();

      ObjFunction* function = endCompiler();
      emitBytes(OP_CONSTANT, makeConstant(OBJ_VAL(function)));
    }

</div>

<div class="source-file-narrow">

*compiler.c*, add after *block*()

</div>

This `beginScope()` doesn’t have a corresponding `endScope()` call.
Because we end Compiler completely when we reach the end of the function
body, there’s no need to close the lingering outermost scope.

For now, we won’t worry about parameters. We parse an empty pair of
parentheses followed by the body. The body starts with a left curly
brace, which we parse here. Then we call our existing `block()`
function, which knows how to compile the rest of a block including the
closing brace.

### <a href="#a-stack-of-compilers" id="a-stack-of-compilers"><span
class="small">24 . 4 . 1</span>A stack of compilers</a>

The interesting parts are the compiler stuff at the top and bottom. The
Compiler struct stores data like which slots are owned by which local
variables, how many blocks of nesting we’re currently in, etc. All of
that is specific to a single function. But now the front end needs to
handle compiling multiple functions <span id="nested">nested</span>
within each other.

Remember that the compiler treats top-level code as the body of an
implicit function, so as soon as we add *any* function declarations,
we’re in a world of nested functions.

The trick for managing that is to create a separate Compiler for each
function being compiled. When we start compiling a function declaration,
we create a new Compiler on the C stack and initialize it.
`initCompiler()` sets that Compiler to be the current one. Then, as we
compile the body, all of the functions that emit bytecode write to the
chunk owned by the new Compiler’s function.

After we reach the end of the function’s block body, we call
`endCompiler()`. That yields the newly compiled function object, which
we store as a constant in the *surrounding* function’s constant table.
But, wait, how do we get back to the surrounding function? We lost it
when `initCompiler()` overwrote the current compiler pointer.

We fix that by treating the series of nested Compiler structs as a
stack. Unlike the Value and CallFrame stacks in the VM, we won’t use an
array. Instead, we use a linked list. Each Compiler points back to the
Compiler for the function that encloses it, all the way back to the root
Compiler for the top-level code.

<div class="codehilite">

``` insert-before
} FunctionType;
```

<div class="source-file">

*compiler.c*  
add after enum *FunctionType*  
replace 1 line

</div>

``` insert
typedef struct Compiler {
  struct Compiler* enclosing;
```

``` insert-after
  ObjFunction* function;
```

</div>

<div class="source-file-narrow">

*compiler.c*, add after enum *FunctionType*, replace 1 line

</div>

Inside the Compiler struct, we can’t reference the Compiler *typedef*
since that declaration hasn’t finished yet. Instead, we give a name to
the struct itself and use that for the field’s type. C is weird.

When initializing a new Compiler, we capture the
about-to-no-longer-be-current one in that pointer.

<div class="codehilite">

``` insert-before
static void initCompiler(Compiler* compiler, FunctionType type) {
```

<div class="source-file">

*compiler.c*  
in *initCompiler*()

</div>

``` insert
  compiler->enclosing = current;
```

``` insert-after
  compiler->function = NULL;
```

</div>

<div class="source-file-narrow">

*compiler.c*, in *initCompiler*()

</div>

Then when a Compiler finishes, it pops itself off the stack by restoring
the previous compiler to be the new current one.

<div class="codehilite">

``` insert-before
#endif
```

<div class="source-file">

*compiler.c*  
in *endCompiler*()

</div>

``` insert
  current = current->enclosing;
```

``` insert-after
  return function;
```

</div>

<div class="source-file-narrow">

*compiler.c*, in *endCompiler*()

</div>

Note that we don’t even need to <span id="compiler">dynamically</span>
allocate the Compiler structs. Each is stored as a local variable in the
C stack<span class="em">—</span>either in `compile()` or `function()`.
The linked list of Compilers threads through the C stack. The reason we
can get an unbounded number of them is because our compiler uses
recursive descent, so `function()` ends up calling itself recursively
when you have nested function declarations.

Using the native stack for Compiler structs does mean our compiler has a
practical limit on how deeply nested function declarations can be. Go
too far and you could overflow the C stack. If we want the compiler to
be more robust against pathological or even malicious
code<span class="em">—</span>a real concern for tools like JavaScript
VMs<span class="em">—</span>it would be good to have our compiler
artificially limit the amount of function nesting it permits.

### <a href="#function-parameters" id="function-parameters"><span
class="small">24 . 4 . 2</span>Function parameters</a>

Functions aren’t very useful if you can’t pass arguments to them, so
let’s do parameters next.

<div class="codehilite">

``` insert-before
  consume(TOKEN_LEFT_PAREN, "Expect '(' after function name.");
```

<div class="source-file">

*compiler.c*  
in *function*()

</div>

``` insert
  if (!check(TOKEN_RIGHT_PAREN)) {
    do {
      current->function->arity++;
      if (current->function->arity > 255) {
        errorAtCurrent("Can't have more than 255 parameters.");
      }
      uint8_t constant = parseVariable("Expect parameter name.");
      defineVariable(constant);
    } while (match(TOKEN_COMMA));
  }
```

``` insert-after
  consume(TOKEN_RIGHT_PAREN, "Expect ')' after parameters.");
```

</div>

<div class="source-file-narrow">

*compiler.c*, in *function*()

</div>

Semantically, a parameter is simply a local variable declared in the
outermost lexical scope of the function body. We get to use the existing
compiler support for declaring named local variables to parse and
compile parameters. Unlike local variables, which have initializers,
there’s no code here to initialize the parameter’s value. We’ll see how
they are initialized later when we do argument passing in function
calls.

While we’re at it, we note the function’s arity by counting how many
parameters we parse. The other piece of metadata we store with a
function is its name. When compiling a function declaration, we call
`initCompiler()` right after we parse the function’s name. That means we
can grab the name right then from the previous token.

<div class="codehilite">

``` insert-before
  current = compiler;
```

<div class="source-file">

*compiler.c*  
in *initCompiler*()

</div>

``` insert
  if (type != TYPE_SCRIPT) {
    current->function->name = copyString(parser.previous.start,
                                         parser.previous.length);
  }
```

``` insert-after

  Local* local = &current->locals[current->localCount++];
```

</div>

<div class="source-file-narrow">

*compiler.c*, in *initCompiler*()

</div>

Note that we’re careful to create a copy of the name string. Remember,
the lexeme points directly into the original source code string. That
string may get freed once the code is finished compiling. The function
object we create in the compiler outlives the compiler and persists
until runtime. So it needs its own heap-allocated name string that it
can keep around.

Rad. Now we can compile function declarations, like this:

<div class="codehilite">

    fun areWeHavingItYet() {
      print "Yes we are!";
    }

    print areWeHavingItYet;

</div>

We just can’t do anything <span id="useful">useful</span> with them.

We can print them! I guess that’s not very useful, though.

## <a href="#function-calls" id="function-calls"><span
class="small">24 . 5</span>Function Calls</a>

By the end of this section, we’ll start to see some interesting
behavior. The next step is calling functions. We don’t usually think of
it this way, but a function call expression is kind of an infix `(`
operator. You have a high-precedence expression on the left for the
thing being called<span class="em">—</span>usually just a single
identifier. Then the `(` in the middle, followed by the argument
expressions separated by commas, and a final `)` to wrap it up at the
end.

That odd grammatical perspective explains how to hook the syntax into
our parsing table.

<div class="codehilite">

``` insert-before
ParseRule rules[] = {
```

<div class="source-file">

*compiler.c*  
add after *unary*()  
replace 1 line

</div>

``` insert
  [TOKEN_LEFT_PAREN]    = {grouping, call,   PREC_CALL},
```

``` insert-after
  [TOKEN_RIGHT_PAREN]   = {NULL,     NULL,   PREC_NONE},
```

</div>

<div class="source-file-narrow">

*compiler.c*, add after *unary*(), replace 1 line

</div>

When the parser encounters a left parenthesis following an expression,
it dispatches to a new parser function.

<div class="codehilite">

<div class="source-file">

*compiler.c*  
add after *binary*()

</div>

    static void call(bool canAssign) {
      uint8_t argCount = argumentList();
      emitBytes(OP_CALL, argCount);
    }

</div>

<div class="source-file-narrow">

*compiler.c*, add after *binary*()

</div>

We’ve already consumed the `(` token, so next we compile the arguments
using a separate `argumentList()` helper. That function returns the
number of arguments it compiled. Each argument expression generates code
that leaves its value on the stack in preparation for the call. After
that, we emit a new `OP_CALL` instruction to invoke the function, using
the argument count as an operand.

We compile the arguments using this friend:

<div class="codehilite">

<div class="source-file">

*compiler.c*  
add after *defineVariable*()

</div>

    static uint8_t argumentList() {
      uint8_t argCount = 0;
      if (!check(TOKEN_RIGHT_PAREN)) {
        do {
          expression();
          argCount++;
        } while (match(TOKEN_COMMA));
      }
      consume(TOKEN_RIGHT_PAREN, "Expect ')' after arguments.");
      return argCount;
    }

</div>

<div class="source-file-narrow">

*compiler.c*, add after *defineVariable*()

</div>

That code should look familiar from jlox. We chew through arguments as
long as we find commas after each expression. Once we run out, we
consume the final closing parenthesis and we’re done.

Well, almost. Back in jlox, we added a compile-time check that you don’t
pass more than 255 arguments to a call. At the time, I said that was
because clox would need a similar limit. Now you can see
why<span class="em">—</span>since we stuff the argument count into the
bytecode as a single-byte operand, we can only go up to 255. We need to
verify that in this compiler too.

<div class="codehilite">

``` insert-before
      expression();
```

<div class="source-file">

*compiler.c*  
in *argumentList*()

</div>

``` insert
      if (argCount == 255) {
        error("Can't have more than 255 arguments.");
      }
```

``` insert-after
      argCount++;
```

</div>

<div class="source-file-narrow">

*compiler.c*, in *argumentList*()

</div>

That’s the front end. Let’s skip over to the back end, with a quick stop
in the middle to declare the new instruction.

<div class="codehilite">

``` insert-before
  OP_LOOP,
```

<div class="source-file">

*chunk.h*  
in enum *OpCode*

</div>

``` insert
  OP_CALL,
```

``` insert-after
  OP_RETURN,
```

</div>

<div class="source-file-narrow">

*chunk.h*, in enum *OpCode*

</div>

### <a href="#binding-arguments-to-parameters"
id="binding-arguments-to-parameters"><span
class="small">24 . 5 . 1</span>Binding arguments to parameters</a>

Before we get to the implementation, we should think about what the
stack looks like at the point of a call and what we need to do from
there. When we reach the call instruction, we have already executed the
expression for the function being called, followed by its arguments. Say
our program looks like this:

<div class="codehilite">

    fun sum(a, b, c) {
      return a + b + c;
    }

    print 4 + sum(5, 6, 7);

</div>

If we pause the VM right on the `OP_CALL` instruction for that call to
`sum()`, the stack looks like this:

![Stack: 4, fn sum, 5, 6,
7.](image/calls-and-functions/argument-stack.png)

Picture this from the perspective of `sum()` itself. When the compiler
compiled `sum()`, it automatically allocated slot zero. Then, after
that, it allocated local slots for the parameters `a`, `b`, and `c`, in
order. To perform a call to `sum()`, we need a CallFrame initialized
with the function being called and a region of stack slots that it can
use. Then we need to collect the arguments passed to the function and
get them into the corresponding slots for the parameters.

When the VM starts executing the body of `sum()`, we want its stack
window to look like this:

![The same stack with the sum() function's call frame window surrounding
fn sum, 5, 6, and 7.](image/calls-and-functions/parameter-window.png)

Do you notice how the argument slots that the caller sets up and the
parameter slots the callee needs are both in exactly the right order?
How convenient! This is no coincidence. When I talked about each
CallFrame having its own window into the stack, I never said those
windows must be *disjoint*. There’s nothing preventing us from
overlapping them, like this:

![The same stack with the top-level call frame covering the entire stack
and the sum() function's call frame window surrounding fn sum, 5, 6, and
7.](image/calls-and-functions/overlapping-windows.png)

<span id="lua">The</span> top of the caller’s stack contains the
function being called followed by the arguments in order. We know the
caller doesn’t have any other slots above those in use because any
temporaries needed when evaluating argument expressions have been
discarded by now. The bottom of the callee’s stack overlaps so that the
parameter slots exactly line up with where the argument values already
live.

Different bytecode VMs and real CPU architectures have different
*calling conventions*, which is the specific mechanism they use to pass
arguments, store the return address, etc. The mechanism I use here is
based on Lua’s clean, fast virtual machine.

This means that we don’t need to do *any* work to “bind an argument to a
parameter”. There’s no copying values between slots or across
environments. The arguments are already exactly where they need to be.
It’s hard to beat that for performance.

Time to implement the call instruction.

<div class="codehilite">

``` insert-before
      }
```

<div class="source-file">

*vm.c*  
in *run*()

</div>

``` insert
      case OP_CALL: {
        int argCount = READ_BYTE();
        if (!callValue(peek(argCount), argCount)) {
          return INTERPRET_RUNTIME_ERROR;
        }
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

We need to know the function being called and the number of arguments
passed to it. We get the latter from the instruction’s operand. That
also tells us where to find the function on the stack by counting past
the argument slots from the top of the stack. We hand that data off to a
separate `callValue()` function. If that returns `false`, it means the
call caused some sort of runtime error. When that happens, we abort the
interpreter.

If `callValue()` is successful, there will be a new frame on the
CallFrame stack for the called function. The `run()` function has its
own cached pointer to the current frame, so we need to update that.

<div class="codehilite">

``` insert-before
          return INTERPRET_RUNTIME_ERROR;
        }
```

<div class="source-file">

*vm.c*  
in *run*()

</div>

``` insert
        frame = &vm.frames[vm.frameCount - 1];
```

``` insert-after
        break;
```

</div>

<div class="source-file-narrow">

*vm.c*, in *run*()

</div>

Since the bytecode dispatch loop reads from that `frame` variable, when
the VM goes to execute the next instruction, it will read the `ip` from
the newly called function’s CallFrame and jump to its code. The work for
executing that call begins here:

<div class="codehilite">

<div class="source-file">

*vm.c*  
add after *peek*()

</div>

    static bool callValue(Value callee, int argCount) {
      if (IS_OBJ(callee)) {
        switch (OBJ_TYPE(callee)) {
          case OBJ_FUNCTION: 
            return call(AS_FUNCTION(callee), argCount);
          default:
            break; // Non-callable object type.
        }
      }
      runtimeError("Can only call functions and classes.");
      return false;
    }

</div>

<div class="source-file-narrow">

*vm.c*, add after *peek*()

</div>

Using a `switch` statement to check a single type is overkill now, but
will make sense when we add cases to handle other callable types.

There’s more going on here than just initializing a new CallFrame.
Because Lox is dynamically typed, there’s nothing to prevent a user from
writing bad code like:

<div class="codehilite">

    var notAFunction = 123;
    notAFunction();

</div>

If that happens, the runtime needs to safely report an error and halt.
So the first thing we do is check the type of the value that we’re
trying to call. If it’s not a function, we error out. Otherwise, the
actual call happens here:

<div class="codehilite">

<div class="source-file">

*vm.c*  
add after *peek*()

</div>

    static bool call(ObjFunction* function, int argCount) {
      CallFrame* frame = &vm.frames[vm.frameCount++];
      frame->function = function;
      frame->ip = function->chunk.code;
      frame->slots = vm.stackTop - argCount - 1;
      return true;
    }

</div>

<div class="source-file-narrow">

*vm.c*, add after *peek*()

</div>

This simply initializes the next CallFrame on the stack. It stores a
pointer to the function being called and points the frame’s `ip` to the
beginning of the function’s bytecode. Finally, it sets up the `slots`
pointer to give the frame its window into the stack. The arithmetic
there ensures that the arguments already on the stack line up with the
function’s parameters:

![The arithmetic to calculate frame-\>slots from stackTop and
argCount.](image/calls-and-functions/arithmetic.png)

The funny little `- 1` is to account for stack slot zero which the
compiler set aside for when we add methods later. The parameters start
at slot one so we make the window start one slot earlier to align them
with the arguments.

Before we move on, let’s add the new instruction to our disassembler.

<div class="codehilite">

``` insert-before
      return jumpInstruction("OP_LOOP", -1, chunk, offset);
```

<div class="source-file">

*debug.c*  
in *disassembleInstruction*()

</div>

``` insert
    case OP_CALL:
      return byteInstruction("OP_CALL", chunk, offset);
```

``` insert-after
    case OP_RETURN:
```

</div>

<div class="source-file-narrow">

*debug.c*, in *disassembleInstruction*()

</div>

And one more quick side trip. Now that we have a handy function for
initiating a CallFrame, we may as well use it to set up the first frame
for executing the top-level code.

<div class="codehilite">

``` insert-before
  push(OBJ_VAL(function));
```

<div class="source-file">

*vm.c*  
in *interpret*()  
replace 4 lines

</div>

``` insert
  call(function, 0);
```

``` insert-after

  return run();
```

</div>

<div class="source-file-narrow">

*vm.c*, in *interpret*(), replace 4 lines

</div>

OK, now back to calls<span class="ellipse"> . . . </span>

### <a href="#runtime-error-checking" id="runtime-error-checking"><span
class="small">24 . 5 . 2</span>Runtime error checking</a>

The overlapping stack windows work based on the assumption that a call
passes exactly one argument for each of the function’s parameters. But,
again, because Lox ain’t statically typed, a foolish user could pass too
many or too few arguments. In Lox, we’ve defined that to be a runtime
error, which we report like so:

<div class="codehilite">

``` insert-before
static bool call(ObjFunction* function, int argCount) {
```

<div class="source-file">

*vm.c*  
in *call*()

</div>

``` insert
  if (argCount != function->arity) {
    runtimeError("Expected %d arguments but got %d.",
        function->arity, argCount);
    return false;
  }
```

``` insert-after
  CallFrame* frame = &vm.frames[vm.frameCount++];
```

</div>

<div class="source-file-narrow">

*vm.c*, in *call*()

</div>

Pretty straightforward. This is why we store the arity of each function
inside the ObjFunction for it.

There’s another error we need to report that’s less to do with the
user’s foolishness than our own. Because the CallFrame array has a fixed
size, we need to ensure a deep call chain doesn’t overflow it.

<div class="codehilite">

``` insert-before
  }
```

<div class="source-file">

*vm.c*  
in *call*()

</div>

``` insert
  if (vm.frameCount == FRAMES_MAX) {
    runtimeError("Stack overflow.");
    return false;
  }
```

``` insert-after
  CallFrame* frame = &vm.frames[vm.frameCount++];
```

</div>

<div class="source-file-narrow">

*vm.c*, in *call*()

</div>

In practice, if a program gets anywhere close to this limit, there’s
most likely a bug in some runaway recursive code.

### <a href="#printing-stack-traces" id="printing-stack-traces"><span
class="small">24 . 5 . 3</span>Printing stack traces</a>

While we’re on the subject of runtime errors, let’s spend a little time
making them more useful. Stopping on a runtime error is important to
prevent the VM from crashing and burning in some ill-defined way. But
simply aborting doesn’t help the user fix their code that *caused* that
error.

The classic tool to aid debugging runtime failures is a **stack
trace**<span class="em">—</span>a print out of each function that was
still executing when the program died, and where the execution was at
the point that it died. Now that we have a call stack and we’ve
conveniently stored each function’s name, we can show that entire stack
when a runtime error disrupts the harmony of the user’s existence. It
looks like this:

<div class="codehilite">

``` insert-before
  fputs("\n", stderr);
```

<div class="source-file">

*vm.c*  
in *runtimeError*()  
replace 4 lines

</div>

``` insert
  for (int i = vm.frameCount - 1; i >= 0; i--) {
    CallFrame* frame = &vm.frames[i];
    ObjFunction* function = frame->function;
    size_t instruction = frame->ip - function->chunk.code - 1;
    fprintf(stderr, "[line %d] in ", 
            function->chunk.lines[instruction]);
    if (function->name == NULL) {
      fprintf(stderr, "script\n");
    } else {
      fprintf(stderr, "%s()\n", function->name->chars);
    }
  }
```

``` insert-after
  resetStack();
}
```

</div>

<div class="source-file-narrow">

*vm.c*, in *runtimeError*(), replace 4 lines

</div>

The `- 1` is because the IP is already sitting on the next instruction
to be executed but we want the stack trace to point to the previous
failed instruction.

After printing the error message itself, we walk the call stack from
<span id="top">top</span> (the most recently called function) to bottom
(the top-level code). For each frame, we find the line number that
corresponds to the current `ip` inside that frame’s function. Then we
print that line number along with the function name.

There is some disagreement on which order stack frames should be shown
in a trace. Most put the innermost function as the first line and work
their way towards the bottom of the stack. Python prints them out in the
opposite order. So reading from top to bottom tells you how your program
got to where it is, and the last line is where the error actually
occurred.

There’s a logic to that style. It ensures you can always see the
innermost function even if the stack trace is too long to fit on one
screen. On the other hand, the “[inverted
pyramid](https://en.wikipedia.org/wiki/Inverted_pyramid_(journalism))”
from journalism tells us we should put the most important information
*first* in a block of text. In a stack trace, that’s the function where
the error actually occurred. Most other language implementations do
that.

For example, if you run this broken program:

<div class="codehilite">

    fun a() { b(); }
    fun b() { c(); }
    fun c() {
      c("too", "many");
    }

    a();

</div>

It prints out:

<div class="codehilite">

    Expected 0 arguments but got 2.
    [line 4] in c()
    [line 2] in b()
    [line 1] in a()
    [line 7] in script

</div>

That doesn’t look too bad, does it?

### <a href="#returning-from-functions" id="returning-from-functions"><span
class="small">24 . 5 . 4</span>Returning from functions</a>

We’re getting close. We can call functions, and the VM will execute
them. But we can’t *return* from them yet. We’ve had an `OP_RETURN`
instruction for quite some time, but it’s always had some kind of
temporary code hanging out in it just to get us out of the bytecode
loop. The time has arrived for a real implementation.

<div class="codehilite">

``` insert-before
      case OP_RETURN: {
```

<div class="source-file">

*vm.c*  
in *run*()  
replace 2 lines

</div>

``` insert
        Value result = pop();
        vm.frameCount--;
        if (vm.frameCount == 0) {
          pop();
          return INTERPRET_OK;
        }

        vm.stackTop = frame->slots;
        push(result);
        frame = &vm.frames[vm.frameCount - 1];
        break;
```

``` insert-after
      }
```

</div>

<div class="source-file-narrow">

*vm.c*, in *run*(), replace 2 lines

</div>

When a function returns a value, that value will be on top of the stack.
We’re about to discard the called function’s entire stack window, so we
pop that return value off and hang on to it. Then we discard the
CallFrame for the returning function. If that was the very last
CallFrame, it means we’ve finished executing the top-level code. The
entire program is done, so we pop the main script function from the
stack and then exit the interpreter.

Otherwise, we discard all of the slots the callee was using for its
parameters and local variables. That includes the same slots the caller
used to pass the arguments. Now that the call is done, the caller
doesn’t need them anymore. This means the top of the stack ends up right
at the beginning of the returning function’s stack window.

We push the return value back onto the stack at that new, lower
location. Then we update the `run()` function’s cached pointer to the
current frame. Just like when we began a call, on the next iteration of
the bytecode dispatch loop, the VM will read `ip` from that frame, and
execution will jump back to the caller, right where it left off,
immediately after the `OP_CALL` instruction.

![Each step of the return process: popping the return value, discarding
the call frame, pushing the return
value.](image/calls-and-functions/return.png)

Note that we assume here that the function *did* actually return a
value, but a function can implicitly return by reaching the end of its
body:

<div class="codehilite">

    fun noReturn() {
      print "Do stuff";
      // No return here.
    }

    print noReturn(); // ???

</div>

We need to handle that correctly too. The language is specified to
implicitly return `nil` in that case. To make that happen, we add this:

<div class="codehilite">

``` insert-before
static void emitReturn() {
```

<div class="source-file">

*compiler.c*  
in *emitReturn*()

</div>

``` insert
  emitByte(OP_NIL);
```

``` insert-after
  emitByte(OP_RETURN);
}
```

</div>

<div class="source-file-narrow">

*compiler.c*, in *emitReturn*()

</div>

The compiler calls `emitReturn()` to write the `OP_RETURN` instruction
at the end of a function body. Now, before that, it emits an instruction
to push `nil` onto the stack. And with that, we have working function
calls! They can even take parameters! It almost looks like we know what
we’re doing here.

## <a href="#return-statements" id="return-statements"><span
class="small">24 . 6</span>Return Statements</a>

If you want a function that returns something other than the implicit
`nil`, you need a `return` statement. Let’s get that working.

<div class="codehilite">

``` insert-before
    ifStatement();
```

<div class="source-file">

*compiler.c*  
in *statement*()

</div>

``` insert
  } else if (match(TOKEN_RETURN)) {
    returnStatement();
```

``` insert-after
  } else if (match(TOKEN_WHILE)) {
```

</div>

<div class="source-file-narrow">

*compiler.c*, in *statement*()

</div>

When the compiler sees a `return` keyword, it goes here:

<div class="codehilite">

<div class="source-file">

*compiler.c*  
add after *printStatement*()

</div>

    static void returnStatement() {
      if (match(TOKEN_SEMICOLON)) {
        emitReturn();
      } else {
        expression();
        consume(TOKEN_SEMICOLON, "Expect ';' after return value.");
        emitByte(OP_RETURN);
      }
    }

</div>

<div class="source-file-narrow">

*compiler.c*, add after *printStatement*()

</div>

The return value expression is optional, so the parser looks for a
semicolon token to tell if a value was provided. If there is no return
value, the statement implicitly returns `nil`. We implement that by
calling `emitReturn()`, which emits an `OP_NIL` instruction. Otherwise,
we compile the return value expression and return it with an `OP_RETURN`
instruction.

This is the same `OP_RETURN` instruction we’ve already
implemented<span class="em">—</span>we don’t need any new runtime code.
This is quite a difference from jlox. There, we had to use exceptions to
unwind the stack when a `return` statement was executed. That was
because you could return from deep inside some nested blocks. Since jlox
recursively walks the AST, that meant there were a bunch of Java method
calls we needed to escape out of.

Our bytecode compiler flattens that all out. We do recursive descent
during parsing, but at runtime, the VM’s bytecode dispatch loop is
completely flat. There is no recursion going on at the C level at all.
So returning, even from within some nested blocks, is as straightforward
as returning from the end of the function’s body.

We’re not totally done, though. The new `return` statement gives us a
new compile error to worry about. Returns are useful for returning from
functions but the top level of a Lox program is imperative code too. You
shouldn’t be able to <span id="worst">return</span> from there.

<div class="codehilite">

    return "What?!";

</div>

Allowing `return` at the top level isn’t the worst idea in the world. It
would give you a natural way to terminate a script early. You could
maybe even use a returned number to indicate the process’s exit code.

We’ve specified that it’s a compile error to have a `return` statement
outside of any function, which we implement like so:

<div class="codehilite">

``` insert-before
static void returnStatement() {
```

<div class="source-file">

*compiler.c*  
in *returnStatement*()

</div>

``` insert
  if (current->type == TYPE_SCRIPT) {
    error("Can't return from top-level code.");
  }
```

``` insert-after
  if (match(TOKEN_SEMICOLON)) {
```

</div>

<div class="source-file-narrow">

*compiler.c*, in *returnStatement*()

</div>

This is one of the reasons we added that FunctionType enum to the
compiler.

## <a href="#native-functions" id="native-functions"><span
class="small">24 . 7</span>Native Functions</a>

Our VM is getting more powerful. We’ve got functions, calls, parameters,
returns. You can define lots of different functions that can call each
other in interesting ways. But, ultimately, they can’t really *do*
anything. The only user-visible thing a Lox program can do, regardless
of its complexity, is print. To add more capabilities, we need to expose
them to the user.

A programming language implementation reaches out and touches the
material world through **native functions**. If you want to be able to
write programs that check the time, read user input, or access the file
system, we need to add native functions<span class="em">—</span>callable
from Lox but implemented in C<span class="em">—</span>that expose those
capabilities.

At the language level, Lox is fairly
complete<span class="em">—</span>it’s got closures, classes,
inheritance, and other fun stuff. One reason it feels like a toy
language is because it has almost no native capabilities. We could turn
it into a real language by adding a long list of them.

However, grinding through a pile of OS operations isn’t actually very
educational. Once you’ve seen how to bind one piece of C code to Lox,
you get the idea. But you do need to see *one*, and even a single native
function requires us to build out all the machinery for interfacing Lox
with C. So we’ll go through that and do all the hard work. Then, when
that’s done, we’ll add one tiny native function just to prove that it
works.

The reason we need new machinery is because, from the implementation’s
perspective, native functions are different from Lox functions. When
they are called, they don’t push a CallFrame, because there’s no
bytecode code for that frame to point to. They have no bytecode chunk.
Instead, they somehow reference a piece of native C code.

We handle this in clox by defining native functions as an entirely
different object type.

<div class="codehilite">

``` insert-before
} ObjFunction;
```

<div class="source-file">

*object.h*  
add after struct *ObjFunction*

</div>

``` insert

typedef Value (*NativeFn)(int argCount, Value* args);

typedef struct {
  Obj obj;
  NativeFn function;
} ObjNative;
```

``` insert-after

struct ObjString {
```

</div>

<div class="source-file-narrow">

*object.h*, add after struct *ObjFunction*

</div>

The representation is simpler than
ObjFunction<span class="em">—</span>merely an Obj header and a pointer
to the C function that implements the native behavior. The native
function takes the argument count and a pointer to the first argument on
the stack. It accesses the arguments through that pointer. Once it’s
done, it returns the result value.

As always, a new object type carries some accoutrements with it. To
create an ObjNative, we declare a constructor-like function.

<div class="codehilite">

``` insert-before
ObjFunction* newFunction();
```

<div class="source-file">

*object.h*  
add after *newFunction*()

</div>

``` insert
ObjNative* newNative(NativeFn function);
```

``` insert-after
ObjString* takeString(char* chars, int length);
```

</div>

<div class="source-file-narrow">

*object.h*, add after *newFunction*()

</div>

We implement that like so:

<div class="codehilite">

<div class="source-file">

*object.c*  
add after *newFunction*()

</div>

    ObjNative* newNative(NativeFn function) {
      ObjNative* native = ALLOCATE_OBJ(ObjNative, OBJ_NATIVE);
      native->function = function;
      return native;
    }

</div>

<div class="source-file-narrow">

*object.c*, add after *newFunction*()

</div>

The constructor takes a C function pointer to wrap in an ObjNative. It
sets up the object header and stores the function. For the header, we
need a new object type.

<div class="codehilite">

``` insert-before
typedef enum {
  OBJ_FUNCTION,
```

<div class="source-file">

*object.h*  
in enum *ObjType*

</div>

``` insert
  OBJ_NATIVE,
```

``` insert-after
  OBJ_STRING,
} ObjType;
```

</div>

<div class="source-file-narrow">

*object.h*, in enum *ObjType*

</div>

The VM also needs to know how to deallocate a native function object.

<div class="codehilite">

``` insert-before
    }
```

<div class="source-file">

*memory.c*  
in *freeObject*()

</div>

``` insert
    case OBJ_NATIVE:
      FREE(ObjNative, object);
      break;
```

``` insert-after
    case OBJ_STRING: {
```

</div>

<div class="source-file-narrow">

*memory.c*, in *freeObject*()

</div>

There isn’t much here since ObjNative doesn’t own any extra memory. The
other capability all Lox objects support is being printed.

<div class="codehilite">

``` insert-before
      break;
```

<div class="source-file">

*object.c*  
in *printObject*()

</div>

``` insert
    case OBJ_NATIVE:
      printf("<native fn>");
      break;
```

``` insert-after
    case OBJ_STRING:
```

</div>

<div class="source-file-narrow">

*object.c*, in *printObject*()

</div>

In order to support dynamic typing, we have a macro to see if a value is
a native function.

<div class="codehilite">

``` insert-before
#define IS_FUNCTION(value)     isObjType(value, OBJ_FUNCTION)
```

<div class="source-file">

*object.h*

</div>

``` insert
#define IS_NATIVE(value)       isObjType(value, OBJ_NATIVE)
```

``` insert-after
#define IS_STRING(value)       isObjType(value, OBJ_STRING)
```

</div>

<div class="source-file-narrow">

*object.h*

</div>

Assuming that returns true, this macro extracts the C function pointer
from a Value representing a native function:

<div class="codehilite">

``` insert-before
#define AS_FUNCTION(value)     ((ObjFunction*)AS_OBJ(value))
```

<div class="source-file">

*object.h*

</div>

``` insert
#define AS_NATIVE(value) \
    (((ObjNative*)AS_OBJ(value))->function)
```

``` insert-after
#define AS_STRING(value)       ((ObjString*)AS_OBJ(value))
```

</div>

<div class="source-file-narrow">

*object.h*

</div>

All of this baggage lets the VM treat native functions like any other
object. You can store them in variables, pass them around, throw them
birthday parties, etc. Of course, the operation we actually care about
is *calling* them<span class="em">—</span>using one as the left-hand
operand in a call expression.

Over in `callValue()` we add another type case.

<div class="codehilite">

``` insert-before
      case OBJ_FUNCTION: 
        return call(AS_FUNCTION(callee), argCount);
```

<div class="source-file">

*vm.c*  
in *callValue*()

</div>

``` insert
      case OBJ_NATIVE: {
        NativeFn native = AS_NATIVE(callee);
        Value result = native(argCount, vm.stackTop - argCount);
        vm.stackTop -= argCount + 1;
        push(result);
        return true;
      }
```

``` insert-after
      default:
```

</div>

<div class="source-file-narrow">

*vm.c*, in *callValue*()

</div>

If the object being called is a native function, we invoke the C
function right then and there. There’s no need to muck with CallFrames
or anything. We just hand off to C, get the result, and stuff it back in
the stack. This makes native functions as fast as we can get.

With this, users should be able to call native functions, but there
aren’t any to call. Without something like a foreign function interface,
users can’t define their own native functions. That’s our job as VM
implementers. We’ll start with a helper to define a new native function
exposed to Lox programs.

<div class="codehilite">

<div class="source-file">

*vm.c*  
add after *runtimeError*()

</div>

    static void defineNative(const char* name, NativeFn function) {
      push(OBJ_VAL(copyString(name, (int)strlen(name))));
      push(OBJ_VAL(newNative(function)));
      tableSet(&vm.globals, AS_STRING(vm.stack[0]), vm.stack[1]);
      pop();
      pop();
    }

</div>

<div class="source-file-narrow">

*vm.c*, add after *runtimeError*()

</div>

It takes a pointer to a C function and the name it will be known as in
Lox. We wrap the function in an ObjNative and then store that in a
global variable with the given name.

You’re probably wondering why we push and pop the name and function on
the stack. That looks weird, right? This is the kind of stuff you have
to worry about when <span id="worry">garbage</span> collection gets
involved. Both `copyString()` and `newNative()` dynamically allocate
memory. That means once we have a GC, they can potentially trigger a
collection. If that happens, we need to ensure the collector knows we’re
not done with the name and ObjFunction so that it doesn’t free them out
from under us. Storing them on the value stack accomplishes that.

Don’t worry if you didn’t follow all that. It will make a lot more sense
once we get around to [implementing the GC](garbage-collection.html).

It feels silly, but after all of that work, we’re going to add only one
little native function.

<div class="codehilite">

<div class="source-file">

*vm.c*  
add after variable *vm*

</div>

    static Value clockNative(int argCount, Value* args) {
      return NUMBER_VAL((double)clock() / CLOCKS_PER_SEC);
    }

</div>

<div class="source-file-narrow">

*vm.c*, add after variable *vm*

</div>

This returns the elapsed time since the program started running, in
seconds. It’s handy for benchmarking Lox programs. In Lox, we’ll name it
`clock()`.

<div class="codehilite">

``` insert-before
  initTable(&vm.strings);
```

<div class="source-file">

*vm.c*  
in *initVM*()

</div>

``` insert

  defineNative("clock", clockNative);
```

``` insert-after
}
```

</div>

<div class="source-file-narrow">

*vm.c*, in *initVM*()

</div>

To get to the C standard library `clock()` function, the “vm” module
needs an include.

<div class="codehilite">

``` insert-before
#include <string.h>
```

<div class="source-file">

*vm.c*

</div>

``` insert
#include <time.h>
```

``` insert-after

#include "common.h"
```

</div>

<div class="source-file-narrow">

*vm.c*

</div>

That was a lot of material to work through, but we did it! Type this in
and try it out:

<div class="codehilite">

    fun fib(n) {
      if (n < 2) return n;
      return fib(n - 2) + fib(n - 1);
    }

    var start = clock();
    print fib(35);
    print clock() - start;

</div>

We can write a really inefficient recursive Fibonacci function. Even
better, we can measure just <span id="faster">*how*</span> inefficient
it is. This is, of course, not the smartest way to calculate a Fibonacci
number. But it is a good way to stress test a language implementation’s
support for function calls. On my machine, running this in clox is about
five times faster than in jlox. That’s quite an improvement.

It’s a little slower than a comparable Ruby program run in Ruby
2.4.3p205, and about 3x faster than one run in Python 3.7.3. And we
still have a lot of simple optimizations we can do in our VM.

<div class="challenges">

## <a href="#challenges" id="challenges">Challenges</a>

1.  Reading and writing the `ip` field is one of the most frequent
    operations inside the bytecode loop. Right now, we access it through
    a pointer to the current CallFrame. That requires a pointer
    indirection which may force the CPU to bypass the cache and hit main
    memory. That can be a real performance sink.

    Ideally, we’d keep the `ip` in a native CPU register. C doesn’t let
    us *require* that without dropping into inline assembly, but we can
    structure the code to encourage the compiler to make that
    optimization. If we store the `ip` directly in a C local variable
    and mark it `register`, there’s a good chance the C compiler will
    accede to our polite request.

    This does mean we need to be careful to load and store the local
    `ip` back into the correct CallFrame when starting and ending
    function calls. Implement this optimization. Write a couple of
    benchmarks and see how it affects the performance. Do you think the
    extra code complexity is worth it?

2.  Native function calls are fast in part because we don’t validate
    that the call passes as many arguments as the function expects. We
    really should, or an incorrect call to a native function without
    enough arguments could cause the function to read uninitialized
    memory. Add arity checking.

3.  Right now, there’s no way for a native function to signal a runtime
    error. In a real implementation, this is something we’d need to
    support because native functions live in the statically typed world
    of C but are called from dynamically typed Lox land. If a user, say,
    tries to pass a string to `sqrt()`, that native function needs to
    report a runtime error.

    Extend the native function system to support that. How does this
    capability affect the performance of native calls?

4.  Add some more native functions to do things you find useful. Write
    some programs using those. What did you add? How do they affect the
    feel of the language and how practical it is?

</div>

<a href="closures.html" class="next">Next Chapter: “Closures” →</a>
Handcrafted by Robert Nystrom — <a
href="https://github.com/munificent/craftinginterpreters/blob/master/LICENSE"
target="_blank">© 2015 – 2021</a>

</div>
