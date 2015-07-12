extern crate sophia;
fn main() {
    let mut env = sophia::Env::new();
    env.setattr("sophia.path", "./storage");
    env.db("test");
    env.open();

    let mut db = env.get_db("db.test");

    db.set(b"hallo", b"leute");
    env.destroy();
}
