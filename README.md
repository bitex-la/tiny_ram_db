# Tiny RAM DB

A RAM Database to allow fast access to data.

## How to Install

```toml
[dependencies]
tiny_ram_db = "0.1.0"

```

## How to use it

```rust
    extern crate tiny_ram_db;

    use tiny_ram_db::HasMany;
    use tiny_ram_db::Record;
    use tiny_ram_db::Table;

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

        Database {
            bars: Table::new(),
            bar_friends: Table::new(),
            foos,
            foo_friends,
        }
    }
```