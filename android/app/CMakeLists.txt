# Placeholder CMakeLists for Rust integration
# Actual native build is handled by cargo-ndk plugin

cmake_minimum_required(VERSION 3.4.1)

project("secure_messenger")

# Rust library will be loaded via System.loadLibrary()
# in the Core.kt object initializer
