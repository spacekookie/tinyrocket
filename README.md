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

## Removing `jemalloc`

Also by default, Rust uses an allocator called [`jemalloc`], which tends to have better performance for many use cases. However, this is not a requirement to use, and for applications that are not required to be high-performance, or that don't make heavy use of dynamic memory allocation, the difference will be negligible.

[`jemalloc`]: http://jemalloc.net/

Since `jemalloc` is not provided by the system, and must instead be compiled and included in the Rust binary, it increases the total binary size. Lets see what happens when we tell the Rust compiler to instead make use of the existing system allocator, which is typically `malloc`. I will also be making the use of `jemalloc` optional using a configurable feature.

After modification, our `main.rs` now looks like this:

```rust
#![feature(plugin)]
#![plugin(rocket_codegen)]
#![feature(global_allocator)]
#![feature(allocator_api)]

// When the `system-alloc` feature is used, use the System Allocator
#[cfg(feature = "system-alloc")]
mod allocator {
    use std::heap::System;

    #[global_allocator]
    pub static mut THE_ALLOC: System = System;
}

// When the `system-alloc` feature is not used, do nothing,
// retaining the default functionality (using jemalloc)
#[cfg(not(feature = "system-alloc"))]
mod allocator {
    #[allow(dead_code)]
    pub static THE_ALLOC: () = ();
}

#[allow(unused_imports)]
use allocator::THE_ALLOC;

extern crate rocket;

#[get("/")]
fn index() -> &'static str {
    "Hello, world!"
}

fn main() {
    rocket::ignite().mount("/", routes![index]).launch();
}
```

We also had to add the following lines to our `Cargo.toml` in order to tell Cargo about the new feature we added:

```toml
[features]
system-alloc = []
```

With these changes made, I did a `cargo clean`, and some new `cargo build`s.

```bash
# dev build
$ cargo build --features system-alloc
# ...
   Compiling tinyrocket v0.1.0 (file:///home/james/personal/tinyrocket)
    Finished dev [unoptimized + debuginfo] target(s) in 47.23 secs

# release build
$ cargo build --features system-alloc --release
# ...
   Compiling tinyrocket v0.1.0 (file:///home/james/personal/tinyrocket)
    Finished release [optimized] target(s) in 106.73 secs
```

Our compile times didn't change much, lets see what kind of binary size we got:

```bash
$ ls -al target/debug/tinyrocket target/release/tinyrocket
-rwxr-xr-x 2 james users 20508800 Mar 31 15:49 target/debug/tinyrocket
-rwxr-xr-x 2 james users  4293464 Mar 31 15:50 target/release/tinyrocket
```

Not bad! But don't forget, we can stack these changes with `strip`!

```bash
➜  tinyrocket git:(with-docs) ✗ strip target/debug/tinyrocket
➜  tinyrocket git:(with-docs) ✗ strip target/release/tinyrocket
➜  tinyrocket git:(with-docs) ✗ ls -al target/debug/tinyrocket target/release/tinyrocket
-rwxr-xr-x 2 james users 3751920 Mar 31 15:53 target/debug/tinyrocket
-rwxr-xr-x 2 james users 1474464 Mar 31 15:53 target/release/tinyrocket
```

| build   | modifications | size (bytes) | size (human) | % change |
| :----   | :------------ | :----------- | :----------- | :------- |
| dev     | none          | 22900656     | 22M          | 0%       |
| dev     | stripped      | 4022576      | 3.9M         | -82.4%   |
| dev     | malloc        | 20508800     | 19.6         | -10.4%   |
| dev     | all above     | 3751920      | 3.6M         | -83.6%   |

| build   | modifications | size (bytes) | size (human) | % change |
| :----   | :------------ | :----------- | :----------- | :------- |
| release | none          | 6706984      | 6.4M         | 0%       |
| release | stripped      | 1749216      | 1.7M         | -73.9%   |
| release | malloc        | 4293464      | 4.1M         | -36.0%   |
| release | all above     | 1474464      | 1.5M         | -78.0%   |

We're getting closer to the 1MB threshold! But we can still do better...

## Panic Abort

By default, Rust also provides useful information when a panic occurs, or gives some ability to unwind a panic. These behaviors are useful, but also usefully optional! We can tell Cargo to just `abort` on a panic condition, which removes the need for any code that supports nicer panic behavior. We can disable this behavior for both debug and release builds by adding the following lines to our `Cargo.toml`:

```toml
[profile.release]
panic = "abort"

[profile.dev]
panic = "abort"
```

I reran the build, first with `jemalloc` still included:

```bash
# dev build
$ cargo build
# ...
   Compiling tinyrocket v0.1.0 (file:///home/james/personal/tinyrocket)
    Finished dev [unoptimized + debuginfo] target(s) in 46.41 secs

# release build
$ cargo build --release
# ...
   Compiling tinyrocket v0.1.0 (file:///home/james/personal/tinyrocket)
    Finished release [optimized] target(s) in 106.17 secs

$ ls -al target/debug/tinyrocket target/release/tinyrocket
-rwxr-xr-x 2 james users 22873512 Mar 31 16:05 target/debug/tinyrocket
-rwxr-xr-x 2 james users  6674328 Mar 31 16:06 target/release/tinyrocket
```

I then also reran the build with all of our current optimizations, including `strip`. Here are the results:

| build   | modifications | size (bytes) | size (human) | % change |
| :----   | :------------ | :----------- | :----------- | :------- |
| dev     | none          | 22900656     | 22M          | 0%       |
| dev     | stripped      | 4022576      | 3.9M         | -82.4%   |
| dev     | malloc        | 20508800     | 19.6         | -10.4%   |
| dev     | panic abort   | 22873512     | 21.8M        | -0.1%    |
| dev     | all above     | 3715056      | 3.6M         | -83.8%   |

| build   | modifications | size (bytes) | size (human) | % change |
| :----   | :------------ | :----------- | :----------- | :------- |
| release | none          | 6706984      | 6.4M         | 0%       |
| release | stripped      | 1749216      | 1.7M         | -73.9%   |
| release | malloc        | 4293464      | 4.1M         | -36.0%   |
| release | panic abort   | 6674328      | 6.4M         | -0.5%    |
| release | all above     | 1458080      | 1.4M         | -78.3%   |

> NOTE: Talk about more aggressive optimizations in the future that
> will make panic abort more helpful in size by ripping out format
> machinery and backtrace info

Okay, that one wasn't as impressive, but every little bit helps! What else can we try?

## Use LLVM's full LTO

Rust's compiler was designed to take full advantage of parallel building. This is great for compile times, however it comes at a cost of making it harder to perform total optimization of the binary. This behavior can be disabled, trading better optimization for increased compile times. We can enable these changes by changing the following in our `Cargo.toml`:

```toml
[profile.release]
panic = "abort"
lto = true
codegen-units = 1
incremental = false

[profile.dev]
panic = "abort"
lto = true
codegen-units = 1
incremental = false
```

For the initial test, I also disabled `panic = "abort"`, so the changes could be seen in isolation. `jemalloc` was also used for this build.

```bash
# dev build
$ cargo build
# ...
   Compiling tinyrocket v0.1.0 (file:///home/james/personal/tinyrocket)
    Finished dev [unoptimized + debuginfo] target(s) in 46.41 secs

# release build
$ cargo build --release
# ...
   Compiling tinyrocket v0.1.0 (file:///home/james/personal/tinyrocket)
    Finished release [optimized] target(s) in 106.17 secs

$ ls -al target/debug/tinyrocket target/release/tinyrocket
-rwxr-xr-x 2 james users 13628168 Mar 31 16:17 target/debug/tinyrocket
-rwxr-xr-x 2 james users  4885384 Mar 31 16:19 target/release/tinyrocket
```

As you can see, our binary decreased in size considerably, however our compile times have also increased. Lets reapply all of our optimizations, and see where we are so far.

| build   | modifications | size (bytes) | size (human) | % change |
| :----   | :------------ | :----------- | :----------- | :------- |
| dev     | none          | 22900656     | 22M          | 0%       |
| dev     | stripped      | 4022576      | 3.9M         | -82.4%   |
| dev     | malloc        | 20508800     | 19.6         | -10.4%   |
| dev     | panic abort   | 22873512     | 21.8M        | -0.1%    |
| dev     | No ThinLTO    | 13628168     | 13M          | -40.5%   |
| dev     | all above     | 3182496      | 3.1M         | -86.1%   |

| build   | modifications | size (bytes) | size (human) | % change |
| :----   | :------------ | :----------- | :----------- | :------- |
| release | none          | 6706984      | 6.4M         | 0%       |
| release | stripped      | 1749216      | 1.7M         | -73.9%   |
| release | malloc        | 4293464      | 4.1M         | -36.0%   |
| release | panic abort   | 6674328      | 6.4M         | -0.5%    |
| release | No ThinLTO    | 4885384      | 4.7M         | -27.2%   |
| release | all above     | 1228600      | 1.2M         | -81.7%   |

We are so close to that 1MB threshold, but there are still more optimizations to be had!