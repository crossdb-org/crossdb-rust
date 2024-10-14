use crossdb::Connection;

fn main() {
    let conn = Connection::open("test").unwrap();
    let mut query = conn.query("select * from system.databases;").unwrap();

    for (name, datatype) in query.columns() {
        println!("Column : {} {}", name, datatype);
    }

    while let Some(row) = query.fetch_row() {
        dbg!(row.values());
    }
}
