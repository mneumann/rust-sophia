extern crate sophia;
extern crate rand;

use std::str;
use std::thread;

const N_KEYS: usize = 100_000_000;
const REPEAT: usize = 10_000_000;

fn write_keys(db: &mut sophia::Db) {
    for i in 0 .. N_KEYS {
        let k = format!("{}", i);
        let s = k.as_bytes();
        db.set(s, s);
    }
}

fn read_keys(db: &mut sophia::Db) {
    use rand::Rng;
    let mut rng = rand::thread_rng();

    for _ in 0 .. REPEAT {
        let i: usize = rng.gen();
        let key = format!("{}", i % N_KEYS);

        let mut obj = db.get(key.as_bytes()).unwrap();
        match obj.get_value() {
            Some(val) => {
                if let Ok(val) = str::from_utf8(val) {
                    assert!(val == key);
                }
            }
            None => {
                println!("key not found");
            }
        }
    }
}

fn main() {
    let mut env = sophia::Env::new();
    env.setattr("sophia.path", "./storage");
    env.setattr("db", "test");
    //env.setattr("db.test.compression", "lz4");
    env.setintattr("db.test.mmap", 1);
    env.open();

    let mut db = env.get_db("test").unwrap();

    write_keys(&mut db);

    println!("starting up...");

    let mut vec = vec![];

    for i in 1..10 {
        let child = thread::spawn(move || {
            let mut mydb = db.clone();
            test_keys(&mut mydb);
        });
        vec.push(child);
    }

    for child in vec {
        child.join();
    }

    env.destroy();
}
