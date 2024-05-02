import * as React from "react";

import { BaseStyles, PageLayout, ThemeProvider } from "@primer/react";
import DropStyleProvider from "./drop-style-provider.jsx";
import DropZone from "./drop-zone.js";
import ErrorBar from "./error-bar.js";
import Footer from "./footer.js";
import Readme from "./readme.jsx";

export default function App() {
  return (
    <ThemeProvider colorMode="dark">
      <BaseStyles>
        <DropStyleProvider>
          <PageLayout containerWidth="medium" sx={{ backgroundColor: "canvas.default", minHeight: "100vh" }}>
            <PageLayout.Content>
              <DropZone />
              <ErrorBar sx={{ m: 4 }} hidden />
              <Readme />
            </PageLayout.Content>
            <PageLayout.Footer sx={{ padding: 2 }}>
              <Footer />
            </PageLayout.Footer>
          </PageLayout>
        </DropStyleProvider>
      </BaseStyles>
    </ThemeProvider>
  );
}
