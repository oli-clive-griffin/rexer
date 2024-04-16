[![](image/logotype.png "Crafting Interpreters")](/)

<div class="contents">

### [Classes and Instances<span class="small">27</span>](#top)

- [<span class="small">27.1</span> Class Objects](#class-objects)
- [<span class="small">27.2</span> Class
  Declarations](#class-declarations)
- [<span class="small">27.3</span> Instances of
  Classes](#instances-of-classes)
- [<span class="small">27.4</span> Get and Set
  Expressions](#get-and-set-expressions)
- 
- [Challenges](#challenges)

<div class="prev-next">

<a href="garbage-collection.html" class="left"
title="Garbage Collection">← Previous</a>
[↑ Up](a-bytecode-virtual-machine.html "A Bytecode Virtual Machine")
<a href="methods-and-initializers.html" class="right"
title="Methods and Initializers">Next →</a>

</div>

</div>

[![](image/logotype.png "Crafting Interpreters")](/)
<a href="garbage-collection.html" class="prev"
title="Garbage Collection">←</a>
<a href="methods-and-initializers.html" class="next"
title="Methods and Initializers">→</a>

<div class="page">

<div class="nav-wrapper">

[![](image/logotype.png "Crafting Interpreters")](/)

<div class="expandable">

### [Classes and Instances<span class="small">27</span>](#top)

- [<span class="small">27.1</span> Class Objects](#class-objects)
- [<span class="small">27.2</span> Class
  Declarations](#class-declarations)
- [<span class="small">27.3</span> Instances of
  Classes](#instances-of-classes)
- [<span class="small">27.4</span> Get and Set
  Expressions](#get-and-set-expressions)
- 
- [Challenges](#challenges)

<div class="prev-next">

<a href="garbage-collection.html" class="left"
title="Garbage Collection">← Previous</a>
[↑ Up](a-bytecode-virtual-machine.html "A Bytecode Virtual Machine")
<a href="methods-and-initializers.html" class="right"
title="Methods and Initializers">Next →</a>

</div>

</div>

<span id="expand-nav">≡</span>

</div>

<div class="number">

27

</div>

# Classes and Instances

> Caring too much for objects can destroy you.
> Only<span class="em">—</span>if you care for a thing enough, it takes
> on a life of its own, doesn’t it? And isn’t the whole point of
> things<span class="em">—</span>beautiful
> things<span class="em">—</span>that they connect you to some larger
> beauty?
>
> Donna Tartt, *The Goldfinch*

The last area left to implement in clox is object-oriented programming.
<span id="oop">OOP</span> is a bundle of intertwined features: classes,
instances, fields, methods, initializers, and inheritance. Using
relatively high-level Java, we packed all that into two chapters. Now
that we’re coding in C, which feels like building a model of the Eiffel
tower out of toothpicks, we’ll devote three chapters to covering the
same territory. This makes for a leisurely stroll through the
implementation. After strenuous chapters like [closures](closures.html)
and the [garbage collector](garbage-collection.html), you have earned a
rest. In fact, the book should be easy from here on out.

People who have strong opinions about object-oriented
programming<span class="em">—</span>read
“everyone”<span class="em">—</span>tend to assume OOP means some very
specific list of language features, but really there’s a whole space to
explore, and each language has its own ingredients and recipes.

Self has objects but no classes. CLOS has methods but doesn’t attach
them to specific classes. C++ initially had no runtime
polymorphism<span class="em">—</span>no virtual methods. Python has
multiple inheritance, but Java does not. Ruby attaches methods to
classes, but you can also define methods on a single object.

In this chapter, we cover the first three features: classes, instances,
and fields. This is the stateful side of object orientation. Then in the
next two chapters, we will hang behavior and code reuse off of those
objects.

## <a href="#class-objects" id="class-objects"><span
class="small">27 . 1</span>Class Objects</a>

In a class-based object-oriented language, everything begins with
classes. They define what sorts of objects exist in the program and are
the factories used to produce new instances. Going bottom-up, we’ll
start with their runtime representation and then hook that into the
language.

By this point, we’re well-acquainted with the process of adding a new
object type to the VM. We start with a struct.

<div class="codehilite">

``` insert-before
} ObjClosure;
```

<div class="source-file">

*object.h*  
add after struct *ObjClosure*

</div>

``` insert

typedef struct {
  Obj obj;
  ObjString* name;
} ObjClass;
```

``` insert-after

ObjClosure* newClosure(ObjFunction* function);
```

</div>

<div class="source-file-narrow">

*object.h*, add after struct *ObjClosure*

</div>

After the Obj header, we store the class’s name. This isn’t strictly
needed for the user’s program, but it lets us show the name at runtime
for things like stack traces.

The new type needs a corresponding case in the ObjType enum.

<div class="codehilite">

``` insert-before
typedef enum {
```

<div class="source-file">

*object.h*  
in enum *ObjType*

</div>

``` insert
  OBJ_CLASS,
```

``` insert-after
  OBJ_CLOSURE,
```

</div>

<div class="source-file-narrow">

*object.h*, in enum *ObjType*

</div>

And that type gets a corresponding pair of macros. First, for testing an
object’s type:

<div class="codehilite">

``` insert-before
#define OBJ_TYPE(value)        (AS_OBJ(value)->type)
```

<div class="source-file">

*object.h*

</div>

``` insert
#define IS_CLASS(value)        isObjType(value, OBJ_CLASS)
```

``` insert-after
#define IS_CLOSURE(value)      isObjType(value, OBJ_CLOSURE)
```

</div>

<div class="source-file-narrow">

*object.h*

</div>

And then for casting a Value to an ObjClass pointer:

<div class="codehilite">

``` insert-before
#define IS_STRING(value)       isObjType(value, OBJ_STRING)
```

<div class="source-file">

*object.h*

</div>

``` insert
#define AS_CLASS(value)        ((ObjClass*)AS_OBJ(value))
```

``` insert-after
#define AS_CLOSURE(value)      ((ObjClosure*)AS_OBJ(value))
```

</div>

<div class="source-file-narrow">

*object.h*

</div>

The VM creates new class objects using this function:

<div class="codehilite">

``` insert-before
} ObjClass;
```

<div class="source-file">

*object.h*  
add after struct *ObjClass*

</div>

``` insert
ObjClass* newClass(ObjString* name);
```

``` insert-after
ObjClosure* newClosure(ObjFunction* function);
```

</div>

<div class="source-file-narrow">

*object.h*, add after struct *ObjClass*

</div>

The implementation lives over here:

<div class="codehilite">

<div class="source-file">

*object.c*  
add after *allocateObject*()

</div>

    ObjClass* newClass(ObjString* name) {
      ObjClass* klass = ALLOCATE_OBJ(ObjClass, OBJ_CLASS);
      klass->name = name; 
      return klass;
    }

</div>

<div class="source-file-narrow">

*object.c*, add after *allocateObject*()

</div>

Pretty much all boilerplate. It takes in the class’s name as a string
and stores it. Every time the user declares a new class, the VM will
create a new one of these ObjClass structs to represent it.

!['Klass' in a zany kidz font.](image/classes-and-instances/klass.png)

I named the variable “klass” not just to give the VM a zany preschool
“Kidz Korner” feel. It makes it easier to get clox compiling as C++
where “class” is a reserved word.

When the VM no longer needs a class, it frees it like so:

<div class="codehilite">

``` insert-before
  switch (object->type) {
```

<div class="source-file">

*memory.c*  
in *freeObject*()

</div>

``` insert
    case OBJ_CLASS: {
      FREE(ObjClass, object);
      break;
    } 
```

``` insert-after
    case OBJ_CLOSURE: {
```

</div>

<div class="source-file-narrow">

*memory.c*, in *freeObject*()

</div>

The braces here are pointless now, but will be useful in the next
chapter when we add some more code to the switch case.

We have a memory manager now, so we also need to support tracing through
class objects.

<div class="codehilite">

``` insert-before
  switch (object->type) {
```

<div class="source-file">

*memory.c*  
in *blackenObject*()

</div>

``` insert
    case OBJ_CLASS: {
      ObjClass* klass = (ObjClass*)object;
      markObject((Obj*)klass->name);
      break;
    }
```

``` insert-after
    case OBJ_CLOSURE: {
```

</div>

<div class="source-file-narrow">

*memory.c*, in *blackenObject*()

</div>

When the GC reaches a class object, it marks the class’s name to keep
that string alive too.

The last operation the VM can perform on a class is printing it.

<div class="codehilite">

``` insert-before
  switch (OBJ_TYPE(value)) {
```

<div class="source-file">

*object.c*  
in *printObject*()

</div>

``` insert
    case OBJ_CLASS:
      printf("%s", AS_CLASS(value)->name->chars);
      break;
```

``` insert-after
    case OBJ_CLOSURE:
```

</div>

<div class="source-file-narrow">

*object.c*, in *printObject*()

</div>

A class simply says its own name.

## <a href="#class-declarations" id="class-declarations"><span
class="small">27 . 2</span>Class Declarations</a>

Runtime representation in hand, we are ready to add support for classes
to the language. Next, we move into the parser.

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
  if (match(TOKEN_CLASS)) {
    classDeclaration();
  } else if (match(TOKEN_FUN)) {
```

``` insert-after
    funDeclaration();
```

</div>

<div class="source-file-narrow">

*compiler.c*, in *declaration*(), replace 1 line

</div>

Class declarations are statements, and the parser recognizes one by the
leading `class` keyword. The rest of the compilation happens over here:

<div class="codehilite">

<div class="source-file">

*compiler.c*  
add after *function*()

</div>

    static void classDeclaration() {
      consume(TOKEN_IDENTIFIER, "Expect class name.");
      uint8_t nameConstant = identifierConstant(&parser.previous);
      declareVariable();

      emitBytes(OP_CLASS, nameConstant);
      defineVariable(nameConstant);

      consume(TOKEN_LEFT_BRACE, "Expect '{' before class body.");
      consume(TOKEN_RIGHT_BRACE, "Expect '}' after class body.");
    }

</div>

<div class="source-file-narrow">

*compiler.c*, add after *function*()

</div>

Immediately after the `class` keyword is the class’s name. We take that
identifier and add it to the surrounding function’s constant table as a
string. As you just saw, printing a class shows its name, so the
compiler needs to stuff the name string somewhere that the runtime can
find. The constant table is the way to do that.

The class’s <span id="variable">name</span> is also used to bind the
class object to a variable of the same name. So we declare a variable
with that identifier right after consuming its token.

We could have made class declarations be *expressions* instead of
statements<span class="em">—</span>they are essentially a literal that
produces a value after all. Then users would have to explicitly bind the
class to a variable themselves like:

<div class="codehilite">

    var Pie = class {}

</div>

Sort of like lambda functions but for classes. But since we generally
want classes to be named anyway, it makes sense to treat them as
declarations.

Next, we emit a new instruction to actually create the class object at
runtime. That instruction takes the constant table index of the class’s
name as an operand.

After that, but before compiling the body of the class, we define the
variable for the class’s name. *Declaring* the variable adds it to the
scope, but recall from [a previous
chapter](local-variables.html#another-scope-edge-case) that we can’t
*use* the variable until it’s *defined*. For classes, we define the
variable before the body. That way, users can refer to the containing
class inside the bodies of its own methods. That’s useful for things
like factory methods that produce new instances of the class.

Finally, we compile the body. We don’t have methods yet, so right now
it’s simply an empty pair of braces. Lox doesn’t require fields to be
declared in the class, so we’re done with the
body<span class="em">—</span>and the parser<span class="em">—</span>for
now.

The compiler is emitting a new instruction, so let’s define that.

<div class="codehilite">

``` insert-before
  OP_RETURN,
```

<div class="source-file">

*chunk.h*  
in enum *OpCode*

</div>

``` insert
  OP_CLASS,
```

``` insert-after
} OpCode;
```

</div>

<div class="source-file-narrow">

*chunk.h*, in enum *OpCode*

</div>

And add it to the disassembler:

<div class="codehilite">

``` insert-before
    case OP_RETURN:
      return simpleInstruction("OP_RETURN", offset);
```

<div class="source-file">

*debug.c*  
in *disassembleInstruction*()

</div>

``` insert
    case OP_CLASS:
      return constantInstruction("OP_CLASS", chunk, offset);
```

``` insert-after
    default:
```

</div>

<div class="source-file-narrow">

*debug.c*, in *disassembleInstruction*()

</div>

For such a large-seeming feature, the interpreter support is minimal.

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
      case OP_CLASS:
        push(OBJ_VAL(newClass(READ_STRING())));
        break;
```

``` insert-after
    }
```

</div>

<div class="source-file-narrow">

*vm.c*, in *run*()

</div>

We load the string for the class’s name from the constant table and pass
that to `newClass()`. That creates a new class object with the given
name. We push that onto the stack and we’re good. If the class is bound
to a global variable, then the compiler’s call to `defineVariable()`
will emit code to store that object from the stack into the global
variable table. Otherwise, it’s right where it needs to be on the stack
for a new <span id="local">local</span> variable.

“Local” classes<span class="em">—</span>classes declared inside the body
of a function or block, are an unusual concept. Many languages don’t
allow them at all. But since Lox is a dynamically typed scripting
language, it treats the top level of a program and the bodies of
functions and blocks uniformly. Classes are just another kind of
declaration, and since you can declare variables and functions inside
blocks, you can declare classes in there too.

There you have it, our VM supports classes now. You can run this:

<div class="codehilite">

    class Brioche {}
    print Brioche;

</div>

Unfortunately, printing is about *all* you can do with classes, so next
is making them more useful.

## <a href="#instances-of-classes" id="instances-of-classes"><span
class="small">27 . 3</span>Instances of Classes</a>

Classes serve two main purposes in a language:

- **They are how you create new instances.** Sometimes this involves a
  `new` keyword, other times it’s a method call on the class object, but
  you usually mention the class by name *somehow* to get a new instance.

- **They contain methods.** These define how all instances of the class
  behave.

We won’t get to methods until the next chapter, so for now we will only
worry about the first part. Before classes can create instances, we need
a representation for them.

<div class="codehilite">

``` insert-before
} ObjClass;
```

<div class="source-file">

*object.h*  
add after struct *ObjClass*

</div>

``` insert

typedef struct {
  Obj obj;
  ObjClass* klass;
  Table fields; 
} ObjInstance;
```

``` insert-after

ObjClass* newClass(ObjString* name);
```

</div>

<div class="source-file-narrow">

*object.h*, add after struct *ObjClass*

</div>

Instances know their class<span class="em">—</span>each instance has a
pointer to the class that it is an instance of. We won’t use this much
in this chapter, but it will become critical when we add methods.

More important to this chapter is how instances store their state. Lox
lets users freely add fields to an instance at runtime. This means we
need a storage mechanism that can grow. We could use a dynamic array,
but we also want to look up fields by name as quickly as possible.
There’s a data structure that’s just perfect for quickly accessing a set
of values by name and<span class="em">—</span>even more
conveniently<span class="em">—</span>we’ve already implemented it. Each
instance stores its fields using a hash table.

Being able to freely add fields to an object at runtime is a big
practical difference between most dynamic and static languages.
Statically typed languages usually require fields to be explicitly
declared. This way, the compiler knows exactly what fields each instance
has. It can use that to determine the precise amount of memory needed
for each instance and the offsets in that memory where each field can be
found.

In Lox and other dynamic languages, accessing a field is usually a hash
table lookup. Constant time, but still pretty heavyweight. In a language
like C++, accessing a field is as fast as offsetting a pointer by an
integer constant.

We only need to add an include, and we’ve got it.

<div class="codehilite">

``` insert-before
#include "chunk.h"
```

<div class="source-file">

*object.h*

</div>

``` insert
#include "table.h"
```

``` insert-after
#include "value.h"
```

</div>

<div class="source-file-narrow">

*object.h*

</div>

This new struct gets a new object type.

<div class="codehilite">

``` insert-before
  OBJ_FUNCTION,
```

<div class="source-file">

*object.h*  
in enum *ObjType*

</div>

``` insert
  OBJ_INSTANCE,
```

``` insert-after
  OBJ_NATIVE,
```

</div>

<div class="source-file-narrow">

*object.h*, in enum *ObjType*

</div>

I want to slow down a bit here because the Lox *language’s* notion of
“type” and the VM *implementation’s* notion of “type” brush against each
other in ways that can be confusing. Inside the C code that makes clox,
there are a number of different types of
Obj<span class="em">—</span>ObjString, ObjClosure, etc. Each has its own
internal representation and semantics.

In the Lox *language*, users can define their own
classes<span class="em">—</span>say Cake and
Pie<span class="em">—</span>and then create instances of those classes.
From the user’s perspective, an instance of Cake is a different type of
object than an instance of Pie. But, from the VM’s perspective, every
class the user defines is simply another value of type ObjClass.
Likewise, each instance in the user’s program, no matter what class it
is an instance of, is an ObjInstance. That one VM object type covers
instances of all classes. The two worlds map to each other something
like this:

![A set of class declarations and instances, and the runtime
representations each maps to.](image/classes-and-instances/lox-clox.png)

Got it? OK, back to the implementation. We also get our usual macros.

<div class="codehilite">

``` insert-before
#define IS_FUNCTION(value)     isObjType(value, OBJ_FUNCTION)
```

<div class="source-file">

*object.h*

</div>

``` insert
#define IS_INSTANCE(value)     isObjType(value, OBJ_INSTANCE)
```

``` insert-after
#define IS_NATIVE(value)       isObjType(value, OBJ_NATIVE)
```

</div>

<div class="source-file-narrow">

*object.h*

</div>

And:

<div class="codehilite">

``` insert-before
#define AS_FUNCTION(value)     ((ObjFunction*)AS_OBJ(value))
```

<div class="source-file">

*object.h*

</div>

``` insert
#define AS_INSTANCE(value)     ((ObjInstance*)AS_OBJ(value))
```

``` insert-after
#define AS_NATIVE(value) \
```

</div>

<div class="source-file-narrow">

*object.h*

</div>

Since fields are added after the instance is created, the “constructor”
function only needs to know the class.

<div class="codehilite">

``` insert-before
ObjFunction* newFunction();
```

<div class="source-file">

*object.h*  
add after *newFunction*()

</div>

``` insert
ObjInstance* newInstance(ObjClass* klass);
```

``` insert-after
ObjNative* newNative(NativeFn function);
```

</div>

<div class="source-file-narrow">

*object.h*, add after *newFunction*()

</div>

We implement that function here:

<div class="codehilite">

<div class="source-file">

*object.c*  
add after *newFunction*()

</div>

    ObjInstance* newInstance(ObjClass* klass) {
      ObjInstance* instance = ALLOCATE_OBJ(ObjInstance, OBJ_INSTANCE);
      instance->klass = klass;
      initTable(&instance->fields);
      return instance;
    }

</div>

<div class="source-file-narrow">

*object.c*, add after *newFunction*()

</div>

We store a reference to the instance’s class. Then we initialize the
field table to an empty hash table. A new baby object is born!

At the sadder end of the instance’s lifespan, it gets freed.

<div class="codehilite">

``` insert-before
      FREE(ObjFunction, object);
      break;
    }
```

<div class="source-file">

*memory.c*  
in *freeObject*()

</div>

``` insert
    case OBJ_INSTANCE: {
      ObjInstance* instance = (ObjInstance*)object;
      freeTable(&instance->fields);
      FREE(ObjInstance, object);
      break;
    }
```

``` insert-after
    case OBJ_NATIVE:
```

</div>

<div class="source-file-narrow">

*memory.c*, in *freeObject*()

</div>

The instance owns its field table so when freeing the instance, we also
free the table. We don’t explicitly free the entries *in* the table,
because there may be other references to those objects. The garbage
collector will take care of those for us. Here we free only the entry
array of the table itself.

Speaking of the garbage collector, it needs support for tracing through
instances.

<div class="codehilite">

``` insert-before
      markArray(&function->chunk.constants);
      break;
    }
```

<div class="source-file">

*memory.c*  
in *blackenObject*()

</div>

``` insert
    case OBJ_INSTANCE: {
      ObjInstance* instance = (ObjInstance*)object;
      markObject((Obj*)instance->klass);
      markTable(&instance->fields);
      break;
    }
```

``` insert-after
    case OBJ_UPVALUE:
```

</div>

<div class="source-file-narrow">

*memory.c*, in *blackenObject*()

</div>

If the instance is alive, we need to keep its class around. Also, we
need to keep every object referenced by the instance’s fields. Most live
objects that are not roots are reachable because some instance refers to
the object in a field. Fortunately, we already have a nice `markTable()`
function to make tracing them easy.

Less critical but still important is printing.

<div class="codehilite">

``` insert-before
      break;
```

<div class="source-file">

*object.c*  
in *printObject*()

</div>

``` insert
    case OBJ_INSTANCE:
      printf("%s instance",
             AS_INSTANCE(value)->klass->name->chars);
      break;
```

``` insert-after
    case OBJ_NATIVE:
```

</div>

<div class="source-file-narrow">

*object.c*, in *printObject*()

</div>

<span id="print">An</span> instance prints its name followed by
“instance”. (The “instance” part is mainly so that classes and instances
don’t print the same.)

Most object-oriented languages let a class define some sort of
`toString()` method that lets the class specify how its instances are
converted to a string and printed. If Lox was less of a toy language, I
would want to support that too.

The real fun happens over in the interpreter. Lox has no special `new`
keyword. The way to create an instance of a class is to invoke the class
itself as if it were a function. The runtime already supports function
calls, and it checks the type of object being called to make sure the
user doesn’t try to invoke a number or other invalid type.

We extend that runtime checking with a new case.

<div class="codehilite">

``` insert-before
    switch (OBJ_TYPE(callee)) {
```

<div class="source-file">

*vm.c*  
in *callValue*()

</div>

``` insert
      case OBJ_CLASS: {
        ObjClass* klass = AS_CLASS(callee);
        vm.stackTop[-argCount - 1] = OBJ_VAL(newInstance(klass));
        return true;
      }
```

``` insert-after
      case OBJ_CLOSURE:
```

</div>

<div class="source-file-narrow">

*vm.c*, in *callValue*()

</div>

If the value being called<span class="em">—</span>the object that
results when evaluating the expression to the left of the opening
parenthesis<span class="em">—</span>is a class, then we treat it as a
constructor call. We <span id="args">create</span> a new instance of the
called class and store the result on the stack.

We ignore any arguments passed to the call for now. We’ll revisit this
code in the [next chapter](methods-and-initializers.html) when we add
support for initializers.

We’re one step farther. Now we can define classes and create instances
of them.

<div class="codehilite">

    class Brioche {}
    print Brioche();

</div>

Note the parentheses after `Brioche` on the second line now. This prints
“Brioche instance”.

## <a href="#get-and-set-expressions" id="get-and-set-expressions"><span
class="small">27 . 4</span>Get and Set Expressions</a>

Our object representation for instances can already store state, so all
that remains is exposing that functionality to the user. Fields are
accessed and modified using get and set expressions. Not one to break
with tradition, Lox uses the classic “dot” syntax:

<div class="codehilite">

    eclair.filling = "pastry creme";
    print eclair.filling;

</div>

The period<span class="em">—</span>full stop for my English
friends<span class="em">—</span>works <span id="sort">sort</span> of
like an infix operator. There is an expression to the left that is
evaluated first and produces an instance. After that is the `.` followed
by a field name. Since there is a preceding operand, we hook this into
the parse table as an infix expression.

I say “sort of” because the right-hand side after the `.` is not an
expression, but a single identifier whose semantics are handled by the
get or set expression itself. It’s really closer to a postfix
expression.

<div class="codehilite">

``` insert-before
  [TOKEN_COMMA]         = {NULL,     NULL,   PREC_NONE},
```

<div class="source-file">

*compiler.c*  
replace 1 line

</div>

``` insert
  [TOKEN_DOT]           = {NULL,     dot,    PREC_CALL},
```

``` insert-after
  [TOKEN_MINUS]         = {unary,    binary, PREC_TERM},
```

</div>

<div class="source-file-narrow">

*compiler.c*, replace 1 line

</div>

As in other languages, the `.` operator binds tightly, with precedence
as high as the parentheses in a function call. After the parser consumes
the dot token, it dispatches to a new parse function.

<div class="codehilite">

<div class="source-file">

*compiler.c*  
add after *call*()

</div>

    static void dot(bool canAssign) {
      consume(TOKEN_IDENTIFIER, "Expect property name after '.'.");
      uint8_t name = identifierConstant(&parser.previous);

      if (canAssign && match(TOKEN_EQUAL)) {
        expression();
        emitBytes(OP_SET_PROPERTY, name);
      } else {
        emitBytes(OP_GET_PROPERTY, name);
      }
    }

</div>

<div class="source-file-narrow">

*compiler.c*, add after *call*()

</div>

The parser expects to find a <span id="prop">property</span> name
immediately after the dot. We load that token’s lexeme into the constant
table as a string so that the name is available at runtime.

The compiler uses “property” instead of “field” here because, remember,
Lox also lets you use dot syntax to access a method without calling it.
“Property” is the general term we use to refer to any named entity you
can access on an instance. Fields are the subset of properties that are
backed by the instance’s state.

We have two new expression forms<span class="em">—</span>getters and
setters<span class="em">—</span>that this one function handles. If we
see an equals sign after the field name, it must be a set expression
that is assigning to a field. But we don’t *always* allow an equals sign
after the field to be compiled. Consider:

<div class="codehilite">

    a + b.c = 3

</div>

This is syntactically invalid according to Lox’s grammar, which means
our Lox implementation is obligated to detect and report the error. If
`dot()` silently parsed the `= 3` part, we would incorrectly interpret
the code as if the user had written:

<div class="codehilite">

    a + (b.c = 3)

</div>

The problem is that the `=` side of a set expression has much lower
precedence than the `.` part. The parser may call `dot()` in a context
that is too high precedence to permit a setter to appear. To avoid
incorrectly allowing that, we parse and compile the equals part only
when `canAssign` is true. If an equals token appears when `canAssign` is
false, `dot()` leaves it alone and returns. In that case, the compiler
will eventually unwind up to `parsePrecedence()`, which stops at the
unexpected `=` still sitting as the next token and reports an error.

If we find an `=` in a context where it *is* allowed, then we compile
the expression that follows. After that, we emit a new
<span id="set">`OP_SET_PROPERTY`</span> instruction. That takes a single
operand for the index of the property name in the constant table. If we
didn’t compile a set expression, we assume it’s a getter and emit an
`OP_GET_PROPERTY` instruction, which also takes an operand for the
property name.

You can’t *set* a non-field property, so I suppose that instruction
could have been `OP_SET_FIELD`, but I thought it looked nicer to be
consistent with the get instruction.

Now is a good time to define these two new instructions.

<div class="codehilite">

``` insert-before
  OP_SET_UPVALUE,
```

<div class="source-file">

*chunk.h*  
in enum *OpCode*

</div>

``` insert
  OP_GET_PROPERTY,
  OP_SET_PROPERTY,
```

``` insert-after
  OP_EQUAL,
```

</div>

<div class="source-file-narrow">

*chunk.h*, in enum *OpCode*

</div>

And add support for disassembling them:

<div class="codehilite">

``` insert-before
      return byteInstruction("OP_SET_UPVALUE", chunk, offset);
```

<div class="source-file">

*debug.c*  
in *disassembleInstruction*()

</div>

``` insert
    case OP_GET_PROPERTY:
      return constantInstruction("OP_GET_PROPERTY", chunk, offset);
    case OP_SET_PROPERTY:
      return constantInstruction("OP_SET_PROPERTY", chunk, offset);
```

``` insert-after
    case OP_EQUAL:
```

</div>

<div class="source-file-narrow">

*debug.c*, in *disassembleInstruction*()

</div>

### <a href="#interpreting-getter-and-setter-expressions"
id="interpreting-getter-and-setter-expressions"><span
class="small">27 . 4 . 1</span>Interpreting getter and setter
expressions</a>

Sliding over to the runtime, we’ll start with get expressions since
those are a little simpler.

<div class="codehilite">

``` insert-before
      }
```

<div class="source-file">

*vm.c*  
in *run*()

</div>

``` insert
      case OP_GET_PROPERTY: {
        ObjInstance* instance = AS_INSTANCE(peek(0));
        ObjString* name = READ_STRING();

        Value value;
        if (tableGet(&instance->fields, name, &value)) {
          pop(); // Instance.
          push(value);
          break;
        }
      }
```

``` insert-after
      case OP_EQUAL: {
```

</div>

<div class="source-file-narrow">

*vm.c*, in *run*()

</div>

When the interpreter reaches this instruction, the expression to the
left of the dot has already been executed and the resulting instance is
on top of the stack. We read the field name from the constant pool and
look it up in the instance’s field table. If the hash table contains an
entry with that name, we pop the instance and push the entry’s value as
the result.

Of course, the field might not exist. In Lox, we’ve defined that to be a
runtime error. So we add a check for that and abort if it happens.

<div class="codehilite">

``` insert-before
          push(value);
          break;
        }
```

<div class="source-file">

*vm.c*  
in *run*()

</div>

``` insert

        runtimeError("Undefined property '%s'.", name->chars);
        return INTERPRET_RUNTIME_ERROR;
```

``` insert-after
      }
      case OP_EQUAL: {
```

</div>

<div class="source-file-narrow">

*vm.c*, in *run*()

</div>

<span id="field">There</span> is another failure mode to handle which
you’ve probably noticed. The above code assumes the expression to the
left of the dot did evaluate to an ObjInstance. But there’s nothing
preventing a user from writing this:

<div class="codehilite">

    var obj = "not an instance";
    print obj.field;

</div>

The user’s program is wrong, but the VM still has to handle it with some
grace. Right now, it will misinterpret the bits of the ObjString as an
ObjInstance and, I don’t know, catch on fire or something definitely not
graceful.

In Lox, only instances are allowed to have fields. You can’t stuff a
field onto a string or number. So we need to check that the value is an
instance before accessing any fields on it.

Lox *could* support adding fields to values of other types. It’s our
language and we can do what we want. But it’s likely a bad idea. It
significantly complicates the implementation in ways that hurt
performance<span class="em">—</span>for example, string interning gets a
lot harder.

Also, it raises gnarly semantic questions around the equality and
identity of values. If I attach a field to the number `3`, does the
result of `1 + 2` have that field as well? If so, how does the
implementation track that? If not, are those two resulting “threes”
still considered equal?

<div class="codehilite">

``` insert-before
      case OP_GET_PROPERTY: {
```

<div class="source-file">

*vm.c*  
in *run*()

</div>

``` insert
        if (!IS_INSTANCE(peek(0))) {
          runtimeError("Only instances have properties.");
          return INTERPRET_RUNTIME_ERROR;
        }
```

``` insert-after
        ObjInstance* instance = AS_INSTANCE(peek(0));
```

</div>

<div class="source-file-narrow">

*vm.c*, in *run*()

</div>

If the value on the stack isn’t an instance, we report a runtime error
and safely exit.

Of course, get expressions are not very useful when no instances have
any fields. For that we need setters.

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
      case OP_SET_PROPERTY: {
        ObjInstance* instance = AS_INSTANCE(peek(1));
        tableSet(&instance->fields, READ_STRING(), peek(0));
        Value value = pop();
        pop();
        push(value);
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

This is a little more complex than `OP_GET_PROPERTY`. When this
executes, the top of the stack has the instance whose field is being set
and above that, the value to be stored. Like before, we read the
instruction’s operand and find the field name string. Using that, we
store the value on top of the stack into the instance’s field table.

After that is a little <span id="stack">stack</span> juggling. We pop
the stored value off, then pop the instance, and finally push the value
back on. In other words, we remove the *second* element from the stack
while leaving the top alone. A setter is itself an expression whose
result is the assigned value, so we need to leave that value on the
stack. Here’s what I mean:

The stack operations go like this:

![Popping two values and then pushing the first value back on the
stack.](image/classes-and-instances/stack.png)

<div class="codehilite">

    class Toast {}
    var toast = Toast();
    print toast.jam = "grape"; // Prints "grape".

</div>

Unlike when reading a field, we don’t need to worry about the hash table
not containing the field. A setter implicitly creates the field if
needed. We do need to handle the user incorrectly trying to store a
field on a value that isn’t an instance.

<div class="codehilite">

``` insert-before
      case OP_SET_PROPERTY: {
```

<div class="source-file">

*vm.c*  
in *run*()

</div>

``` insert
        if (!IS_INSTANCE(peek(1))) {
          runtimeError("Only instances have fields.");
          return INTERPRET_RUNTIME_ERROR;
        }
```

``` insert-after
        ObjInstance* instance = AS_INSTANCE(peek(1));
```

</div>

<div class="source-file-narrow">

*vm.c*, in *run*()

</div>

Exactly like with get expressions, we check the value’s type and report
a runtime error if it’s invalid. And, with that, the stateful side of
Lox’s support for object-oriented programming is in place. Give it a
try:

<div class="codehilite">

    class Pair {}

    var pair = Pair();
    pair.first = 1;
    pair.second = 2;
    print pair.first + pair.second; // 3.

</div>

This doesn’t really feel very *object*-oriented. It’s more like a
strange, dynamically typed variant of C where objects are loose
struct-like bags of data. Sort of a dynamic procedural language. But
this is a big step in expressiveness. Our Lox implementation now lets
users freely aggregate data into bigger units. In the next chapter, we
will breathe life into those inert blobs.

<div class="challenges">

## <a href="#challenges" id="challenges">Challenges</a>

1.  Trying to access a non-existent field on an object immediately
    aborts the entire VM. The user has no way to recover from this
    runtime error, nor is there any way to see if a field exists
    *before* trying to access it. It’s up to the user to ensure on their
    own that only valid fields are read.

    How do other dynamically typed languages handle missing fields? What
    do you think Lox should do? Implement your solution.

2.  Fields are accessed at runtime by their *string* name. But that name
    must always appear directly in the source code as an *identifier
    token*. A user program cannot imperatively build a string value and
    then use that as the name of a field. Do you think they should be
    able to? Devise a language feature that enables that and implement
    it.

3.  Conversely, Lox offers no way to *remove* a field from an instance.
    You can set a field’s value to `nil`, but the entry in the hash
    table is still there. How do other languages handle this? Choose and
    implement a strategy for Lox.

4.  Because fields are accessed by name at runtime, working with
    instance state is slow. It’s technically a constant-time
    operation<span class="em">—</span>thanks, hash
    tables<span class="em">—</span>but the constant factors are
    relatively large. This is a major component of why dynamic languages
    are slower than statically typed ones.

    How do sophisticated implementations of dynamically typed languages
    cope with and optimize this?

</div>

<a href="methods-and-initializers.html" class="next">Next Chapter:
“Methods and Initializers” →</a> Handcrafted by Robert Nystrom — <a
href="https://github.com/munificent/craftinginterpreters/blob/master/LICENSE"
target="_blank">© 2015 – 2021</a>

</div>
