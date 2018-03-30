#![feature(plugin)]
#![plugin(rocket_codegen)]
#![feature(global_allocator)]
#![feature(allocator_api)]

// This section is necessary after rustc 1.20.x due to the new way
// allocator selection is handled. Either `jemalloc` may be used, or the system
// allocator (`malloc`) provided by glibc
#[cfg(feature = "system-alloc")]
mod allocator {
    use std::heap::System;

    #[global_allocator]
    pub static mut THE_ALLOC: System = System;
}

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
