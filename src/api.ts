import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import type { Item } from './search';

export interface AppInfo {
  version: string;
  /** The global hotkey that is registered, or null if none could be. */
  hotkey: string | null;
  dataFile: string;
}

const inTauri = '__TAURI_INTERNALS__' in window;

// Outside Tauri (`pnpm dev` in a browser) the UI runs on the seed data, kept
// in memory, so it can be worked on without building the app.
let memory: Item[] | null = null;
async function mem(): Promise<Item[]> {
  if (!memory) {
    const seed = await import('../src-tauri/seed/shortcuts.json');
    memory = seed.default.items as Item[];
  }
  return memory;
}

export const api = {
  async info(): Promise<AppInfo> {
    if (inTauri) return invoke('app_info');
    return { version: 'dev', hotkey: 'Alt+H', dataFile: '(浏览器预览，改动不会保存)' };
  },

  async list(): Promise<Item[]> {
    if (inTauri) return invoke('list_items');
    return [...(await mem())];
  },

  async save(item: Item): Promise<Item> {
    if (inTauri) return invoke('save_item', { item });
    const items = await mem();
    const saved = { ...item, id: item.id || `u${Date.now()}` };
    const at = items.findIndex((i) => i.id === saved.id);
    if (at >= 0) items[at] = saved;
    else items.push(saved);
    return saved;
  },

  async remove(id: string): Promise<void> {
    if (inTauri) return invoke('delete_item', { id });
    memory = (await mem()).filter((i) => i.id !== id);
  },

  async hide(): Promise<void> {
    if (inTauri) await invoke('hide_window');
  },

  async openDataFolder(): Promise<void> {
    if (inTauri) await invoke('open_data_folder');
  },

  /** Calls back each time the window is summoned. Returns the unsubscribe. */
  onShown(callback: () => void): () => void {
    if (!inTauri) return () => {};
    const unlisten = listen('shown', callback);
    return () => void unlisten.then((off) => off());
  },
};
