import { ConsoleStdout, File as WasiFile, OpenFile, WASI } from "@bjorn3/browser_wasi_shim";

async function renderFlameGraph(data) {
  const input = new WasiFile(data);
  const output = new WasiFile();
  const error = new WasiFile();
  const wasi = new WASI([], [], [
    new OpenFile(input),
    new OpenFile(output),
    new OpenFile(error),
  ]);
  let wasm_url = new URL("../target/wasm32-wasi/release/wasmphobia.opt.wasm", import.meta.url);
  if (import.meta.env.MODE !== "production") {
    wasm_url = new URL("../target/wasm32-wasi/release/wasmphobia.wasm", import.meta.url);
  }
  const { instance } = await WebAssembly.instantiateStreaming(fetch(wasm_url), {
    "wasi_snapshot_preview1": wasi.wasiImport,
  });

  const ret = wasi.start({ exports: instance.exports });
  if (ret != 0) {
    const errorMessage = new TextDecoder().decode(error.data);
    throw Error("Could not render SVG:" + errorMessage);
  }
  return new TextDecoder().decode(output.data);
}

document.all.file.onchange = async ev => {
  const file = ev.target.files[0];
  const buf = await new Response(file).arrayBuffer();
  const svg = await renderFlameGraph(buf);
  const svgFile = new File([svg], "flamegraph.svg", { type: "image/svg+xml" });
  const url = URL.createObjectURL(svgFile);
  location.href = url;
};
