import * as React from "react";

import { Flash } from "@primer/react";
import * as styles from "./styles.module.css";

export default function ErrorBar({ msg = "", ...others }) {
  return (
    <Flash variant="danger" {...others} className={styles.errorBar}>
      <span className={styles.errorText}>{msg}</span>
    </Flash>
  );
}
