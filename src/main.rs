extern crate sophia;
fn main() {
    let mut env = sophia::Env::new();
    env.setattr("sophia.path", "./storage");
    env.db("test");
    env.open();
    let mut db = env.get_db("test").unwrap();

    // Write keys
    /*for i in 0 .. 10_000_000 {
        let k = format!("{}", i);
        let s = k.as_bytes();
        db.set(s, s);
    }
    */

    let val = String::from_utf8(db.get(b"104444").unwrap());
    println!("{:?}", val);

    env.destroy();
}
