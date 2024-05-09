import renderFlameGraph from "./flamegraph.js";

addEventListener("message", async ev => {
  const { id, file } = ev.data;
  try {
    const svgContent = await renderFlameGraph(file);
    const svg = new File([svgContent], `${file.name}.svg`, { type: "image/svg+xml" });
    postMessage({ id, svg });
  } catch (e) {
    postMessage({ id, error: e.message });
  }
});
