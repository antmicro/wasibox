# Wasibox

Copyright (c) 2022-2023 [Antmicro](https://www.antmicro.com)

Wasibox is a collection of utilities targetted mainly at `wasi` architecture.

## Build for wasi target
You will need a custom Rust nightly toolchain that builds [`wasi_ext_lib`](https://github.com/antmicro/wasi_ext_lib) project. Get the custom Rust compiler by following the intructions in [`build`](https://github.com/antmicro/wasi_ext_lib#build) and [`Rust library`](https://github.com/antmicro/wasi_ext_lib#rust-library) sections. 

After completing the previous steps; you need to define `CC` with [`WASI_SDK_PATH`](https://github.com/antmicro/wasi_ext_lib#build) environment variable before you start building the project:

```
wasi_sdk="${WASI_SDK_PATH}/bin/clang --sysroot=${WASI_SDK_PATH}/share/wasi-sysroot"

CC=$wasi_sdk cargo +stage2 build --target wasm32-wasi --release
```