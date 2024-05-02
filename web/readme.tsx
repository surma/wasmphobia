import * as React from "react";

import { Box, Heading, Link, Text } from "@primer/react";
import Markdown from "react-markdown";

import styled from "styled-components";
import readmeText from "../README.md?raw";

function createHeadingComponent(level) {
  return ({ children }) => <Heading as={`h${level}`}>{children}</Heading>;
}

const BetterPre = styled.pre`
  white-space: normal
`;

const markdownComponentMap = {
  a: Link,
  p: Text,
  pre: BetterPre,
  ...Object.fromEntries(
    [1, 2, 3, 4, 5, 6].map(i => [`h${i}`, createHeadingComponent(i)]),
  ),
} as any;

export default function Readme() {
  return (
    <Box>
      <Markdown components={markdownComponentMap}>{readmeText}</Markdown>
    </Box>
  );
}
