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
