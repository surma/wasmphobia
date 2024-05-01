import * as React from "react";

import { Box, Heading, Link, Text } from "@primer/react";
import Markdown from "react-markdown";
import { readme } from "./styles.module.css";

import readmeText from "../README.md?raw";

const markdownComponentMap = {
  a: Link,
  p: Text,
  ...Object.fromEntries(
    [1, 2, 3, 4, 5, 6].map(i => [`h${i}`, ({ children }) => <Heading as={`h${i}`}>{children}</Heading>]),
  ),
};

export default function Readme() {
  return (
    <Box>
      <div className={readme}>
        <Markdown components={markdownComponentMap}>{readmeText}</Markdown>
      </div>
    </Box>
  );
}
