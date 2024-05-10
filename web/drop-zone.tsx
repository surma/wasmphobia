import * as React from "react";

import {
  Box,
  Button,
  Checkbox,
  CheckboxGroup,
  FormControl,
  Heading,
  Spinner,
  TextInput,
  useTheme,
} from "@primer/react";
import * as styles from "./styles.module.css";

import options from "cli-flags:";

const HIDDEN_FLAGS = new Set(["--title"]);

export default function DropZone() {
  const theme = useTheme();
  return (
    <>
      <div className={[styles.dropZone, styles.dropSignal].join(" ")} />
      <Box
        sx={{
          position: "relative",
          borderStyle: "dashed",
          borderWidth: ".3rem",
          borderColor: "border.subtle",
          borderRadius: ".9rem",
          p: 3,
          m: 4,
          display: "flex",
          flexDirection: "column",
          textAlign: "center",
          gap: 3,
        }}
      >
        <Box sx={{ "--bgcolor": theme.theme.colors.canvas.overlay }} className={styles.spinner} hidden>
          <Spinner size="large" />
        </Box>
        <Heading as="h2" sx={{ p: 3, display: "flex", justifyContent: "center", alignItems: "center" }}>
          <span>
            Drop a <code>.wasm</code> file
          </span>
        </Heading>
        <Button className={styles.fileSelect} variant="primary">
          Select a Wasm file from your computer
        </Button>
        <Button className={styles.exampleButton}>
          Load Wasmphobia’s Wasm file (~10MB)
        </Button>
        <form className={styles.optionsForm}>
          <CheckboxGroup sx={{ mt: 3 }}>
            <CheckboxGroup.Caption>Options</CheckboxGroup.Caption>
            {options.filter(option => !HIDDEN_FLAGS.has(option.flag)).map(option => <CliOption option={option} />)}
          </CheckboxGroup>
        </form>
      </Box>
    </>
  );
}

function CliOption({ option }) {
  switch (option.type) {
    case "bool":
      return <BoolCliOption option={option} />;
    case "usize":
      return <NumberCliOption option={option} />;
    default:
      throw Error(`Unknown CLI flag type: ${option.type}`);
  }
}

function BoolCliOption({ option }) {
  return (
    <FormControl>
      <Checkbox name={option.flag} defaultChecked={JSON.parse(option.def)} />
      <FormControl.Label>{option.title}</FormControl.Label>
      <FormControl.Caption>{option.description}</FormControl.Caption>
    </FormControl>
  );
}

function NumberCliOption({ option }) {
  return (
    <FormControl>
      <TextInput type="number" min={0} name={option.flag} defaultValue={Number(option.def)} />
      <FormControl.Label>{option.title}</FormControl.Label>
      <FormControl.Caption>{option.description}</FormControl.Caption>
    </FormControl>
  );
}
