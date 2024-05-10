import * as React from "react";

import {
  Box,
  Button,
  Checkbox,
  CheckboxGroup,
  Details,
  FormControl,
  Heading,
  Link,
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
        <OptionsForm />
      </Box>
    </>
  );
}

function OptionsForm() {
  const activeOptions = options.filter(option => !HIDDEN_FLAGS.has(option.flag));
  const checkboxes = activeOptions.filter(option => option.type === "bool");
  const other = activeOptions.filter(option => option.type !== "bool");
  return (
    <form className={styles.optionsForm}>
      <Details sx={{ mt: 3, textAlign: "left" }}>
        <Link as="summary">Show options</Link>
        <Box p={3} display="grid" gridTemplateColumns="repeat(2, 1fr)" gridColumnGap={3}>
          <Box display="grid" gridTemplateColumns={"1fr"} gridRowGap={3}>
            {other.map(option => <CliOption option={option} sx={{ gridColumn: "2/-1" }} />)}
          </Box>
          <Box display="grid" gridTemplateColumns={"auto"} gridRowGap={3}>
            {checkboxes.map(option => <CliOption option={option} sx={{ gridColumn: "1/2" }} />)}
          </Box>
        </Box>
      </Details>
    </form>
  );
}

function CliOption({ option, ...rest }) {
  switch (option.type) {
    case "bool":
      return <BoolCliOption option={option} {...rest} />;
    case "usize":
      return <NumberCliOption option={option} {...rest} />;
    default:
      throw Error(`Unknown CLI flag type: ${option.type}`);
  }
}

function BoolCliOption({ option, ...rest }) {
  return (
    <FormControl {...rest}>
      <Checkbox name={option.flag} defaultChecked={JSON.parse(option.def ?? "false")} />
      <FormControl.Label>{option.title}</FormControl.Label>
      <FormControl.Caption>{option.description}</FormControl.Caption>
    </FormControl>
  );
}

function NumberCliOption({ option, ...rest }) {
  return (
    <FormControl {...rest}>
      <TextInput type="number" min={0} name={option.flag} defaultValue={Number(option.def)} />
      <FormControl.Label>{option.title}</FormControl.Label>
      <FormControl.Caption>{option.description}</FormControl.Caption>
    </FormControl>
  );
}
