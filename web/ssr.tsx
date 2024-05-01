import * as fs from "node:fs/promises";
import { renderToString } from "preact-render-to-string";
import * as React from "react";
import { ServerStyleSheet } from "styled-components";

import App from "./app.jsx";

const SSR_MARKER = `<!-- SSR -->`;
const indexPath = new URL("./index.html", import.meta.url).pathname;
const index = await fs.readFile(indexPath, "utf8");
if (!index.includes(SSR_MARKER)) throw Error("index.html does not contain an SSR marker. Rebuild?");

const sheet = new ServerStyleSheet();
const html = renderToString(sheet.collectStyles(<App />));
const styleTags = sheet.getStyleTags();
const ssrIndex = index.replace(SSR_MARKER, styleTags + html);
await fs.writeFile(indexPath, ssrIndex);
sheet.seal();
