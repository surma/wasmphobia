import { ConsoleStdout, File as WasiFile, OpenFile, WASI } from "@bjorn3/browser_wasi_shim";

export default async function renderFlameGraph(file, options = []) {
  const fileName = file.name;
  const data = await new Response(file).arrayBuffer();
  const input = new WasiFile(data);
  const output = new WasiFile();
  let errorLog;
  const wasi = new WASI(["__", `--title=${fileName}`, ...options], [], [
    new OpenFile(input),
    new OpenFile(output),
    ConsoleStdout.lineBuffered(msg => {
      errorLog += msg + "\n";
      console.warn(msg);
    }),
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
    throw Error("Could not create flamegraph: " + errorLog);
  }
  return new TextDecoder().decode(output.data);
}
