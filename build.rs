use std::process::Command;
use std::env;
use std::fs;
use std::path::Path;

fn main() {
    let mut cmd = Command::new("make");
    cmd.arg("static").current_dir("sophia");
    cmd.status().unwrap();

    let dst = env::var("OUT_DIR").unwrap();
    fs::copy("sophia/libsophia.a", Path::new(&dst).join("libsophia.a")).unwrap();
    println!("cargo:rustc-link-search=native={}", dst);
    println!("cargo:rustc-link-lib=static=sophia");
}
