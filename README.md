# NonBlocking Channel
[![crates.io](https://img.shields.io/crates/v/nonblocking-channel.svg)](https://crates.io/crates/nonblocking-channel)
[![docs.rs](https://img.shields.io/docsrs/nonblocking-channel)](https://docs.rs/nonblocking-channel/latest/nonblocking_channel/)
[![crates.io](https://img.shields.io/crates/l/nonblocking-channel.svg)](https://github.com/LucentFlux/nonblocking-channel/blob/main/LICENSE)

This crate provides a SPSC channel implementation, built on `ringbuf`, which is guaranteed to never block, even for a few cycles. It also allows the sender to be promoted to a MPSC sender, however the MPSC sender has the potential to block for very short periods.

This crate is intended for use specifically in a WebAssembly project to replace other channels on the main thread.

Current Rust channel implementations such as `std::sync::mpsc` or `flume` have methods such as `try_recv` which claim to be lockless. However they still have the potential to block for a handful of cycles, due to reallocation or other implementation details. This is not usually an issue, as an implementation with an upper bound on lock time is equivalent to a lock-free implementation, however running WebAssembly in a browser is slightly more strict - the main thread is not allowed to call `atomic.wait` *at all*, even if the wait would only be for a few cycles. 