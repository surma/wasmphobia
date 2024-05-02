import * as React from "react";

import { Box, Heading, Link, Text } from "@primer/react";
import Markdown from "react-markdown";

import readmeText from "../README.md?raw";

const markdownComponentMap = {
  a: Link,
  p: Text,
  ...Object.fromEntries(
    [1, 2, 3, 4, 5, 6].map(i => [`h${i}`, ({ children }) => {
      return <Heading as={`h${i}`}>{children}</Heading>;
    }]),
  ),
} as any;

export default function Readme() {
  return (
    <Box>
      <Markdown components={markdownComponentMap}>{readmeText}</Markdown>
    </Box>
  );
}
