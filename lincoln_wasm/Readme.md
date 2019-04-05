# What is this for?

This is a library supposed to be use as a WebAssembly module. An example was given in `html` folder.

# Prerequsits

* The [Rust](https://rustup.rs/) programming language.
* [wasm-pack](https://rustwasm.github.io/wasm-pack/).
* [http-server](https://crates.io/crates/http-server). Install by
```
cargo install http-server
```

For the last piece (http server), you can also use the following:

* [simple-server](https://crates.io/crates/simple-server)
* [simple-http-server](https://crates.io/crates/simple-http-server)

# Build

```sh
cd html
cd ..; wasm-pack build --target web; copy pkg\Lincoln_wasm.js html; copy pkg\Lincoln_wasm_bg.wasm html; cd html; http-server -p 8080
```

# Try

![image](https://user-images.githubusercontent.com/2227031/55533862-5dfc2b80-56fe-11e9-9b19-c6d67fa07d9e.png)

Open http://127.0.0.1:8080 ,

Copy and paste the [factorial](https://github.com/earthengine/Lincoln/blob/master/fact.json) program to the text area. Enter a number like `10` in the input box, hit "run". You will get the result and how many steps the intepretor need to run the program.

![image](https://user-images.githubusercontent.com/2227031/55554137-c7476300-572d-11e9-9814-6d8ec617e0bb.png)

If you open the Javascript console, you can see the above piece, which is a representation of the compiled program.

# Under the hook

Within `index.html`, some simple functions are defined to work as the "external" entry points, plus an async function `run` that loads the WebAssembly intepretor, and define an event handler for the "Run" button. It then uses the intepretor API to run the program given in the text area.

# Easy Integration

The following Javascript code is all you need to use this in your web site:

```html
 <script type="module">
    import { LincolnIntepretor, default as init } from './lincoln_wasm.js';
    async function run() {
        await init('./lincoln_wasm_bg.wasm');
        // work with the API
    }
    run();
</script>
```
`LincolnIntepretor.new()` creates an instance of the intepretor.

`LincolnIntepretor.prototype.set_program` allows to pass in a Javascript object as the running program.

```javascript
intepretor.set_program(JSON.parse(program))
```

`LincolnIntepretor.prototype.compile` compiles the program. You will need to pass a set of functions and values. Functions can be just it; but other values have to have a name so you will write `{name: "one", value: 1}` instead.

```javascript
intepretor.compile([copy_int,drop_int,try_minus,mul,{name: "one", value: 1}]);
```

`LincolnIntepretor.prototype.run` runs the program. You need to specify an entry point (by name), a variation (because a single entry point can have variations), and an array contains the initial values. The last value is a boolean to indicate whether you want to run it in step mode.

```javascript
intepretor.run("fact", 0, [input], true);
```

`LincolnIntepretor.prototype.step` requires the intepretor to be in step mode. It does not take any arguments, and returns true if it remains in step mode (execution didn't terminate), or false if that is the end of execution.

Once at the end of execution, `LincolnIntepretor.prototype.get_context` gives you the final result.

```javascript
intepretor.get_context();
```

