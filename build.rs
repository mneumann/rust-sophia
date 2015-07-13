use std::process::Command;

fn main() {
    let mut cmd = Command::new("make");
    cmd.arg("static").current_dir("sophia");
    cmd.status().unwrap();
    println!("cargo:rustc-link-search=native=./sophia");
    println!("cargo:rustc-link-lib=static=sophia");
}
