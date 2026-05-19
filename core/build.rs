fn main() {
    println!("cargo:rerun-if-changed=src/rnbo_bridge.cpp");
    println!("cargo:rerun-if-changed=rnbo_exported/RNBO.cpp");
    println!("cargo:rerun-if-changed=rnbo_exported/RNBO.h");
    cc::Build::new()
        .cpp(true)
        .std("c++11")
        .file("src/rnbo_bridge.cpp")
        .file("rnbo_exported/RNBO.cpp")
        .include("rnbo_exported")
        .compile("rnbo_engine");
}
