# `ThreadX example Rust application`

The application follows a structure defined at https://ferrous-systems.com/blog/test-embedded-app/. This organizes
the project into separately testable parts. Read the blog to learn how to run the tests.

## Dependencies

#### 1. `flip-link`:

```console
$ cargo install flip-link
```

#### 2. `probe-rs`:

``` console
$ # make sure to install v0.2.0 or later
$ cargo install probe-rs --features cli
```

#### 3. `Rust target`:

This project is currently set up for the STM32F103.

``` console
$ rustup target add thumbv7em-none-eabihf
```

## Running

Go to the threadx-app/cross folder and run

```console
cargo run --bin hello --release
```

The code assumes that you will be using an ST-Link debugger. 
