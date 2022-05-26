<div align=center>
	<h1>rust_os</h1>
</div>

This repository maintains a rust project that tries to create a simple kernel using rust. The kernel as of now can only print on screen using vga buffer and is not interactive.

## Dependencies

- [bootimage](https://docs.rs/bootimage/latest/bootimage/)
- [bootloader](https://docs.rs/bootloader/latest/bootloader/)
- [x86_64](https://docs.rs/x86_64/latest/x86_64/)
- [spin](https://docs.rs/spin/latest/spin/)
- [uart_16650](https://docs.rs/uart_16550/latest/uart_16550/)
- [volatile](https://docs.rs/volatile/latest/volatile/)

## Run kernel on QEMU

### Build bootable image
- First, install rust's nightly version for your operating system.
- Then install rust-src, by running `rustup component add rust-src`.
- Install llvm preview tools for rust, `rustup component add llvm-tools-preview`.
- Now install bootimage to build image, run `cargo install bootimage`.
- To build it, go to root directory and run `cargo build && cargo bootimage`.
- This will create a `.bin` bootable image in `target/x86_64/debug/`.

### Install QEMU

- First, install QEMU/Virt-Manager on your machine, and then install sdl display for QEMU.
- Then reboot your machine to load all dependencies.
- Run `qemu-system-x86_64 -drive format=raw,file=<build-bin-location> -display sdl`.