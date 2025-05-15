# crossdb-rust

```toml
[dependencies]
crossdb = { git = "https://github.com/crossdb-org/crossdb-rust" }
```

```rs
#[derive(Debug, serde::Deserialize)]
struct User {
    id: i32,
    name: String,
    age: Option<i8>,
}

fn main() -> crossdb::Result<()> {
    let mut conn = crossdb::Connection::open_with_memory()?;

    conn.execute("CREATE TABLE IF NOT EXISTS users(id INT, name VARCHAR, age TINYINT);")?;
    let stmt = conn.prepare("INSERT INTO users (id, name, age) values (?, ?, ?);")?;

    stmt.execute((1, "Alex", 18))?;
    stmt.execute((2, "Thorne", 22))?;
    stmt.execute((3, "Ryder", 36))?;

    let mut query = conn.query("SELECT * FROM users;")?;

    for col in query.columns() {
        println!("Column: {col}");
    }

    while let Some(user) = query.fetch_row_as::<User>() {
        dbg!(user);
    }

    let affected_rows = conn.execute("DELETE FROM users;")?;
    assert_eq!(affected_rows, 3);

    Ok(())
}
```
