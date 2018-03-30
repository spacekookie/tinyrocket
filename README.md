# How to build a tiny binary

## x86-64

```bash
xargo build --target x86_64-unknown-linux-gnu --release --features system-alloc
strip target/x86_64-unknown-linux-gnu/release/tinyrocket
upx --brute target/x86_64-unknown-linux-gnu/release/tinyrocket
```

## armv7-unknown-linux-gnueabihf

```bash
cross build --target armv7-unknown-linux-gnueabihf --release --features system-alloc
arm-linux-eabihf-strip target/armv7-unknown-linux-gnueabihf/release/tinyrocket
upx --brute target/armv7-unknown-linux-gnueabihf/release/tinyrocket
```
