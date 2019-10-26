use akumuli_rs::{self, DB, DBConfig};

fn main() {
	env_logger::init();
	let db = DB::open_or_create("example_db", &DBConfig::default()).unwrap();
	let session = db.create_session().unwrap();
	session.write("random tag=val", 1.1).unwrap();
}