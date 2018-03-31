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
