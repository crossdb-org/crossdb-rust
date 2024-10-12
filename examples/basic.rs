use crossdb::Connection;

fn main() {
    let conn = Connection::open("test.db").unwrap();
    let mut rst = conn.exec("select * FROM system.databases;").unwrap();

    for i in 0..rst.column_count() {
        println!("Column {i}: {} {}", rst.column_name(i), rst.column_type(i));
    }

    while let Some(row) = rst.fetch_row() {
        dbg!(row);
    }
}
