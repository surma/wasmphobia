import * as fs from "node:fs/promises";

const MARKER = "cli-flags:";

const STRUCT_MATCHER = /struct\s*Args\s*\{[^}]+\}/;
const FLAG_MATCHER =
  /#\[arg\(.+default_value_t\s*=\s*(?<def>.+)(,|\)).+$\s*\/\/\/(?<comment>.+)$\s*(?<flag>[^:]+):(?<type>[^,]+),?$/gm;

export default function optionsExtractorPlugin() {
  return {
    name: "CLI Flags",
    resolveId(id) {
      if (id !== MARKER) return;
      const file = new URL("./src/main.rs", import.meta.url).pathname;
      this.addWatchFile(file);
      return MARKER + file;
    },
    async load(id) {
      if (!id.startsWith(MARKER)) return;
      const file = id.slice(MARKER.length);
      const code = await fs.readFile(file, "utf8");
      const [struct] = STRUCT_MATCHER.exec(code);
      const flags = [];
      while (true) {
        const match = FLAG_MATCHER.exec(struct);
        if (!match) break;

        const { comment, flag, type, def } = match.groups;
        flags.push({
          flag: `--${flag.trim().replaceAll("_", "-")}`,
          title: flagToTitle(flag.trim()),
          description: comment.trim(),
          type: type.trim(),
          def: def.trim(),
        });
      }
      return `export default ${JSON.stringify(flags)}`;
    },
  };
}

function flagToTitle(s) {
  return s.replace(/(_|^)([a-z])/g, (_, l, m) => l.replace("_", " ") + m.toUpperCase());
}
