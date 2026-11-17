import { useEffect, useRef, useState, type FormEvent } from 'react';
import type { Item, Kind } from './search';

interface Props {
  item: Item;
  apps: string[];
  onSave: (item: Item) => Promise<void>;
  onDelete: (id: string) => Promise<void>;
  onCancel: () => void;
}

export function Editor({ item, apps, onSave, onDelete, onCancel }: Props) {
  const [app, setApp] = useState(item.app);
  const [kind, setKind] = useState<Kind>(item.kind);
  const [keys, setKeys] = useState(item.keys);
  const [desc, setDesc] = useState(item.desc);
  const [tags, setTags] = useState(item.tags.join(', '));
  const [error, setError] = useState('');
  const [confirmDelete, setConfirmDelete] = useState(false);
  const first = useRef<HTMLInputElement>(null);
  const isNew = item.id === '';

  useEffect(() => first.current?.focus(), []);

  const submit = async (event: FormEvent) => {
    event.preventDefault();
    try {
      await onSave({ id: item.id, app, kind, keys, desc, tags: tags.split(/[,，]/) });
    } catch (e) {
      setError(String(e));
    }
  };

  const remove = async () => {
    if (!confirmDelete) return setConfirmDelete(true);
    try {
      await onDelete(item.id);
    } catch (e) {
      setError(String(e));
    }
  };

  return (
    <form
      className="editor"
      onSubmit={submit}
      onKeyDown={(e) => {
        if (e.key === 'Escape') {
          e.stopPropagation();
          onCancel();
        }
      }}
    >
      <h1>{isNew ? '新增条目' : '编辑条目'}</h1>

      <div className="field-row">
        <label className="field">
          <span>应用</span>
          <input
            ref={first}
            value={app}
            onChange={(e) => setApp(e.target.value)}
            list="apps"
            placeholder="Vim、VS Code、Windows"
            required
          />
          <datalist id="apps">
            {apps.map((a) => (
              <option key={a} value={a} />
            ))}
          </datalist>
        </label>

        <div className="field kind" role="radiogroup" aria-label="类型">
          <span>类型</span>
          <div>
            <label>
              <input type="radio" name="kind" checked={kind === 'key'} onChange={() => setKind('key')} />
              按键
            </label>
            <label>
              <input
                type="radio"
                name="kind"
                checked={kind === 'command'}
                onChange={() => setKind('command')}
              />
              命令
            </label>
          </div>
        </div>
      </div>

      <label className="field">
        <span>{kind === 'key' ? '按键' : '命令'}</span>
        <input
          className="mono"
          value={keys}
          onChange={(e) => setKeys(e.target.value)}
          placeholder={kind === 'key' ? '`Ctrl-W` `q`' : '`devstack restart mom`'}
          required
        />
        <small>用反引号 ` 包住要按或要输入的部分，其余文字当作说明。Enter 复制第一段。</small>
      </label>

      <label className="field">
        <span>作用</span>
        <input value={desc} onChange={(e) => setDesc(e.target.value)} placeholder="关闭当前分屏" />
      </label>

      <label className="field">
        <span>标签</span>
        <input value={tags} onChange={(e) => setTags(e.target.value)} placeholder="分屏, 窗口" />
      </label>

      {error && <p className="form-error">{error}</p>}

      <div className="actions">
        {!isNew && (
          <button type="button" className="danger" onClick={remove}>
            {confirmDelete ? '再按一次确认删除' : '删除'}
          </button>
        )}
        <span className="spacer" />
        <button type="button" onClick={onCancel}>
          取消
        </button>
        <button type="submit" className="primary">
          保存
        </button>
      </div>
    </form>
  );
}
