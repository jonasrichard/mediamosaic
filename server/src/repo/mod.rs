use rusqlite::Connection;

pub struct Repository {
    connection: Connection,
}

impl Repository {
    pub fn open() -> Self {
        let connection = Connection::open("./media.db3").expect("Unable to open media database");

        Self { connection }
    }

    pub fn create_schema(&self) {
        if self.connection.table_exists(None, "directory").unwrap() {
            return;
        }

        let _ = self
            .connection
            .execute(
                "create table directory (
                    id integer primary key,
                    path text not null,
                    scanned_at datetime,
                    checksum blob
                )",
                (),
            )
            .unwrap();
    }
}
