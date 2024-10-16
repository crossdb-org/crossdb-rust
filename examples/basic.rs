use crossdb::Connection;

fn main() {
    let conn = Connection::open("test").unwrap();
    let mut query = conn.query("select * from system.databases;").unwrap();

    for col in query.columns() {
        println!("Column: {col}");
    }

    while let Some(row) = query.fetch_row() {
        dbg!(row);
    }
}
