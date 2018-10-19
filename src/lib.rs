use std::cell::RefCell;
use std::collections::HashMap;
use std::sync::Arc;

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

pub struct Table<T> {
    pub data: HashMap<String, Record<T>>,
}

impl<T> Table<T> {
    pub fn new() -> Self {
        Self {
            data: HashMap::new(),
        }
    }

    pub fn find<I: AsRef<str>>(&self, id: I) -> Record<T> {
        self.data.get(id.as_ref()).expect("RecordNotFound").clone()
    }

    pub fn insert<I>(&mut self, id: I, value: T) -> Record<T>
    where
        I: AsRef<str> + std::string::ToString,
    {
        let record = Record {
            id: Arc::new(id.to_string()),
            data: Arc::new(value),
        };
        self.data.insert(id.to_string(), record.clone());
        record
    }
}

pub struct HasMany<T> {
    pub data: RefCell<Vec<Record<T>>>,
}

impl<T> HasMany<T> {
    pub fn new() -> Self {
        Self {
            data: RefCell::new(Vec::new()),
        }
    }

    pub fn push(&self, record: &Record<T>) {
        self.data.borrow_mut().push(record.clone());
    }

    pub fn get(&self, index: usize) -> Record<T> {
        self.data.borrow()[index].clone()
    }
}

#[cfg(test)]
mod tests {
    use HasMany;
    use Record;
    use Table;

    struct Foo {
        name: String,
        foo_friends: HasMany<FooFriend>,
    }

    impl Foo {
        fn new(name: String) -> Self {
            Foo {
                name,
                foo_friends: HasMany::new(),
            }
        }
    }

    struct FooFriend {
        name: String,
        foo: Record<Foo>,
    }

    struct Bar {
        name: String,
        bar_friends: HasMany<BarFriend>,
    }

    struct BarFriend {
        name: String,
        bar: Record<Bar>,
    }

    struct Database {
        foos: Table<Foo>,
        bars: Table<Bar>,
        foo_friends: Table<FooFriend>,
        bar_friends: Table<BarFriend>,
    }

    fn create_db() -> Database {
        let mut foos: Table<Foo> = Table::new();
        let mut foo_friends: Table<FooFriend> = Table::new();

        let one_foo: Record<Foo> = foos.insert("1", Foo::new("one_foo".into()));

        // 11.90s, SSD, 16Gb RAM, 4 cores i5 Intel
        for _x in 0..500_000 {
            let one = foo_friends.insert(
                "1",
                FooFriend {
                    name: "friend 1".into(),
                    foo: one_foo.clone(),
                },
            );
            let two = foo_friends.insert(
                "2",
                FooFriend {
                    name: "friend 2".into(),
                    foo: one_foo.clone(),
                },
            );
            let three = foo_friends.insert(
                "3",
                FooFriend {
                    name: "friend 3".into(),
                    foo: one_foo.clone(),
                },
            );

            one_foo.data.foo_friends.push(&one);
            one_foo.data.foo_friends.push(&two);
            one_foo.data.foo_friends.push(&three);

            let two_foo: Record<Foo> = foos.insert("2", Foo::new("two_foo".into()));
            let four = foo_friends.insert(
                "4",
                FooFriend {
                    name: "friend 4".into(),
                    foo: two_foo.clone(),
                },
            );
            let five = foo_friends.insert(
                "5",
                FooFriend {
                    name: "friend 5".into(),
                    foo: two_foo.clone(),
                },
            );

            two_foo.data.foo_friends.push(&one);
            two_foo.data.foo_friends.push(&two);
            two_foo.data.foo_friends.push(&three);
            two_foo.data.foo_friends.push(&four);
            two_foo.data.foo_friends.push(&five);
        }

        Database {
            bars: Table::new(),
            bar_friends: Table::new(),
            foos,
            foo_friends,
        }
    }

    #[test]
    fn obtain_data() {
        let db = create_db();
        let second_friend = db.foos.find("1").data.foo_friends.get(1);
        println!("Second foo friend id: {:#?}", second_friend.id);
        println!("Second foo friend name: {:#?}", second_friend.data.name);
        assert!(second_friend == db.foo_friends.find("2"));
        assert!(second_friend.data.foo == db.foos.find("1"));
    }
}
