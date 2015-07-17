#[macro_use(obj)]
extern crate sophia;
extern crate rand;

use std::str;
use std::thread;

const N_KEYS: usize = 100_000_000;
const REPEAT: usize = 10_000_000;

use sophia::SetGetOps;

fn write_keys(env: &sophia::Env, db: &sophia::Db) {
    for i in 0 .. N_KEYS {
        let k = format!("{}", i);
        let s = k.as_bytes();

        let obj = obj![db; key => s, value => s];
        env.set(obj);
    }
}

fn read_keys(env: &sophia::Env, db: &sophia::Db) {
    use rand::Rng;
    let mut rng = rand::thread_rng();

    for _ in 0 .. REPEAT {
        let i: usize = rng.gen();
        let key = format!("{}", i % N_KEYS);

        let obj = obj![db; key => key.as_bytes()];

        let kv = env.get(obj).unwrap();
        match kv.get_value() {
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

    write_keys(&env, &db);

    println!("starting up...");

    let mut vec = vec![];

    for i in 1..10 {
        let child = thread::spawn(move || {
            read_keys(&env, &db);
        });
        vec.push(child);
    }

    for child in vec {
        child.join();
    }

    env.destroy();
}
