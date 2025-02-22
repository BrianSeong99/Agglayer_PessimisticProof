# Fibonacci: A template project for Valida

This is a simple program that calculates the n-th fibonacci number and proves the execution of the computation in Valida. You can use this as a template for your projects which create Valida proofs of execution of Rust code.

## Installing the Valida toolchain

Using this template requires the Valida toolchain, version 0.6.0-alpha to be installed on your system. There are two ways to install this toolchain: via Docker, or via the Linux release bundle.

### Docker-based installation

#### x86_64-based platforms

To install and use the toolchain via Docker on a 64-bit computer with an Intel-compatible chipset (x86_64), such as Intel- or AMD-based computers:

```bash
# Download the container
docker pull ghcr.io/lita-xyz/llvm-valida-releases/valida-build-container:v0.7.0-alpha-amd64

# cd your-valida-project

# Enter the container:
docker run --platform linux/amd64 -it --rm -v $(realpath .):/src ghcr.io/lita-xyz/llvm-valida-releases/valida-build-container:v0.7.0-alpha-amd64

# You are now in a shell with the valida rust toolchain installed!
```

#### ARM64-based platforms

To install and use the toolchain via Docker on a 64-bit computer with an ARM64-compatible chipset (ARM64), such as Apple silicon-based computers:

```bash
# Download the container
docker pull ghcr.io/lita-xyz/llvm-valida-releases/valida-build-container:v0.7.0-alpha-arm64

# cd your-valida-project

# Enter the container:
docker run --platform linux/arm64 -it --rm -v $(realpath .):/src ghcr.io/lita-xyz/llvm-valida-releases/valida-build-container:v0.7.0-alpha-arm64

# You are now in a shell with the valida rust toolchain installed!
```

### Linux-based installation

#### System requirements

 * This toolchain supports x86-64 Linux based on `glibc-2.9` or newer `glibc`.
 * [`rustup`](https://www.rust-lang.org/tools/install) is required.
 * Arch Linux and Ubuntu 24.04 LTS are specifically supported, with other platforms possibly requiring some tinkering to make work.

#### Download

To download the Linux-based release bundle:

```
wget https://github.com/lita-xyz/llvm-valida-releases/releases/download/v0.6.0-alpha/llvm-valida-v0.6.0-alpha-linux-x86_64.tar.xz
```

#### Installation

From the untarred release bundle, in the directory called `valida-toolchain`, the same directory containing these release notes, run:

```bash
sudo ./install.sh
```

#### Entering the Valida shell

Upon having installed the toolchain, the Valida shell should be on your `PATH`, and if you run `which valida-shell`, you should see something like:

```bash
/home/morgan/.local/bin/valida-shell
```

If the result is very different from this, then either the installation did not complete successfully, or you had another executable named `valida-shell` somewhere on your `PATH`.

If you run `valida-shell`, then you should see a shell prompt that reads `valida> `. You should then have on your `PATH` all of the executables from the Valida toolchain needed to follow the usage instructions below.

## Usage

Build the project, from the root directory of this repo, in the Valida shell:

```bash
cargo +valida build
```

To run the program, in the Valida shell, from the root directory of this repo:

```bash
valida run ./target/valida-unknown-baremetal-gnu/debug/fibonacci log
```

The `run` command runs the program, prompting for an input, and print the output to the console and the file `log` in the current directory.

To run the program with a file input, you can use the following commands:

```bash
echo -ne 25 > 25.input
valida run ./target/valida-unknown-baremetal-gnu/debug/fibonacci log 25.input
```

The file 25.bin is a binary file containing the number 25. This is the input to the `fibonacci` program.

The `run` command will load the binary, and execute the program. The program will then run, and print the output to the console and the file `log` in the current directory.

The log file should contain:

```
The 25-th fibonacci number is: 75025
```

## Writing your own Valida project

We aim to make it so that your native Rust code works on Valida with minimal changes. There are constant improvements in this regard so please check this section for each new version of Valida.

### Changes required for your program to work on Valida and differences from native Rust

As of now, one can use all `stdio` printing functions normally, including `println!`, `eprintln!`, `print!`, and `eprint!`. However, because Valida only has one output tape which prints to `stdout`, all errors are also printed to `stdout`, as opposed to the usual case of `stderr`. Relatedly, [`panic!`](https://doc.rust-lang.org/std/macro.panic.html) is supported, and the error messages when panicking are printed to `stdout`(as opposed to `stderr`) as well.

For reading, one should use the `entrypoint::io` library. The standard read functions like `stdin().read_line(&mut line)` are not yet supported. Support will be added in future versions. See below for more details.

For projects with dependencies on randomness (`rand`), you will need to use the `main` branch of `valida-rs` and call the Valida version of the randomness functions. See below for more details.

### Using `valida-rs`

`valida-rs` is to be used as a dependency in a Valida project. It provides an IO library that works on Valida and the entry point for programs that require randomness.

#### The `IO` library

This library provides common IO functions that work on Valida. See [io.rs](https://github.com/lita-xyz/valida-rs/blob/main/src/io.rs) for the full list of available functions. To use this library, add `valida-rs` to your `Cargo.toml` as shown below. Then call the functions with the `valida_rs::io::` prefix.

To use our IO library functions, you will want to use the `no-deps` branch. Unless you need randomness and/or serialization/deserialization of data, in which case you should use the `main` branch. See below for more details.

To use the "no-deps" branch, add the following to your `Cargo.toml`:

```toml
[dependencies]
valida-rs = { git = "https://github.com/lita-xyz/valida-rs.git", branch = "no-deps" }
```

Also, in your `src/main.rs`, add the following:

```rust
#![no_main]

#[no_mangle]  
pub fn main() {
    // your code here
}
```

The `#[no_mangle]` attribute tells the compiler not to mangle (rename) the function name during compilation. We need this because the Valida runtime looks for a function specifically named "main".

#### For projects that require `rand`

Projects that require `rand` should use the `main` branch of `valida-rs` as a dependency. Add the following to your `Cargo.toml`:

```toml
[dependencies]
valida-rs = { git = "https://github.com/lita-xyz/valida-rs.git", branch = "main" }
getrandom = "0.2.15" # or the current version
rand = "0.8.5" # or the current version
```

Also, add the following to your `src/main.rs`:

```rust
#![no_main]

valida_rs::entrypoint!(main);
```

The `entrypoint!` macro:

- Sets up a deterministic random number generator: It ensures that when `rand` functions are called, they are fixed to a specified seed and thus are deterministic. This is required for the program to be provable.
- Creates a new entry point that wraps the user's main function: This is required because we need to make Rust call this `main` function, the standard Rust `main` function does not work on Valida.


The `#![no_main]` (with `!`) is an inner attribute that applies to the entire crate, telling the Rust compiler not to look for a standard main function entry point. We need this because we are providing a custom entry point.

## Issue reporting

Any issues report at https://github.com/lita-xyz/valida-releases/issues
