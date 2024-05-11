import * as styles from "./styles.module.css";
import { nextEvent } from "./utils.js";

const WASMPHOBIA_WASM = "https://s3.eu-west-1.amazonaws.com/f.surma.link/wasmphobia.wasm";

let worker;
if (!import.meta.env.ssr) {
  worker = new Worker(new URL("./worker.js", import.meta.url), { type: "module" });
  worker.addEventListener("error", ev => console.error(ev));
}

if (import.meta.env.DEV) {
  await import("./render.jsx");
}

const dropSignal = document.querySelector(`.${styles.dropSignal}`);
const dropZone = document.body;
const fileSelect = document.querySelector(`.${styles.fileSelect}`);
const optionsForm = document.querySelector(`.${styles.optionsForm}`);
const spinner = document.querySelector(`.${styles.spinner}`);
const exampleButton = document.querySelector(`.${styles.exampleButton}`);

optionsForm.addEventListener("submit", ev => ev.preventDefault());

let idCounter = 0;
async function process(file) {
  try {
    showSpinner();
    const id = idCounter++;
    const options = getSelectedOptions();
    worker.postMessage({ id, file, options });
    const { data: result } = await nextEvent(worker, "message", ev => ev.data.id === id);
    if (result.error) {
      throw Error(result.error);
    }
    const url = URL.createObjectURL(result.svg);
    location.href = url;
  } catch (e) {
    showError(e.message);
  } finally {
    hideSpinner();
  }
}

function signalDropValid() {
  dropSignal.classList.add(styles.dropValid);
}
function signalDropInvalid() {
  dropSignal.classList.add(styles.dropInvalid);
}
function resetDropSignal() {
  dropSignal.classList.remove(styles.dropValid, styles.dropInvalid);
}

fileSelect.onclick = () => {
  const f = document.createElement("input");
  f.type = "file";
  f.onchange = () => process(f.files[0]);
  f.click();
};

exampleButton.onclick = async () => {
  showSpinner();
  try {
    const blob = await fetch(WASMPHOBIA_WASM).then(r => r.blob());
    await process(new File([blob], "wasmphobia.wasm", { type: "application/wasm" }));
  } catch (e) {
    showError(e.message);
  } finally {
    hideSpinner();
  }
};

function isInvalidDrop(dt) {
  if (dt.items.length != 1) return "Only single files are supported";
  const item = dt.items[0];
  if (item.kind != "file") return "Only files can be opened";
  if (item.type == "application/wasm") return null;
  // Without a known file type, we’ll assume it’s a valid vile
  if (!item.type) return null;
  return "Unsupported file format";
}

dropZone.ondragleave = () => resetDropSignal();

dropZone.ondragover = ev => {
  ev.preventDefault();
  if (isInvalidDrop(ev.dataTransfer)) {
    signalDropInvalid();
    return;
  }
  signalDropValid();
};
dropZone.ondrop = ev => {
  ev.preventDefault();
  resetDropSignal();
  const error = isInvalidDrop(ev.dataTransfer);
  if (error) {
    showError(error);
    return;
  }
  const file = ev.dataTransfer.files[0];
  process(file);
};

const errorBar = document.querySelector(`.${styles.errorBar}`);
const errorText = document.querySelector(`.${styles.errorText}`);

function showError(text) {
  errorText.textContent = text;
  errorBar.hidden = false;
}

function getSelectedOptions() {
  return Array.from(optionsForm.elements).flatMap(el => {
    switch (el.type) {
      case "checkbox":
        return el.checked ? [el.name] : [];
      case "number":
        return `${el.name}=${el.value}`;
      default:
        throw Error("Unknown option field");
    }
  });
}

function showSpinner() {
  spinner.hidden = false;
}

function hideSpinner() {
  spinner.hidden = true;
}
