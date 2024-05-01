import renderFlameGraph from "./framegraph.js";

if (!import.meta.env.SSR) {
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

drop.ondragleave = ev => {
  drop.style.backgroundColor = "initial";
};

drop.ondragover = ev => {
  ev.preventDefault();
  const container = ev.target.closest("#drop");
  // console.log({ev});

  if (!isValidWasmDrop(ev.dataTransfer)) {
    drop.style.backgroundColor = "red";
    return;
  }
  drop.style.backgroundColor = "green";
};
drop.ondrop = ev => {
  ev.preventDefault();
  drop.style.backgroundColor = "initial";
  if (!isValidWasmDrop(ev.dataTransfer)) return;
  const file = ev.dataTransfer.files[0];
  process(file);
};
