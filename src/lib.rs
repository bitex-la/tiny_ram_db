use std::cmp::Eq;
use std::collections::{HashMap, HashSet};
use std::hash::{Hash, Hasher};
use std::marker::PhantomData;
use std::string::ToString;
use std::sync::{Arc, RwLock};
use std::fmt::Display;

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
            data: HashMap::new(),
            indexes: Default::default(),
        }
    }

    pub fn find<I: AsRef<str>>(&self, id: I) -> Record<T> {
        self.data.get(id.as_ref()).expect("RecordNotFound").clone()
    }

    pub fn insert<I>(&mut self, id: I, value: T) -> Record<T>
    where
        I: AsRef<str> + ToString + Display,
    {
        let record = Record {
            id: Arc::new(id.to_string()),
            data: Arc::new(value),
        };
        self.data.insert(id.to_string(), record.clone());
        self.indexes.index(&record);
        record
    }
}

pub trait Indexer: Default {
    type Item;

    fn index(&mut self, _item: &Record<Self::Item>) {}
}

pub struct NoIndexes<T>(PhantomData<T>);

impl<T> Indexer for NoIndexes<T> {
    type Item = T;
}

impl<T> Default for NoIndexes<T> {
    fn default() -> Self {
        Self { 0: PhantomData }
    }
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

#[cfg(test)]
mod tests {
    use std::string::ToString;
    use std::sync::{Arc, Mutex};
    use std::collections::HashSet;
    use std::thread;
    use std::time::Instant;
    use Index;
    use Indexer;
    use NoIndexes;
    use Record;
    use Table;

    struct Author {
        name: String,
    }

    impl Author {
        fn new<I: ToString>(name: I) -> Self {
            Self {
                name: name.to_string(),
            }
        }
    }

    struct Post {
        text: String,
        author: Record<Author>,
    }

    #[derive(Default)]
    struct PostIndexes {
        by_author: Index<Record<Author>, Post>,
    }

    impl Indexer for PostIndexes {
        type Item = Post;
        fn index(&mut self, item: &Record<Post>) {
            self.by_author.insert(item.data.author.clone(), item.clone())
        }
    }

    impl Post {
        fn new<I: ToString>(author: &Record<Author>, text: I) -> Self {
            Self {
                author: author.clone(),
                text: text.to_string(),
            }
        }
    }


    struct Database {
        authors: Table<Author, NoIndexes<Author>>,
        posts: Table<Post, PostIndexes>,
    }

    fn create_db() -> Arc<Mutex<Database>> {
        let db: Arc<Mutex<Database>> = Arc::new(Mutex::new(Database {
            authors: Table::new(),
            posts: Table::new(),
        }));

        let bob = db
            .lock()
            .expect("Bob creating")
            .authors
            .insert("1", Author::new("bob"));
        let ana = db
            .lock()
            .expect("Ana creating")
            .authors
            .insert("2", Author::new("ana"));

        let mut handles = vec![];

        let bob_db = db.clone();

        let bob_thread = thread::spawn(move || {
          for x in 0..500_000 {
            bob_db
                .lock()
                .expect("Bob DB unavailable")
                .posts
                .insert(
                    x.to_string(),
                    Post::new(&bob, format!("Bob says hello #{}", x)),
                );
          }
        });

        handles.push(bob_thread);

        let ana_db = db.clone();
        let ana_thread = thread::spawn(move || {
          for x in 0..500_000 {
            ana_db
                .lock()
                .expect("Ana DB unavailable")
                .posts
                .insert(
                    x.to_string(),
                    Post::new(&ana, format!("Ana says hello #{}", x)),
                );
          }
        });

        handles.push(ana_thread);

        for handle in handles {
            handle.join().expect("Error joining handles");
        }

        db
    }

    #[test]
    fn obtain_data() {
        let start = Instant::now();
        let db = create_db();
        println!("DB Creation took {:?}", start.elapsed());
        let a_post = db.lock().expect("Error a_post").posts.find("400000");
        println!("A post text is: {}", &a_post.data.text);
        println!("A post author is: {}", &a_post.data.author.data.name);
        let by_author = db
            .lock()
            .expect("Error by_author")
            .posts
            .indexes
            .by_author
            .get(&a_post.data.author, |items| items.len() );

        println!("Author total post count is : {}", by_author)
    }
}
