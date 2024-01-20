fn main() {
    let script = "linkers/kernel-x86_64.ld";

    println!("cargo:rerun-if-changed={script}");
    println!("cargo:rustc-link-arg=-T{script}");
}
