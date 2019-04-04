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

Open http://127.0.0.1:8080 
