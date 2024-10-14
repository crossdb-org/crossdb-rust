# crossdb-rs

```toml
[dependencies]
crossdb = { git = "https://github.com/crossdb-org/crossdb-rust" }
```

```rs
use crossdb::{Connection, Result};

fn main() -> Result<()> {
    let mut conn = Connection::open_with_memory()?;

    conn.execute("create table if not exists users(id int, name CHAR(255));")?;
    let stmt = conn.prepare("insert into users (id, name) values (?, ?);")?;

    stmt.execute((1, "Alex"))?;
    stmt.execute((2, "Thorne"))?;
    stmt.execute((3, "Ryder"))?;

    let mut query = conn.query("select * from users;")?;

    for (name, datatype) in query.columns() {
        println!("Column : {} {}", name, datatype);
    }

    while let Some(row) = query.fetch_row() {
        println!("User: {}, Name: {}", row.get("id"), row.get("name"));
    }

    Ok(())
}
```

## TODO

- NULL value
- Windows support
- Dynamic link crossdb
- use serde to serialize/deserialize
