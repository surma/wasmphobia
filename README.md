# Wasmphobia

Wasmphobia gives you a breakdown of which source code file contributed to your module’s binary size. It does this using the DWARF debugging data.

## Usage

You can use Wasmphobia interactively on the [website](https://wasmphobia.surma.technology) or install it locally as a CLI

```
cargo install --git https://github.com/surma/wasmphobia
```

## Accuracy

This tool is not quite complete in consuming all aspects of the DWARF data, so the breakdown can be incomplete and have larger sections that are unattributed, it won’t wrongly attribute a section.

## Shoutouts and Credit

- [Gimli](https://docs.rs/gimli/latest/gimli/) for parsing DWARF
- [walrus](https://docs.rs/walrus/latest/walrus/) for parsing Wasm
- [Inferno](https://docs.rs/inferno/latest/inferno/) to render flame graphs
- [Primer](https://primer.style/) for the website
- [Bundlephobia](https://bundlephobia.com/) for the name

---

License Apache-2.0
