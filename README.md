# crossdb-rs

```toml
[dependencies]
crossdb = { git = "https://github.com/crossdb-org/crossdb-rust" }
```

```rs
use crossdb::Connection;

fn main() {
    let conn = Connection::open_with_memory().unwrap();
    let mut rst = conn.exec("select * from system.databases;").unwrap();

    for i in 0..rst.column_count() {
        println!("Column {i}: {} {}", rst.column_name(i), rst.column_type(i));
    }

    while let Some(row) = rst.fetch_row() {
        dbg!(row);
    }
}
```

## TODO
* NULL value
* Add more apis
* Windows support
* Dynamic link crossdb
* use serde to serialize/deserialize
