# Wasmphobia

Wasmphobia analyzes a WebAssembly file and gives you a breakdown of what contributed to the module’s size. This is only really useful when the WebAssembly binary has DWARF debugging data embedded.

## Usage

You can use Wasmphobia interactively on the [website](https://wasmphobia.surma.technology) or install it locally as a CLI

```
cargo install --git https://github.com/surma/wasmphobia
```

## How to compile your Wasm

If you care about file size, make sure you compile your code with optimizations (like `-O3` and `-flto`) enabled. In most languages, doing a “release” build should enable these settings for you. However, at the same time, doing a release build often strips debug information from the binary. Here’s a short list of how to do release build _with_ debug symbols.

### Rust

You can add `--config "profile.release.debug=true"` to your cargo invocation. If your release profile strips symbols, you will also need to disable this with `--config "profile.release.strip=false"`. For example, to make a release build targeting WASI, you’d run:

```
cargo build --config "profile.release.debug=true" --config "profile.release.strip=false" --release --target wasm32-wasi
```

### C++ / Emscripten

```
$(CPP) -O3 -gfull ...
```

## Shoutouts and Credit

- [Gimli](https://docs.rs/gimli) for parsing DWARF
- [addr2line](https://docs.rs/addr2line) for converting addresses to source locations.
- [Inferno](https://docs.rs/inferno) to render flame graphs
- [Primer](https://primer.style/) for the website
- [Bundlephobia](https://bundlephobia.com/) for the name

---

License Apache-2.0
