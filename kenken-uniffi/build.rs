fn main() {
    println!("cargo:rerun-if-changed=src/keen.udl");
    uniffi::generate_scaffolding("src/keen.udl").expect("uniffi scaffolding");
}
