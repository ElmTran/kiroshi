import { describe, expect, it } from 'vitest';
import { copyText, search, segments, type Item } from './search';

const item = (id: string, app: string, keys: string, desc: string, tags: string[] = []): Item => ({
  id,
  app,
  kind: 'key',
  keys,
  desc,
  tags,
});

const items = [
  item('1', 'yazi', '`d`', '删除到回收站', ['文件操作']),
  item('2', 'yazi', '`D`', '永久删除（慎用）', ['文件操作']),
  item('3', 'Vim', '`gd`', '跳到当前文件里的局部定义', ['跳转']),
  item('4', 'lnav', '`:filter-in trace=cdd2077e`', '只看包含它的行', ['过滤']),
  item('5', '终端', '`vim ` + `Ctrl-T`', '模糊选文件'),
];

const ids = (query: string) => search(items, query).map((i) => i.id);

describe('segments', () => {
  it('splits literal parts from the text between them', () => {
    expect(segments('`gg` / `G`')).toEqual([
      { text: 'gg', literal: true },
      { text: '/', literal: false },
      { text: 'G', literal: true },
    ]);
  });

  it('treats text without backticks as one literal', () => {
    expect(segments('Ctrl+Shift+P')).toEqual([{ text: 'Ctrl+Shift+P', literal: true }]);
  });

  it('keeps trailing spaces inside a literal', () => {
    expect(segments('`vim ` + `Ctrl-T`')[0]).toEqual({ text: 'vim ', literal: true });
  });
});

describe('search', () => {
  it('returns everything for an empty query', () => {
    expect(ids('  ')).toEqual(['1', '2', '3', '4', '5']);
  });

  it('ranks the exact-case key first', () => {
    expect(ids('D').slice(0, 2)).toEqual(['2', '1']);
    expect(ids('d').slice(0, 2)).toEqual(['1', '2']);
  });

  it('matches descriptions, apps and tags', () => {
    expect(ids('删除')).toEqual(['1', '2']);
    expect(ids('LNAV')).toEqual(['4']);
    expect(ids('跳转')).toEqual(['3']);
  });

  it('requires every word to match', () => {
    expect(ids('yazi 永久')).toEqual(['2']);
    expect(ids('yazi 跳转')).toEqual([]);
  });
});

describe('copyText', () => {
  it('copies the first literal part', () => {
    expect(copyText(items[3]!)).toBe(':filter-in trace=cdd2077e');
    expect(copyText(items[4]!)).toBe('vim ');
  });
});
