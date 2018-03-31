# Tiny Rocket

At the recent 2018 Rust All Hands, I met up with Katharina [@spacekookie], who works on an [open source project] that creates software for Embedded Linux Devices. She had talked with the other engineers on the project about including some Rust components, however with their limited flash storage space (8MB for the whole firmware, including operating system and all other software), she was worried that the Rust binaries wouldn't fit. The current webserver component for their project was measured in the 100's of KB, while the Rust binary she produced was already multiple MBs, even with a `--release` build!

[@spacekookie]: https://github.com/spacekookie
[open source project]: https://github.com/qaul/qaul.net

I had also done some work on Embedded Linux devices before for [my current employer], though the devices we were working on had 100's of MBs of flash, so size optimization hadn't been something that had been necessary yet. Luckily, I had some experience with tricks used for bare metal systems written [in Rust], so I offered to take a look at what we could do.

[my current employer]: https://github.com/geeny/linux-hub-sdk
[in Rust]: https://github.com/rust-lang-nursery/embedded-wg

The goal was to get the binary down under 1MB, and ideally under 500KB. Lets see where we are starting from:

## The Environment

All of these tests were performed on an Arch Linux machine, with the current (as of this writing) Nightly version of Rust. Some details:

```bash
$ uname -a
Linux archmbp13 4.15.9-1-ARCH #1 SMP PREEMPT Sun Mar 11 17:54:33 UTC 2018 x86_64 GNU/Linux
$ rustup show
Default host: x86_64-unknown-linux-gnu
...
active toolchain
----------------

nightly-x86_64-unknown-linux-gnu (default)
rustc 1.26.0-nightly (9cb18a92a 2018-03-02)
```

## The Base Case

Here is what our "Hello World" binary looks like (you can also find the code for this experiment in [spacekookie's repo]).

[spacekookie's repo]: https://github.com/spacekookie/tinyrocket

```rust
#![feature(plugin)]
#![plugin(rocket_codegen)]

extern crate rocket;

#[get("/")]
fn index() -> &'static str {
    "Hello, world!"
}

fn main() {
    rocket::ignite().mount("/", routes![index]).launch();
}
```

I started off by building `dev` and `release` and release builds of this project:

```bash
# dev build
$ cargo build
# ...
   Compiling tinyrocket v0.1.0 (file:///home/james/personal/tinyrocket)
    Finished dev [unoptimized + debuginfo] target(s) in 46.59 secs

# release build
$ cargo build --release
# ...
   Compiling tinyrocket v0.1.0 (file:///home/james/personal/tinyrocket)
    Finished release [optimized] target(s) in 106.88 secs
```

And these were the binary sizes we got:

```bash
$ ls -al target/debug/tinyrocket
-rwxr-xr-x 2 james users 22900656 Mar 31 15:10 target/debug/tinyrocket
$ ls -al target/release/tinyrocket
-rwxr-xr-x 2 james users 6706984 Mar 31 15:12 target/release/tinyrocket
```

### Current size status

| build   | modifications | size (bytes) | size (human) | % change |
| :----   | :------------ | :----------- | :----------- | :------- |
| dev     | none          | 22900656     | 22M          | 0%       |

| build   | modifications | size (bytes) | size (human) | % change |
| :----   | :------------ | :----------- | :----------- | :------- |
| release | none          | 6706984      | 6.4M         | 0%       |

Ouch. 22MB for the `dev` build, and 6.4MB for the `release` build. Those won't work for us!

## Stripping the Binary

By default, Rust and LLVM retain lots of information in the binary that is very useful for debugging. However, this information is not strictly necessary for running the program. `binutils` provides us with a binary called `strip`, which removes the information. Lets try that. At this stage, there is no modification to the Rust code or compiler settings, just adding a step to your build and release process.

```bash
$ strip target/debug/tinyrocket
$ strip target/release/tinyrocket
$ ls -al target/debug/tinyrocket
-rwxr-xr-x 2 james users 4022576 Mar 31 15:21 target/debug/tinyrocket
$ ls -al target/release/tinyrocket
-rwxr-xr-x 2 james users 1749216 Mar 31 15:21 target/release/tinyrocket
```

### Current size status

| build   | modifications | size (bytes) | size (human) | % change |
| :----   | :------------ | :----------- | :----------- | :------- |
| dev     | none          | 22900656     | 22M          | 0%       |
| dev     | stripped      | 4022576      | 3.9M         | -82.4%   |

| build   | modifications | size (bytes) | size (human) | % change |
| :----   | :------------ | :----------- | :----------- | :------- |
| release | none          | 6706984      | 6.4M         | 0%       |
| release | stripped      | 1749216      | 1.7M         | -73.9%   |

Not bad for a first step! These binaries will work pretty much the same as the original ones, though they would be harder to debug effectively. This is also often standard practice when releasing binaries.
