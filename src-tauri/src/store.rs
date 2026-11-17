//! The shortcut list: one hand-editable JSON file in the app data directory.
//!
//! The file is read on every call, so edits made in a text editor show up the
//! next time the window is summoned. A file that does not parse is never
//! overwritten; the error goes to the UI instead.

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

const SEED: &str = include_str!("../seed/shortcuts.json");
const FILE_NAME: &str = "shortcuts.json";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Kind {
    Key,
    Command,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Item {
    #[serde(default)]
    pub id: String,
    pub app: String,
    pub kind: Kind,
    /// The key combination or command. Backticks mark the literal parts.
    pub keys: String,
    #[serde(default)]
    pub desc: String,
    #[serde(default)]
    pub tags: Vec<String>,
}

#[derive(Serialize, Deserialize)]
struct FileData {
    version: u32,
    items: Vec<Item>,
}

pub struct Store {
    path: PathBuf,
    // Serializes read-modify-write cycles.
    lock: Mutex<()>,
}

impl Store {
    pub fn new(dir: &Path) -> Self {
        Self {
            path: dir.join(FILE_NAME),
            lock: Mutex::new(()),
        }
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn list(&self) -> Result<Vec<Item>, String> {
        let _guard = self.lock.lock().unwrap_or_else(|e| e.into_inner());
        self.read()
    }

    /// Adds the item, or replaces the one with the same id.
    pub fn save(&self, mut item: Item) -> Result<Item, String> {
        item.app = item.app.trim().to_string();
        item.keys = item.keys.trim().to_string();
        item.desc = item.desc.trim().to_string();
        item.tags = item
            .tags
            .iter()
            .map(|t| t.trim().to_string())
            .filter(|t| !t.is_empty())
            .collect();
        if item.app.is_empty() {
            return Err("应用不能为空".into());
        }
        if item.keys.is_empty() {
            return Err("按键或命令不能为空".into());
        }

        let _guard = self.lock.lock().unwrap_or_else(|e| e.into_inner());
        let mut items = self.read()?;
        if item.id.is_empty() {
            item.id = new_id();
        }
        match items.iter_mut().find(|i| i.id == item.id) {
            Some(existing) => *existing = item.clone(),
            None => items.push(item.clone()),
        }
        self.write(items)?;
        Ok(item)
    }

    pub fn delete(&self, id: &str) -> Result<(), String> {
        let _guard = self.lock.lock().unwrap_or_else(|e| e.into_inner());
        let mut items = self.read()?;
        items.retain(|i| i.id != id);
        self.write(items)
    }

    fn read(&self) -> Result<Vec<Item>, String> {
        if !self.path.exists() {
            let seed: FileData = serde_json::from_str(SEED).map_err(|e| e.to_string())?;
            self.write(seed.items)?;
        }
        let text = fs::read_to_string(&self.path)
            .map_err(|e| format!("读取 {} 失败：{e}", self.path.display()))?;
        let data: FileData = serde_json::from_str(&text)
            .map_err(|e| format!("{} 无法解析：{e}", self.path.display()))?;
        Ok(data.items)
    }

    /// Writes to a temporary file first, so a crash never leaves half a file.
    fn write(&self, items: Vec<Item>) -> Result<(), String> {
        let err = |e: std::io::Error| format!("写入 {} 失败：{e}", self.path.display());
        if let Some(dir) = self.path.parent() {
            fs::create_dir_all(dir).map_err(err)?;
        }
        let text = serde_json::to_string_pretty(&FileData { version: 1, items })
            .map_err(|e| e.to_string())?;
        let tmp = self.path.with_extension("json.tmp");
        fs::write(&tmp, text + "\n").map_err(err)?;
        fs::rename(&tmp, &self.path).map_err(err)
    }
}

fn new_id() -> String {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or_default();
    format!("u{nanos:x}")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_store(name: &str) -> Store {
        let dir = std::env::temp_dir().join(format!("kiroshi-test-{name}-{}", new_id()));
        Store::new(&dir)
    }

    fn item(keys: &str) -> Item {
        Item {
            id: String::new(),
            app: "Vim".into(),
            kind: Kind::Key,
            keys: keys.into(),
            desc: "测试".into(),
            tags: vec![" a ".into(), "".into()],
        }
    }

    #[test]
    fn first_read_writes_the_seed() {
        let store = temp_store("seed");
        let items = store.list().unwrap();
        assert!(items.len() > 50);
        assert!(store.path().exists());
    }

    #[test]
    fn save_adds_then_replaces_and_delete_removes() {
        let store = temp_store("crud");
        let before = store.list().unwrap().len();

        let saved = store.save(item("`gf`")).unwrap();
        assert!(!saved.id.is_empty());
        assert_eq!(saved.tags, vec!["a".to_string()]);
        assert_eq!(store.list().unwrap().len(), before + 1);

        let mut changed = saved.clone();
        changed.desc = "改过".into();
        store.save(changed).unwrap();
        let items = store.list().unwrap();
        assert_eq!(items.len(), before + 1);
        assert_eq!(items.last().unwrap().desc, "改过");

        store.delete(&saved.id).unwrap();
        assert_eq!(store.list().unwrap().len(), before);
    }

    #[test]
    fn empty_keys_are_rejected() {
        let store = temp_store("empty");
        assert!(store.save(item("  ")).is_err());
    }

    #[test]
    fn a_broken_file_is_reported_and_left_alone() {
        let store = temp_store("broken");
        store.list().unwrap();
        fs::write(store.path(), "{ not json").unwrap();
        assert!(store.list().is_err());
        assert!(store.save(item("`gf`")).is_err());
        assert_eq!(fs::read_to_string(store.path()).unwrap(), "{ not json");
    }
}
