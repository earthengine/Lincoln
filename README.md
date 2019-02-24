# What is Lincoln?

Lincoln is a simple conceptual programming environment. Rather than focusing on making programming easier, it is aim to be a easy computing model at the moment. So do not expect programming in Lincoln would be easy. However, it have very easy concepts and brain models, so it is not hard to learn.

The name "Lincoln" is to regard the great American president, but it also means short pronunciation "Linear Continuation", which means:

* Every statement have a continuation, either through the context value, or through the statement setting.
* All values are linear, including the continuations, which means they can only be used once and have to be used once.

# What is continuation?

In other languages you have functions, which you get a value back from calling a function. A function works on a stack, and a stack is a special kind of data: the size is dynamic. In Lincoln, all values are like the stack, even the "return

# Concepts

## Permutation

When a continuation is going to be executed, we use permutations to reorder the values. We use the Fisher and Yate's algorithm (but instead of generate the number randomly, specify the number) to map a permutation into a set of numbers, then map these numbers into a single number, stored in a 64 bit unsigned number.

As a result of a 64 bit number cannot present a number greater than 20!, we limit the number of values to 20.

## Context

A context is a set of values. In theory, we allow any number of values in a contexts. For the reason that we do not allow permutations for more than 20 elements, we limit the number of elements of a context to 20.

## Values

A contaxt can contain the following values:

* Closure. A closure contains a reference to a group of code entry, and a set of "captured" values, stored in a context.
* Wrapped. A wrapped value presents a value that is only undersood by the execution engine. In this demostration this can be anything that is `Any`.
* FinalReceiver. This represents a special value that was provided by the execution engine only. When it was evaluated the program execution ends normally.

## Code entries

A program contains a set of code entries, or instructions. There are 4 different kind of code entries:

* Jump. After performing a permutation on the current context, jump to another entry or an external function.
* Call. Create a value in the current context with part or the current context, and a group of entries or external functions. Then jump to another entry or external function.
* Ret. Evaluate the first closure value in the current context. A specified "variant" is use to pick one entry from the group.

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
run entry 0 "10"
```
You are expected to get
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
run entry 0 "10"
```
You are expected to get
```text
Result(1/1): 3628800
```

# Future development
We have a lot to do

* Seperate the core engine into a lib module, making it independent to the external function sets.
* Introduce a type system and make the human input language easier to use.
* Instead of interpreting, compile it to another language or binary. One potential target is Webassembly.
* Futher examples to come!

# Contact me!

If you are interested in this project, please feel free to contact me through 

* https://www.linkedin.com/in/zhiyu-ren-51a48620/

Or just raise an issue if you don't care it to be public.