extern crate tiny_ram_db;
extern crate error_chain;
use std::string::ToString;
use std::thread;
use std::time::Instant;
use tiny_ram_db::{Index, Indexer, Record, Table, PlainTable};
use tiny_ram_db::errors::*;

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
    by_text: Index<String, Post>,
}

impl Indexer for PostIndexes {
    type Item = Post;
    fn index(&mut self, item: &Record<Post>) -> Result<bool> {
        self.by_author.insert(item.data.author.clone(), item.clone())?;
        self.by_text.insert(item.data.text.clone(), item.clone())?;
        Ok(true)
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

#[derive(Clone)]
struct Database {
    authors: PlainTable<Author>,
    posts: Table<Post, PostIndexes>,
}

fn create_db() -> Result<Database> {
    let mut db: Database = Database {
        authors: PlainTable::new(),
        posts: Table::new(),
    };

    let bob = db.authors.insert(Author::new("bob"))?;
    let ana = db.authors.insert(Author::new("ana"))?;

    let mut handles = vec![];

    let mut bob_db = db.clone();
    let bob_thread = thread::spawn(move || {
        for x in 0..500_000 {
            bob_db.posts.insert(
                Post::new(&bob, format!("Bob says hello #{}", x)),
            ).unwrap();
        }
    });

    handles.push(bob_thread);

    let mut ana_db = db.clone();
    let ana_thread = thread::spawn(move || {
        for x in 0..500_000 {
            ana_db.posts.insert(
                Post::new(&ana, format!("Ana says hello #{}", x)),
            ).unwrap();
        }
    });

    handles.push(ana_thread);

    for handle in handles {
        handle.join().expect("Error joining handles");
    }

    Ok(db)
}

#[test]
fn obtain_data() {
    obtain_data_result().expect("Error")
}

fn obtain_data_result() -> Result<()> {
    let start = Instant::now();
    let db = create_db()?;
    println!("DB Creation took {:?}", start.elapsed());
    let a_post = db.posts.find(400000)?;
    println!("A post text is: {}", &a_post.data.text);
    println!("A post author is: {}", &a_post.data.author.data.name);
    let by_author = db.posts.indexes.read()?.by_author
        .get(&a_post.data.author, |items| items.len() )?;

    println!("Author total post count is : {}", by_author);

    let by_text = db.posts.indexes.read()?.by_text
        .get(&"Bob says hello #9".into(), |e|
          e.iter().nth(0).unwrap().clone()
        )?;

    println!("Bob post #9 author is {}", by_text.data.author.data.name);

    Ok(())
}
