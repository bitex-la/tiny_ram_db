extern crate tiny_ram_db;
use std::string::ToString;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Instant;
use tiny_ram_db::{Index, Indexer, Record, Table, PlainTable};

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
    authors: PlainTable<Author>,
    posts: Table<Post, PostIndexes>,
}

fn create_db() -> Arc<Mutex<Database>> {
    let db: Arc<Mutex<Database>> = Arc::new(Mutex::new(Database {
        authors: PlainTable::new(),
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
