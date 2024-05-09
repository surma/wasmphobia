# Wasmphobia

Wasmphobia analyzes a WebAssembly file and gives you a breakdown of what contributed to the module’s size. This is only really useful when the WebAssembly binary has DWARF debugging data embedded.

## Usage

You can use Wasmphobia interactively on the [website](https://wasmphobia.surma.technology) or install it locally as a CLI

```
cargo install --git https://github.com/surma/wasmphobia
```

## Shoutouts and Credit

- [Gimli](https://docs.rs/gimli) for parsing DWARF
- [addr2line](https://doc.rs/addr2line) for converting addresses to source locations.
- [Inferno](https://docs.rs/inferno) to render flame graphs
- [Primer](https://primer.style/) for the website
- [Bundlephobia](https://bundlephobia.com/) for the name

---

License Apache-2.0
