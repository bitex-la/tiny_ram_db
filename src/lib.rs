use std::cell::{RefCell};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use std::string::ToString;
use std::cmp::Eq;
use std::hash::{Hash, Hasher};
use std::marker::PhantomData;

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
    pub data: HashMap<String, Record<T>>,
    pub indexes: Indexes
}

impl<T, Indexes: Indexer<Item=T>> Table<T, Indexes> {
    pub fn new() -> Self {
        Self {
            data: HashMap::new(),
            indexes: Default::default()
        }
    }

    pub fn find<I: AsRef<str>>(&self, id: I) -> Record<T> {
        self.data.get(id.as_ref()).expect("RecordNotFound").clone()
    }

    pub fn insert<I>(&mut self, id: I, value: T) -> Record<T>
    where
        I: AsRef<str> + ToString,
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

  fn index(&self, _item: &Record<Self::Item>){}
}

pub struct NoIndexes<T>(PhantomData<T>);

impl<T> Indexer for NoIndexes<T> {
  type Item = T;
}

impl<T> Default for NoIndexes<T> {
  fn default() -> Self { Self{0: PhantomData} }
}

pub struct Index<K: Eq + Hash , V> {
    pub data: RefCell<HashMap<K, Arc<RefCell<HashSet<Record<V>>>>>>,
}

impl<K: Eq + Hash, V> Default for Index<K, V> {
    fn default() -> Self { 
      Self{ data: RefCell::new(HashMap::new()) }
    }
}

impl<K: Eq + Hash, V> Index<K, V> {
    pub fn insert(&self, k: K, record: Record<V>) {
        let mut index = self.data.borrow_mut();
        let values = index.entry(k)
          .or_insert(Arc::new(RefCell::new(HashSet::new())));
        values.borrow_mut().insert(record);
    }

    pub fn get(&self, k: &K) -> Arc<RefCell<HashSet<Record<V>>>> {
        let hashmap = self.data.borrow();
        let hashset = hashmap.get(k).unwrap();
        Arc::clone(&hashset)
    }
}

#[cfg(test)]
mod tests {
    use NoIndexes;
    use Indexer;
    use Index;
    use Record;
    use Table;
    use std::string::ToString;
    use std::time::Instant;

    struct Author {
        name: String,
    }

    impl Author {
        fn new<I: ToString>(name: I) -> Self {
            Self { name: name.to_string() }
        }
    }

    struct Post {
        text: String,
        author: Record<Author>,
    }

    #[derive(Default)]
    struct PostIndexes {
        by_author: Index<Record<Author>, Post>
    }

    impl Indexer for PostIndexes {
        type Item = Post;
        fn index(&self, item: &Record<Post>){
            self.by_author.insert(item.data.author.clone(), item.clone())
        }
    }

    impl Post {
        fn new<I: ToString>(author: &Record<Author>, text: I) -> Self {
            Self { author: author.clone(), text: text.to_string() }
        }
    }

    struct Database {
        authors: Table<Author, NoIndexes<Author>>,
        posts: Table<Post, PostIndexes>,
    }

    fn create_db() -> Database {
        let mut db : Database = Database {
          authors: Table::new(),
          posts: Table::new(),
        };

        let bob = db.authors.insert("1", Author::new("bob"));
        let ana = db.authors.insert("2", Author::new("ana"));

        for x in 0..500_000 {
            db.posts.insert(
                x.to_string(),
                Post::new(&bob, format!("Bob says hello #{}", x))
            );
            db.posts.insert(
                (1000000 + x).to_string(),
                Post::new(&ana, format!("Ana's recipe #{}", x))
            );
        }
        db
    }

    #[test]
    fn obtain_data() {
        let start = Instant::now();
        let db = create_db();
        println!("DB Creation took {:?}", start.elapsed());
        let a_post = db.posts.find("400000");
        println!("A post text is: {}", &a_post.data.text);
        println!("A post author is: {}", &a_post.data.author.data.name);
        let by_author = &db.posts.indexes.by_author;
        let index = by_author.get(&a_post.data.author);
        let borrowed = index.borrow();
        
        println!("Author total post count is : {}", borrowed.len())
    }
}
