import * as React from "react";

import { Box, Button, Checkbox, CheckboxGroup, FormControl, Heading } from "@primer/react";
import * as styles from "./styles.module.css";

const options = [
  {
    flag: "--show-frames",
    name: "Show frames",
    default: true,
    caption: "Shows function names. When functions get inlined, it shows a stack of them.",
  },
  {
    flag: "--demangle-rust-names",
    name: "Demangle Rust Names",
    default: true,
    caption: "Make Rust functions humanly readable.",
  },
];

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
        <form className={styles.optionsForm}>
          <CheckboxGroup sx={{ mt: 3 }}>
            <CheckboxGroup.Caption>Options</CheckboxGroup.Caption>
            {options.map(option => (
              <FormControl>
                <Checkbox name={option.flag} checked={option.default} />
                <FormControl.Label>{option.name}</FormControl.Label>
                <FormControl.Caption>{option.caption}</FormControl.Caption>
              </FormControl>
            ))}
          </CheckboxGroup>
        </form>
      </Box>
    </>
  );
}
