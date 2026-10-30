//! Cargo requires a build script for `links`. No native library is built:
//! the inherited executor identity prevents multiple runtimes in one graph.
fn main() {}
