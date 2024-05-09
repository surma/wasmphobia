import { Box, Link } from "@primer/react";
import * as React from "react";
import CommitHash from "./commit-hash";

export default function Footer() {
  return (
    <Box sx={{ display: "flex", justifyContent: "center", alignItems: "center" }}>
      <Box sx={{ textAlign: "center" }}>
        Made with 🤦‍♂️ by <Link href="https://x.com/dassurma" target="_blank">Surma</Link>. Source code on{" "}
        <Link href="https://github.com/surma/wasmphobia" target="_blank">GitHub</Link>. Commit <CommitHash length={6} />
      </Box>
    </Box>
  );
}
