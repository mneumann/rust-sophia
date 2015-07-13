use std::str;

fn write_keys(db: &mut sophia::Db) {
    for i in 0 .. 100_000_000 {
        let k = format!("{}", i);
        let s = k.as_bytes();
        db.set(s, s);
    }
}

extern crate sophia;
fn main() {
    let mut env = sophia::Env::new();
    env.setattr("sophia.path", "./storage");
    env.db("test");
    env.setattr("db", "test");
    //env.setattr("db.test.compression", "lz4");
    env.setintattr("db.test.mmap", 1);
    env.open();

    let mut db = env.get_db("test").unwrap();

    //write_keys(&mut db);

    println!("starting up...");
    for i in 0 .. 1_000_000 {
        let k = format!("{}", i);

        let mut obj = db.get(k.as_bytes());
        match obj.get_value() {
            Some(val) => {
                if let Ok(val) = str::from_utf8(val) {
                    //println!("{:?}", val);
                }
            }
            None => {
                println!("key not found");
            }
        }
    }

    env.destroy();
}
