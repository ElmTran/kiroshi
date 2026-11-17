import { useCallback, useEffect, useMemo, useRef, useState } from 'react';
import { api, type AppInfo } from './api';
import { Editor } from './Editor';
import { copyText, search, segments, type Item } from './search';

const ALL = '';

export function App() {
  const [items, setItems] = useState<Item[]>([]);
  const [loadError, setLoadError] = useState('');
  const [info, setInfo] = useState<AppInfo | null>(null);
  const [query, setQuery] = useState('');
  const [app, setApp] = useState(ALL);
  const [selected, setSelected] = useState(0);
  const [editing, setEditing] = useState<Item | null>(null);
  const [notice, setNotice] = useState('');
  const input = useRef<HTMLInputElement>(null);
  const list = useRef<HTMLUListElement>(null);

  const reload = useCallback(async () => {
    try {
      setItems(await api.list());
      setLoadError('');
    } catch (e) {
      setLoadError(String(e));
    }
  }, []);

  useEffect(() => {
    void reload();
    void api.info().then(setInfo);
    // Each time the window is summoned: pick up edits made to the file by
    // hand, and get ready to type over the last query.
    return api.onShown(() => {
      void reload();
      setEditing(null);
      setNotice('');
      input.current?.focus();
      input.current?.select();
    });
  }, [reload]);

  const apps = useMemo(() => [...new Set(items.map((i) => i.app))], [items]);
  const matched = useMemo(() => search(items, query), [items, query]);
  const counts = useMemo(() => {
    const map = new Map<string, number>();
    for (const item of matched) map.set(item.app, (map.get(item.app) ?? 0) + 1);
    return map;
  }, [matched]);
  const visible = useMemo(
    () => (app === ALL ? matched : matched.filter((i) => i.app === app)),
    [matched, app],
  );
  const current = visible[Math.min(selected, visible.length - 1)];

  useEffect(() => setSelected(0), [query, app]);
  useEffect(() => {
    list.current?.querySelector('[aria-selected="true"]')?.scrollIntoView({ block: 'nearest' });
  }, [selected, visible]);

  const copy = async (item: Item) => {
    try {
      await navigator.clipboard.writeText(copyText(item));
      setNotice(`已复制 ${copyText(item)}`);
      await api.hide();
    } catch (e) {
      setNotice(`复制失败：${String(e)}`);
    }
  };

  // Tab skips apps with nothing matching the query.
  const cycleApp = (step: number) => {
    const order = [ALL, ...apps.filter((a) => a === app || counts.has(a))];
    const at = order.indexOf(app);
    setApp(order[(at + step + order.length) % order.length] ?? ALL);
  };

  const onKeyDown = (e: React.KeyboardEvent) => {
    if (editing) return;
    const ctrl = e.ctrlKey || e.metaKey;
    if (e.key === 'ArrowDown' || (ctrl && e.key === 'j')) {
      setSelected((s) => Math.min(s + 1, visible.length - 1));
    } else if (e.key === 'ArrowUp' || (ctrl && e.key === 'k')) {
      setSelected((s) => Math.max(s - 1, 0));
    } else if (e.key === 'Enter') {
      if (current) void copy(current);
    } else if (e.key === 'Tab') {
      cycleApp(e.shiftKey ? -1 : 1);
    } else if (ctrl && e.key === 'n') {
      setEditing({ id: '', app, kind: 'key', keys: '', desc: '', tags: [] });
    } else if (ctrl && e.key === 'e') {
      if (current) setEditing(current);
    } else if (e.key === 'Escape') {
      if (query) setQuery('');
      else if (app !== ALL) setApp(ALL);
      else void api.hide();
    } else {
      return;
    }
    e.preventDefault();
  };

  // The search box is disabled while the editor is open, so it can only take
  // the focus back once the editor has gone.
  useEffect(() => {
    if (!editing) input.current?.focus();
  }, [editing]);

  const closeEditor = () => setEditing(null);

  return (
    <div className="shell" onKeyDown={onKeyDown}>
      <header className="search" data-tauri-drag-region>
        <input
          ref={input}
          value={query}
          onChange={(e) => setQuery(e.target.value)}
          placeholder="搜索按键、命令或作用，例如：分屏、Ctrl-W、yazi 删除"
          aria-label="搜索"
          autoFocus
          spellCheck={false}
          disabled={editing !== null}
        />
        <span className="count">{visible.length} 条</span>
      </header>

      <div className="body">
        <nav className="rail" aria-label="应用">
          {[ALL, ...apps].map((name) => (
            <button
              key={name}
              type="button"
              className={name === app ? 'active' : ''}
              tabIndex={-1}
              onClick={() => {
                setApp(name);
                input.current?.focus();
              }}
            >
              <span>{name === ALL ? '全部' : name}</span>
              <span className="n">{name === ALL ? matched.length : (counts.get(name) ?? 0)}</span>
            </button>
          ))}
        </nav>

        <main className="main">
          {editing ? (
            <Editor
              key={editing.id}
              item={editing}
              apps={apps}
              onCancel={closeEditor}
              onSave={async (item) => {
                await api.save(item);
                await reload();
                closeEditor();
              }}
              onDelete={async (id) => {
                await api.remove(id);
                await reload();
                closeEditor();
              }}
            />
          ) : loadError ? (
            <div className="empty">
              <p>数据文件读不出来，所以没有改动它。</p>
              <p className="detail">{loadError}</p>
              <button type="button" onClick={() => void api.openDataFolder()}>
                打开数据文件夹
              </button>
            </div>
          ) : visible.length === 0 ? (
            <div className="empty">
              <p>{query ? `没有匹配“${query}”的条目。` : '这里还没有条目。'}</p>
              <p className="detail">按 Ctrl+N 记下一条。</p>
            </div>
          ) : (
            <ul className="results" ref={list} role="listbox" aria-label="结果">
              {visible.map((item, index) => (
                <li
                  key={item.id}
                  role="option"
                  aria-selected={item === current}
                  className={item.kind}
                  onMouseDown={(e) => e.preventDefault()}
                  onClick={() => setSelected(index)}
                  onDoubleClick={() => void copy(item)}
                >
                  <span className="keys">
                    {segments(item.keys).map((s, i) =>
                      s.literal ? <kbd key={i}>{s.text}</kbd> : <i key={i}>{s.text}</i>,
                    )}
                  </span>
                  <span className="desc">
                    {item.desc.includes('`')
                      ? segments(item.desc).map((s, i) =>
                          s.literal ? <code key={i}>{s.text}</code> : <span key={i}>{s.text}</span>,
                        )
                      : item.desc}
                  </span>
                  <span className="where">
                    {app === ALL && <b>{item.app}</b>}
                    {item.tags.join(' ')}
                  </span>
                </li>
              ))}
            </ul>
          )}
        </main>
      </div>

      <footer className="hints">
        {notice ? (
          <span className="notice">{notice}</span>
        ) : editing ? (
          <span>
            <kbd>Enter</kbd> 保存 <kbd>Esc</kbd> 取消
          </span>
        ) : (
          <span>
            <kbd>↑↓</kbd> 选择 <kbd>Enter</kbd> 复制 <kbd>Tab</kbd> 切换应用 <kbd>Ctrl+N</kbd> 新增{' '}
            <kbd>Ctrl+E</kbd> 编辑 <kbd>Esc</kbd> 隐藏
          </span>
        )}
        <span className="spacer" />
        {info &&
          (info.hotkey ? (
            <span>
              <kbd>{info.hotkey}</kbd> 随时呼出
            </span>
          ) : (
            <span className="warn">全局热键被占用，请从托盘图标打开</span>
          ))}
      </footer>
    </div>
  );
}
