// Turns a Markdown cheatsheet into Kiroshi's seed data.
//
//   node tools/import-cheatsheet.mjs <cheatsheet.md> [out.json]
//
// Every two-column table row becomes one item. The app comes from the `##`
// heading, the tag from the `###` heading; a `###` heading that starts with a
// program name ("lnav 过滤") overrides the app. Backticks are kept: the UI
// renders them as key caps or command text.
import { readFileSync, writeFileSync } from 'node:fs';

const [src, out = 'src-tauri/seed/shortcuts.json'] = process.argv.slice(2);
if (!src) {
  console.error('usage: node tools/import-cheatsheet.mjs <cheatsheet.md> [out.json]');
  process.exit(1);
}

const splitRow = (line) =>
  line
    .trim()
    .replace(/^\|/, '')
    .replace(/\|$/, '')
    .split(/(?<!\\)\|/)
    .map((c) => c.trim().replaceAll('\\|', '|'));

function kindOf(headerCell, keys) {
  const hasKey = headerCell.includes('按键');
  const hasCommand = headerCell.includes('命令');
  if (hasCommand && !hasKey) return 'command';
  if (!hasCommand) return 'key';
  // Mixed tables: ":ex commands" and "program args" are commands.
  const first = /`([^`]+)`/.exec(keys)?.[1] ?? keys;
  return /^:/.test(first) || /^[a-z]{2,}\s/.test(first) ? 'command' : 'key';
}

const items = [];
let h2 = '';
let app = '';
let tag = '';
let header = null;

for (const line of readFileSync(src, 'utf8').split(/\r?\n/)) {
  if (line.startsWith('## ')) {
    h2 = line
      .slice(3)
      .replace(/^\d+\.\s*/, '')
      .replace(/（.*$/, '')
      .trim();
    app = h2;
    tag = '';
    header = null;
  } else if (line.startsWith('### ')) {
    const h3 = line.slice(4).trim();
    const program = /^([A-Za-z][\w-]*)(?:\s+(.*))?$/.exec(h3);
    app = program ? program[1] : h2;
    tag = program ? (program[2] ?? '') : h3;
    header = null;
  } else if (line.trimStart().startsWith('|')) {
    const cells = splitRow(line);
    if (!header) header = cells;
    else if (cells.every((c) => /^:?-+:?$/.test(c))) continue;
    else if (cells.length >= 2 && cells[0]) {
      items.push({
        id: `seed-${String(items.length + 1).padStart(3, '0')}`,
        app,
        kind: kindOf(header[0], cells[0]),
        keys: cells[0],
        // Drop bold markers, but not a literal `**` inside code.
        desc: cells[1].replace(/\*\*([^*`]+)\*\*/g, '$1'),
        tags: tag ? [tag] : [],
      });
    }
  } else {
    header = null;
  }
}

writeFileSync(out, JSON.stringify({ version: 1, items }, null, 2) + '\n');
const apps = [...new Set(items.map((i) => i.app))];
console.log(`${items.length} items, ${apps.length} apps (${apps.join(', ')}) -> ${out}`);
