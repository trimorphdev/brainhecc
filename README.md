# Brainhecc
A compiler for Brain[hecc] programs, written in Rust with Cranelift.

It compiles any valid Brainhecc program into an executable binary.

## Installation

### Prerequisites
- Cargo/Rust, to install & build the Brainhecc program.
- GCC (or any other linker), required to link Brainheck programs.

> Note: to compile Brainhecc programs, any linker may be used, *but* Brainhecc programs must be linked with the C standard library, otherwise compiled programs won't work.

### Setting Up
Cargo can automatically download and build the Brainhecc program:

```
cargo install brainhecc
```

### Hello World
The Hello, World! example can be compiled an ran with the following commands:

```
brainhecc examples/hello_world.brainhecc hello_world.o # compile the program
gcc hello_world.o -o hello_world # link the program with the C standard library
./hello_world # run the program
              # => Hello, world!
```