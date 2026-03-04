fn main() {
    println!("cargo:rerun-if-changed=src/");
    println!("cargo:rerun-if-changed=schemas/");
    // Use vendored protoc to guarantee hermetic portability (Titan Protocol Rule 2)
    let current_dir = std::env::current_dir().unwrap();
    let protoc_path = current_dir.join("protoc_bin/bin/protoc.exe");
    std::env::set_var("PROTOC", protoc_path.to_str().unwrap());

    // Compile Protobuf Schemas
    prost_build::compile_protos(&["schemas/titan.proto"], &["schemas/"]).unwrap();

    // Enable CXX building of our titan_iceoryx wrapper
    // We bind it against actual iceoryx library paths if they exist
    // For compilation to succeed if native libs aren't fully staged yet,
    // we instruct cxx_build to compile our .cpp normally.

    cxx_build::bridge("src/shared_memory.rs")
        .file("src/titan_iceoryx.cpp")
        .include("src")
        .include("third_party/iceoryx/install/include/iceoryx/v2.95.8")
        .flag_if_supported("-std=c++17")
        .flag_if_supported("/std:c++17")
        .compile("titan_iceoryx");

    println!("cargo:rustc-link-search=native={}/third_party/iceoryx/install/lib", current_dir.display());
    println!("cargo:rustc-link-lib=static=iceoryx_posh");
    println!("cargo:rustc-link-lib=static=iceoryx_posh_roudi");
    println!("cargo:rustc-link-lib=static=iceoryx_hoofs");
    println!("cargo:rustc-link-lib=static=iceoryx_platform");
    println!("cargo:rustc-link-lib=dylib=advapi32");
}
