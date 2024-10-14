use crossdb::Connection;

fn main() {
    let conn = Connection::open("test.db").unwrap();
    let mut rst = conn.exec("select * FROM system.databases;").unwrap();

    for (name, datatype) in rst.columns() {
        println!("Column : {} {}", name, datatype);
    }

    while let Some(row) = rst.fetch_row() {
        dbg!(row.values());
    }
}
