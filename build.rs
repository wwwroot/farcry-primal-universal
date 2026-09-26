fn main() {
    println!("cargo:rustc-link-arg=resource.res");
    println!("cargo:rerun-if-changed=resource.rc");
}
