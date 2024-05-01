import { BaseStyles, ThemeProvider } from "@primer/react";

import renderFlameGraph from "./framegraph.js";

import { FileBinaryIcon } from "@primer/octicons-react";
import { Box, Button, PageLayout } from "@primer/react";
import { Blankslate, PageHeader } from "@primer/react/experimental";

function Placeholder({ label }) {
  return <Box>{label}</Box>;
}

function App() {
  return (
    <ThemeProvider>
      <BaseStyles>
        <PageLayout>
          <PageLayout.Content>
            <Blankslate spacious>
              <Blankslate.Visual>
                <FileBinaryIcon size={"medium"} />
              </Blankslate.Visual>
              <Blankslate.Heading as={"h1"}>Wasmphobia</Blankslate.Heading>
              <Blankslate.Description>
                Drop a WebAssembly (<code>.wasm</code>) on this page to get a breakdown of what is contained within. If
                the binary contains DWARF debugging symbols, they will be used (on a best-effort basis) to break down
                the file size by source code files.
              </Blankslate.Description>
              <Blankslate.PrimaryAction>
                Analyze a WebAssembly file
              </Blankslate.PrimaryAction>
              <Blankslate.SecondaryAction>
                ???
              </Blankslate.SecondaryAction>
            </Blankslate>
          </PageLayout.Content>
          <PageLayout.Footer>
            <Placeholder label="Footer" />
          </PageLayout.Footer>
        </PageLayout>
      </BaseStyles>
    </ThemeProvider>
  );
}

React.render(<App />, document.body);

document.all.file.onchange = async ev => {
  const file = ev.target.files[0];
  const buf = await new Response(file).arrayBuffer();
  const svg = await renderFlameGraph(buf);
  const svgFile = new File([svg], "flamegraph.svg", { type: "image/svg+xml" });
  const url = URL.createObjectURL(svgFile);
  location.href = url;
};
