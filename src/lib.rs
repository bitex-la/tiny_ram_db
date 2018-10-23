/* TODO:
 *  - Integer IDS, index by strings.
 *  - Auto ID on insert. (Also: Insert with ID?)
 *  - Use Error Chain
 *  - Make db available to all Records.
 *  - Serialize and deserialize db from jsonapi.
 *  - Proper tests module
 */
#[macro_use]
extern crate error_chain;

use std::cmp::Eq;
use std::collections::{HashMap, HashSet};
use std::hash::{Hash, Hasher};
use std::string::ToString;
use std::sync::{Arc, RwLock};

pub struct Record<T> {
    pub id: Arc<String>,
    pub data: Arc<T>,
}

impl<T> Clone for Record<T> {
    fn clone(&self) -> Self {
        Self {
            id: Arc::clone(&self.id),
            data: Arc::clone(&self.data),
        }
    }
}

impl<T> PartialEq for Record<T> {
    fn eq(&self, other: &Record<T>) -> bool {
        self.id == other.id
    }
}

impl<T> Eq for Record<T> {}

impl<T> Hash for Record<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

pub struct Table<T, Indexes> {
    pub data: Arc<RwLock<HashMap<String, Record<T>>>>,
    pub indexes: Indexes,
}

impl<T, Indexes: Indexer<Item = T>> Table<T, Indexes> {
    pub fn new() -> Self {
        Self {
            data: Arc::new(RwLock::new(HashMap::new())),
            indexes: Default::default(),
        }
    }

    pub fn find<I: AsRef<str>>(&self, id: I) -> Record<T> {
        self.data.read().expect("Table read lock").get(id.as_ref()).expect("RecordNotFound").clone()
    }

    pub fn insert<I: ToString>(&mut self, id: I, value: T) -> Record<T> {
        let record = Record {
            id: Arc::new(id.to_string()),
            data: Arc::new(value),
        };
        self.data.write().expect("Table write lock").insert(id.to_string(), record.clone());
        self.indexes.index(&record);
        record
    }
}

/* PlainTable is a bit of duplication, but makes the API clearer.
 * Once 'never' type is stable we can implement PlainTable as an alias
 * of Table with a no-op indexer
 */
pub struct PlainTable<T> {
    pub data: Arc<RwLock<HashMap<String, Record<T>>>>,
}

impl<T> PlainTable<T> {
    pub fn new() -> Self {
        Self {
            data: Arc::new(RwLock::new(HashMap::new()))
        }
    }

    pub fn find<I: AsRef<str>>(&self, id: I) -> Record<T> {
        self.data.read().expect("Table read lock").get(id.as_ref()).expect("RecordNotFound").clone()
    }

    pub fn insert<I: ToString>(&mut self, id: I, value: T) -> Record<T> {
        let record = Record {
            id: Arc::new(id.to_string()),
            data: Arc::new(value),
        };
        self.data.write().expect("Table write lock").insert(id.to_string(), record.clone());
        record
    }
}

pub trait Indexer: Default {
    type Item;

    fn index(&mut self, _item: &Record<Self::Item>) {}
}

pub struct Index<K: Eq + Hash, V> {
    pub data: RwLock<HashMap<K, HashSet<Record<V>>>>,
}

impl<K: Eq + Hash, V> Default for Index<K, V> {
    fn default() -> Self {
        Self {
            data: RwLock::new(HashMap::new()),
        }
    }
}

impl<K: Eq + Hash, V> Index<K, V> {
    pub fn insert(&mut self, k: K, record: Record<V>) {
        let mut inner_data = self.data.write().expect("ErrorWritingDatabase");
        let value = inner_data.entry(k).or_insert(HashSet::new());
        value.insert(record);
    }

    pub fn get<F, A>(&self, k: &K, closure: F) -> A
        where F: FnOnce(&HashSet<Record<V>>) -> A
    {
        let hashmap = self.data.read().expect("ErrorReadingDatabase");
        closure(hashmap.get(k).expect("RecordNotFound"))
    }
}
