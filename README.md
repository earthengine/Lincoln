# Lincoln

What do you need to do to write an intepreter? This is an attempt to minimalise the things an interpreter writer need to concern.

## Piror Arts

There are a few computing model is generally considered "minimal" or simplest.

### Turing Machine

The Turing machine is the first model that being accepted to be the definition of "computable", and its defintion is very simple. 

Its operational definition makes it easy to estimate the complexity of a specific algorithm, because each operational state transfer is garanteed to have the same amount of work to finish.

However, to create a Turing machine that solve a problem in practice, will require a lot of work to represent the concepts in the problem, as Turing machine's concept (states, head movements, read/write and symbols) is too simple and does not mean anything.

It is also not easy to seperate a big machine into its parts to understand what and how it does. This makes it extremly hard for human being.

### Lambda expression

On the other hand, lambda expression is also considered simple as it only contains very limited operations, and it is sementical. It is easy for human being to understand, because it is built from small parts with easy combinators.

Unfortunately, it lacks proper operational defintion: the evaluation order is not defined (we have call-by-name and call-by-value variants), and the basic operation (reduction) is based on substitution, a single state transfer can have a huge effect - the result of a single state transfer can be 100 times longer than before, although it is still have linear complexity.

## What does "minimalism" mean?

I mean a very strict rule that if any feature can be implemented outside the intepreter, we don't include it as a feature. Precicelly, this means:

### Opaque Values

Every language have values, so does mine. However, the values in this system is opaque, which means I don't assume any operation is possible on the values, except receiving from and sending them to the external world. This means you cannot create, destroy or copy any values came from the outside.

However, the language do include a special type of value called "Closures", which are simple wrappers over external values. You can create them by wrapping it, or destroy it by unwrapping to external values. And that is all you can do. If you have such a value, you cannot simply drop it nor copy it. You also even cannot alias it.

Don't even think about more complicated stuff like threading etc, they can be done in the external world only!

### Runtime type system

I don't offer static type system. Instead, every type are assumed the same. The intepreter will tell you that you have wrong number of values, called the wrong entry point etc, but when you build the program, you are not pretected by a type system. So be careful when coding!

(This does not mean that we cannot make a static type system though; it is just not in the scope yet and it will soon be)

### Minimal Operation Set

At any point of evaluation time, a set of values and a code point is specified. I have already talked about values. A code point is one of the following 3 cases:

* An "Entry" code point refer to a specific instruction of the program (see the following for the types of instructions).

* An "External" code point refer to a function to be provided by the outside world (usually, by the intepreter itself).

* A "Termination" code point is a special one that notify the engine it should stop.

One or more code points can be combined in a group. A grouped code point can be used in branches.

As an IR (like assembly languages), I only support 3 "instruction" type (type of `Entry`s):

* Call. You can specify what to call, and where to return. (Unlike many assembly languages, you have to specify which piece of code is going to receive the result) The return position will be turned into a "Closure" value, accessable in the callee code. n

* Jump. You can use Jump to organise arguments, and then sent them to a new code point. A specified "permutation" is used to define how the arguments should be prepred for the jump.

* Return. You pick the first value from the context as the return position, also a variation index to specify the actual entry to be returned to.

### Turing completness

The above description of Lincoln have one missed point: you can easily see how sequential execution can be done and I explicitly said conditional execution is done through `Group` code point. But how about recursion/loops?

First of all, using `External` everyting can be done. As all code references are in `Call` and `Jump` or `Group`, you can also create loops by refering an earlier code point. However, if we disallow reference loops in those (which is easy to enforce), we will have a system that is Turing incomplete: it is provable that the number of execution steps is finite betwee two calls to `External` code points (it is exponential to the number of entries however). This means a Turing machine that is garantee to stop.

This is sometimes a desirable feature! So the ability to create loops will be an optional feature.



