export type Kind = 'key' | 'command';

export interface Item {
  id: string;
  app: string;
  kind: Kind;
  /** The key combination or command. Backticks mark the literal parts. */
  keys: string;
  desc: string;
  tags: string[];
}

export interface Segment {
  text: string;
  /** True for the parts written in backticks: what you actually press or type. */
  literal: boolean;
}

/** Splits "`gg` / `G`" into literal parts and the text between them. */
export function segments(keys: string): Segment[] {
  if (!keys.includes('`')) return [{ text: keys, literal: true }];
  return keys
    .split(/(`[^`]+`)/)
    .filter((part) => part.trim() !== '')
    .map((part) =>
      part.startsWith('`') && part.endsWith('`') && part.length > 2
        ? { text: part.slice(1, -1), literal: true }
        : { text: part.trim(), literal: false },
    );
}

/** What Enter copies: the first literal part. */
export function copyText(item: Item): string {
  return segments(item.keys).find((s) => s.literal)?.text ?? item.keys;
}

function scoreToken(item: Item, token: string): number {
  const lower = token.toLowerCase();
  const literals = segments(item.keys)
    .filter((s) => s.literal)
    .map((s) => s.text);
  const plain = item.keys.replaceAll('`', '');
  const app = item.app.toLowerCase();

  // Keys are case sensitive (`d` is not `D`), so an exact match ranks first.
  if (literals.includes(token)) return 100;
  if (literals.some((l) => l.toLowerCase() === lower)) return 90;
  if (plain.includes(token)) return 60;
  if (plain.toLowerCase().includes(lower)) return 50;
  if (app.startsWith(lower)) return 45;
  if (app.includes(lower)) return 40;
  if (item.desc.toLowerCase().includes(lower)) return 30;
  if (item.tags.some((t) => t.toLowerCase().includes(lower))) return 20;
  return 0;
}

/**
 * Items matching every space-separated word of the query, best first. Ties
 * keep the order of the file. An empty query returns everything.
 */
export function search(items: Item[], query: string): Item[] {
  const tokens = query.split(/\s+/).filter(Boolean);
  if (tokens.length === 0) return items;
  const scored: { item: Item; score: number }[] = [];
  for (const item of items) {
    let total = 0;
    for (const token of tokens) {
      const score = scoreToken(item, token);
      if (score === 0) {
        total = 0;
        break;
      }
      total += score;
    }
    if (total > 0) scored.push({ item, score: total });
  }
  return scored.sort((a, b) => b.score - a.score).map((s) => s.item);
}
