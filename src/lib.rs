/* TODO:
 *  - Make db available to all Records.
 *  - Serialize and deserialize db from jsonapi.
 */
#[macro_use]
extern crate error_chain;

#[macro_use]
extern crate serde_derive;

extern crate serde;
extern crate serde_json;

use std::cmp::Eq;
use std::collections::{HashMap, HashSet};
use std::hash::{Hash, Hasher};
use std::sync::{Arc, RwLock};

pub mod errors;
use errors::*;

#[derive(Debug, Serialize, Deserialize)]
pub struct Record<T> {
    pub id: Option<usize>,
    pub data: Arc<T>,
}

impl<T> Clone for Record<T> {
    fn clone(&self) -> Self {
        Self {
            id: self.id,
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

#[derive(Debug, Serialize, Deserialize)]
pub struct Table<T, Indexes> {
    pub data: Arc<RwLock<Vec<Record<T>>>>,
    pub indexes: Arc<RwLock<Indexes>>,
}

impl<T, Indexes: Indexer<Item = T>> Clone for Table<T, Indexes> {
    fn clone(&self) -> Self {
        Self {
            data: Arc::clone(&self.data),
            indexes: Arc::clone(&self.indexes),
        }
    }
}

impl<T, Indexes: Indexer<Item = T>> Table<T, Indexes> {
    pub fn new() -> Self {
        Self {
            data: Arc::new(RwLock::new(Vec::new())),
            indexes: Default::default(),
        }
    }

    pub fn find(&self, id: usize) -> Result<Record<T>> {
        match self.data.read()?.get(id) {
            Some(entry) => Ok(entry.clone()),
            _ => bail!(ErrorKind::RecordNotFound("".into())),
        }
    }

    pub fn insert(&mut self, value: T) -> Result<Record<T>> {
        let mut table = self.data.write()?;
        let id = table.len() + 1;
        let record = Record {
            id: Some(id),
            data: Arc::new(value),
        };
        table.push(record.clone());
        self.indexes.write()?.index(&record)?;
        Ok(record)
    }
}

/* PlainTable is a bit of duplication, but makes the API clearer.
 * Once 'never' type is stable we can implement PlainTable as an alias
 * of Table with a no-op indexer
 */
#[derive(Debug, Serialize, Deserialize)]
pub struct PlainTable<T> {
    pub data: Arc<RwLock<Vec<Record<T>>>>,
}

impl<T> PlainTable<T> {
    pub fn new() -> Self {
        Self {
            data: Arc::new(RwLock::new(Vec::new())),
        }
    }

    pub fn find(&self, id: usize) -> Result<Record<T>> {
        match self.data.read()?.get(id) {
            Some(entry) => Ok(entry.clone()),
            _ => bail!(ErrorKind::RecordNotFound("".into())),
        }
    }

    pub fn insert(&mut self, value: T) -> Result<Record<T>> {
        let mut table = self.data.write()?;
        let id = table.len() + 1;
        let record = Record {
            id: Some(id),
            data: Arc::new(value),
        };
        table.push(record.clone());
        Ok(record)
    }
}

impl<T> Clone for PlainTable<T> {
    fn clone(&self) -> Self {
        Self {
            data: Arc::clone(&self.data),
        }
    }
}

pub trait Indexer: Default {
    type Item;

    fn index(&mut self, _item: &Record<Self::Item>) -> Result<bool> {
        Ok(true)
    }
}

pub struct Index<K: Eq + Hash, V> {
    pub data: HashMap<K, HashSet<Record<V>>>,
}

impl<K: Eq + Hash, V> Default for Index<K, V> {
    fn default() -> Self {
        Self {
            data: HashMap::new(),
        }
    }
}

impl<K: Eq + Hash, V> Index<K, V> {
    pub fn insert(&mut self, k: K, record: Record<V>) -> Result<bool> {
        Ok(self.data.entry(k).or_insert(HashSet::new()).insert(record))
    }

    pub fn get<F, A>(&self, k: &K, closure: F) -> Result<A>
    where
        F: FnOnce(&HashSet<Record<V>>) -> A,
    {
        Ok(match self.data.get(k) {
            Some(a) => closure(a),
            _ => closure(&HashSet::new()),
        })
    }
}
