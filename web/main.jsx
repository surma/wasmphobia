import "./styles.css";
import renderFlameGraph from "./framegraph.js";
import { invalidDrop, validDrop } from "./styles.module.css";

if (import.meta.env.DEV) {
  await import("./render.jsx");
}

async function process(file) {
  const buf = await new Response(file).arrayBuffer();
  const svg = await renderFlameGraph(buf);
  const svgFile = new File([svg], "flamegraph.svg", { type: "image/svg+xml" });
  const url = URL.createObjectURL(svgFile);
  location.href = url;
}

const { fileselect, drop } = document.all;

function signalDropValid() {
  drop.classList.add(validDrop);
}
function signalDropInvalid() {
  drop.classList.add(invalidDrop);
}
function resetDropSignal() {
  drop.classList.remove(validDrop, invalidDrop);
}

fileselect.onclick = ev => {
  const f = document.createElement("input");
  f.type = "file";
  f.onchange = () => process(f.files[0]);
  f.click();
};

function isValidWasmDrop(dt) {
  if (dt.items.length != 1) return null;
  const item = dt.items[0];
  if (item.kind != "file") return null;
  if (item.type != "application/wasm") return null;
  return item;
}

drop.ondragleave = () => resetDropSignal();

drop.ondragover = ev => {
  ev.preventDefault();
  const container = ev.target.closest("#drop");

  if (!isValidWasmDrop(ev.dataTransfer)) {
    signalDropInvalid();
    return;
  }
  signalDropValid();
};
drop.ondrop = ev => {
  ev.preventDefault();
  resetDropSignal();
  if (!isValidWasmDrop(ev.dataTransfer)) return;
  const file = ev.dataTransfer.files[0];
  process(file);
};
