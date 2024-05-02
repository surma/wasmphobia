import * as React from "react";

import { Box, Button, Heading } from "@primer/react";
import * as styles from "./styles.module.css";

export default function DropZone() {
  return (
    <>
      <div className={[styles.dropZone, styles.dropSignal].join(" ")} />
      <Box
        sx={{
          borderStyle: "dashed",
          borderWidth: ".3rem",
          borderColor: "border.subtle",
          borderRadius: ".9rem",
          p: 3,
          m: 4,
          display: "flex",
          flexDirection: "column",
          textAlign: "center",
        }}
      >
        <Heading as="h2" sx={{ p: 3, display: "flex", justifyContent: "center", alignItems: "center" }}>
          <span>
            Drop a <code>.wasm</code> file
          </span>
        </Heading>
        <Button className={styles.fileSelect}>
          ... or select one
        </Button>
      </Box>
    </>
  );
}
