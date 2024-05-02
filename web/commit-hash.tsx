import { Box, Link } from "@primer/react";
import * as React from "react";

let hash = "<hash>";
if (import.meta.env.SSR) {
  hash = await getGitCommitHash();
}

export default function CommitHash({ length = Number.POSITIVE_INFINITY }) {
  return hash.slice(0, length);
}

async function getGitCommitHash() {
  const fs = await import("node:fs/promises");
  const head = await fs.readFile(new URL("../.git/HEAD", import.meta.url), "utf8");
  if (!head.startsWith("ref: ")) throw Error("Invalid HEAD file");
  const ref = head.slice("ref: ".length);
  const commit = await fs.readFile(new URL("../.git/" + ref, import.meta.url), "utf8");
  return commit;
}
