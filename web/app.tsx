import * as React from "react";

import { BaseStyles, PageLayout, ThemeProvider } from "@primer/react";
import DropStyleProvider from "./drop-style-provider.jsx";
import DropZone from "./drop-zone.js";
import Footer from "./footer.js";
import Readme from "./readme.jsx";

export default function App() {
  return (
    <ThemeProvider colorMode="dark">
      <BaseStyles>
        <DropStyleProvider>
          <PageLayout containerWidth="medium" sx={{ backgroundColor: "canvas.default", height: "100vh" }}>
            <PageLayout.Content>
              <DropZone />
              <Readme />
            </PageLayout.Content>
            <PageLayout.Footer sx={{ position: "absolute", bottom: 0, padding: 2 }}>
              <Footer />
            </PageLayout.Footer>
          </PageLayout>
        </DropStyleProvider>
      </BaseStyles>
    </ThemeProvider>
  );
}
