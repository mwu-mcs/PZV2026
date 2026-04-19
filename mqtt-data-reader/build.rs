fn main() {
    cxx_build::bridge("src/lib.rs")
        .include("src/include")
        .flag_if_supported("-std=c++14")
        .compile("datasource_bridge");
}
