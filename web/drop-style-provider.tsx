import * as React from "react";

import { themeGet } from "@primer/react";
import styled from "styled-components";

const Div = styled.div`
  --drop-invalid-bg-color: ${themeGet("colors.danger.muted")};
  --drop-valid-bg-color: ${themeGet("colors.success.muted")};
`;

export default function DropStyleProvider({ children }) {
  return <Div>{children}</Div>;
}
