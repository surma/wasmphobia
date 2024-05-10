import * as fs from "node:fs/promises";

const MARKER = "cli-flags:";

const STRUCT_MATCHER = /struct\s*Args\s*\{[^}]+\}/;
const FLAG_MATCHER = /#\[arg\((?<params>.+)\)\].*$\s*\/\/\/(?<comment>.+)$\s*(?<flag>[^:]+):(?<type>[^,]+),?$/gm;
const DEFAULT_MATCHER = /default_value_t\s*=\s*(?<def>[^,\)]+)/;

export default function optionsExtractorPlugin() {
  return {
    name: "CLI Flags",
    resolveId(id) {
      if (id !== MARKER) return;
      const file = new URL("./src/main.rs", import.meta.url).pathname;
      return MARKER + file;
    },
    async load(id) {
      if (!id.startsWith(MARKER)) return;
      const file = id.slice(MARKER.length);
      this.addWatchFile(file);
      const code = await fs.readFile(file, "utf8");
      const [struct] = STRUCT_MATCHER.exec(code);
      const flags = [];
      while (true) {
        const match = FLAG_MATCHER.exec(struct);
        if (!match) break;

        const { comment, flag, type, params } = match.groups;
        const flagDesc = {
          flag: `--${flag.trim().replaceAll("_", "-")}`,
          title: flagToTitle(flag.trim()),
          description: comment.trim(),
          type: type.trim(),
        };
        const def = DEFAULT_MATCHER.exec(params);
        if (def) {
          flagDesc.def = def.groups.def.trim();
        }
        flags.push(flagDesc);
      }
      return `export default ${JSON.stringify(flags)}`;
    },
  };
}

function flagToTitle(s) {
  return s.replace(/(_|^)([a-z])/g, (_, l, m) => l.replace("_", " ") + m.toUpperCase());
}
