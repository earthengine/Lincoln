# What is Lincoln?

Lincoln is developed to be a new programming environment, including a simple IR, an simple interpreter, and later some layers of hight level languages.

The IR is also designed for a form that is suitable as a target of many other programming languages, although those languages generally have quite different sementics.

In computing, Lincoln lies in between two other simple models: the Turing machine and the lambda calculus. The Turing machine have simple operational sementic, but lacks of denotational semantics, which means you need to formally describe what a "state" and a piece of symbols in the tap means, the definition of the machine cannot tell you anything about it. The lambda calculus on the other hand, has full of denotational semantics, but hard to define operational sementic - a single substitution can be arbitary complicated, and the order of reduction is unspeified.

Lincoln has simple operational sementic: the state transition is deterministic, and is always O(1). However it is also have native denotation semantics. In Lincoln, values are not simple symbols like in the Turing machine. They are black boxes that can carry meaning and internal states.

# Features

* Minimalism: minimal build-in operations (`Jmp`,`Ret`,`Call`), minimal build-in types (closure types).
* Simple interpreting: the fully functional interpretor is about 1000 line of Rust code, includes the full type definition, thanks to the minimalism design.

In additional, unlike other programming platforms, Lincoln does not look for a big "std" library. Instead, every Lincoln program is supposed to be isolated, abstracted building blocks that rely on the outside world to supply basic operations. It only garantee that, as long as the external provided did their part, a Lincoln program will do what it supposed to do.

# Concepts

## Context

A context is a set of values. The basic operation over a context includes: split a context into two, or merge two context into one, or take a value out from a context, or use a permutation to reorder the values. Both should be O(1) operation.

How to represent a context is up to the intepretor. In this demostrate intepretor we represent it as a vector, but it can be the set of registries of a CPU, or a piece of memory, or cookies from a website, or anything.

## Values

The values in a context can be categoried into two:

* Closures. This is the only "build-in" type of Lincoln. They can be created and executed (used) within Lincoln. It is suprisingly rich and is able to represent types that are familier to other languages, includeing 
 
  * Tuples or product types
  * Enums or sum types
  * Intersection types or objects with multiple consuming methods

* External types. Lincoln does not understand those types and so any attempt to drop or execute them will result in error. They can only be produced and consumed by external code.

## Permutation

When a continuation is going to be executed, we use permutations to reorder the values. We use the Fisher and Yate's algorithm (but instead of generate the number randomly, specify the number) to map a permutation into a set of numbers, then map these numbers into a single number.

Implementations of intepretor will need to support at least 6 variables, as this is what a 8-bit integer can represent for a permutation.

## Instruction set

A program contains a set of code entries, or instructions. There are 3 different kind of code entries:

* Jump. After performing a permutation on the current context, jump to another entry or an external function. It have to specify where it will jumpt to. 
* Call. Create a value in the current context with part or the current context, and a group of entries or external functions. Then jump to another entry or external function. It have to specify which entry to call, and after that where to return. 
* Ret. Take a value out from the context, then execute the value with the rest of the current context, on the specified variant. If the value is a closure, this means the specified varient of the entry group will be used, and the captured values will be merged with the current context.

Every code entries are refered by a name. 

## Code references

When talking about "another entry or an external function" in code entries, I am talking about code references. A code reference is pointing to one of the following:

* Entry. A code entry.
* Extern. A "extern" function defined in the program
* ExternFn. A "extern" function that can be provided by the external world, but not defined in the program.
* Termination. Indiciate the execution that the execution is finish.

# Build and run the program

Run the following in a console:
```sh
git clone https://github.com/earthengine/lincoln
cargo run
```
The program will show a prompt

```sh
Lincoln> 
```
Now you can run some commands like

```text
entry: call entry1 1 from
entry1: jmp count #!ba
set export entry
```
You can use save command to save your work to a json file:

```text
save myprogram.json
```
Or load it back

```text
load myprogram.json
```
You can then compile the prog by 

```text
compile bint
```
where "bint" is a set of external functions for converting from and to a number data type defined in pure Lincoln. 

There are also another set of external functions provided, which is "fact" that use the executer's native data type.

To run the program, use
```
run entry 0 10
```
and get something like:

```text
Result(1/1): 10
```

# Examples
Two example programs are given

## bint.json
This is a demostration of how to define copiable native data type. To run, use the following commands:

```text
load bint.json
compile bint
run test_copy 0 "10usize"
```
Right now only "usize" type are supported, it is usually a 64 bit unsigned integer according to your system.

By running this you are expected to get
```text
Result(1/2): 10
Result(2/2): 10
```

## fact.json
This is a demostration of how to define algorithm completely rely on the outside world to provide the functions we need.

To run, use the following commands:

```text
load fact.json
compile fact
run fact 0 "10usize"
```
You are expected to get
```text
Result(1/1): 3628800
```

# Future development


The minimalism design of Lincoln means that the current IR will be likely be freezed like it is right now. Some thoughts

* Copiable types - they are provided by external entries - a "copy" entry
 can always copy the value. Furthermore, a "copiable" closure is possible if all variables it captures are copiable: it is simply a closure that have one extra variant that copies its captured variables.
* Dropable types - same as copiable. they can be a external type, or it can be an extra variant that does nothing.

## A proposal of high level language

If we represent the `fact` example above in a high level language that is Haskell-like, it would looks like

```haskell
extern zero
extern one
extern copy_int
extern eq
extern drop_int
extern minus
extern mul

fact c n := -- a definition of `fact`, takes variable `c` and `n`
    f =     -- an assignment, which is always lazy
        call c n := fact c n -- a definition of a variant "call"
        drop c := c          -- another variant "drop"
    zero -> z                -- an invocation, result are in `z`
    one -> o                 -- `zero` and `one` are external
    copy_int n -> n1 n2      -- `copy_int` is external
    eq n1 z                  -- Without the arrow, the group of variants simple values only
        equals :=
            drop_int n2 ->       -- `drop_int` is external
            f.drop ->        -- call the `drop` variant of f
            c o              -- the final statement
        not_equal := 
            copy_int n2 -> n1 n2
            minus n1 o -> n    -- minus is external
            c = _ n :=         -- Assign a closure to `c`
                mul n n2 -> n  -- mul is external
                c n
            f.call c n         -- call the `call` variant of f (recursion)
```
The above example demostrates the basic idea:

* All variable have to be defined once, and use once
* An inner scope can shadow the same variable in the outer. In such a case, the outer variable are not captured so it is still valid and have to be used somewhere else.
* Assignments are lazy, and invocations are eager. Both can be used to define variables.
* The right hand side of an assignment is a closure. The first variant of a closure can be unnamed.

# Further plans

The language in the above is not statically typed, but we should be able to have a type system for it.

## Lifetimes? Borrow checker?

If talking about the future statically typed language, yes they will be considered. Otherwise, it is not hard to image a dynamically typed language like the above that checking it at runtime.

# Prior Arts

Needless to say again Lincoln is something in between Turing machine and Lambda calculus. 

There is also another similar idea like [B Geron's continuation calculus](https://www.google.com/url?sa=t&rct=j&q=&esrc=s&source=web&cd=12&cad=rja&uact=8&ved=2ahUKEwjgvP7u3KThAhVWWH0KHR5_AjcQFjALegQIBBAC&url=https%3A%2F%2Farxiv.org%2Fpdf%2F1309.1257&usg=AOvVaw32VPkxNu1DMiZu0v3h0MIZ). My system is different because it is linear: B Geron's system allows to copy and drop terms freely, but my system do not.

# Contact me!

If you are interested in this project, please feel free to contact me through 

* https://www.linkedin.com/in/zhiyu-ren-51a48620/

Or just raise an issue if you don't care it to be public.