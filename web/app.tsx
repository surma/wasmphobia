import * as React from "react";

import { FileBinaryIcon } from "@primer/octicons-react";
import { BaseStyles, Box, Button, Link, PageLayout, ThemeProvider } from "@primer/react";
import { Blankslate, PageHeader } from "@primer/react/experimental";
import Readme from "./readme.jsx";

export default function App() {
  return (
    <ThemeProvider>
      <BaseStyles>
        <PageLayout>
          <PageLayout.Content>
            <div id="drop">
              <Blankslate spacious>
                <Blankslate.Visual>
                  <FileBinaryIcon size={"medium"} />
                </Blankslate.Visual>
                <Blankslate.Heading as={"h1"}>Wasmphobia</Blankslate.Heading>
                <Blankslate.Description>
                  Drop a WebAssembly (<code>.wasm</code>) on this page to get a breakdown of what is contained within.
                  If the binary contains DWARF debugging symbols, they will be used (on a best-effort basis) to break
                  down the file size by source code files.
                </Blankslate.Description>
                <div id="fileselect">
                  <Blankslate.PrimaryAction>
                    Analyze a WebAssembly file
                  </Blankslate.PrimaryAction>
                </div>
                {false && (
                  <Blankslate.SecondaryAction>
                    ???
                  </Blankslate.SecondaryAction>
                )}
              </Blankslate>
              <Readme />
            </div>
          </PageLayout.Content>
          <PageLayout.Footer>
            Made with 🤦‍♂️ by <Link href="https://x.com/dassurma" target="_blank">Surma</Link>. Source code on{" "}
            <Link href="https://github.com/surma/wasmphobia" target="_blank">GitHub</Link>.
          </PageLayout.Footer>
        </PageLayout>
      </BaseStyles>
    </ThemeProvider>
  );
}
