use std::num::NonZeroUsize;

use nonblocking_channel::{RecvResult, SendResult};
use wasm_bindgen_test::wasm_bindgen_test;

fn check_reaches_capacity(capacity: usize) {
    let (mut sender, mut receiver) =
        nonblocking_channel::nonblocking_channel(NonZeroUsize::new(capacity).unwrap());

    for i in 0..capacity {
        sender.try_send(i).unwrap()
    }

    assert_eq!(sender.try_send(capacity), SendResult::Full(capacity));

    for i in 0..capacity {
        assert_eq!(receiver.try_recv(), RecvResult::Ok(i));
    }
    assert_eq!(receiver.try_recv(), RecvResult::Empty);
}

#[test]
fn native_check_reaches_capacity_1() {
    check_reaches_capacity(1)
}
#[test]
fn native_check_reaches_capacity_16() {
    check_reaches_capacity(16)
}
#[test]
fn native_check_reaches_capacity_127() {
    check_reaches_capacity(127)
}
#[test]
fn native_check_reaches_capacity_128() {
    check_reaches_capacity(128)
}
#[test]
fn native_check_reaches_capacity_1025() {
    check_reaches_capacity(1025)
}

#[wasm_bindgen_test]
fn web_check_reaches_capacity_1() {
    check_reaches_capacity(1)
}
#[wasm_bindgen_test]
fn web_check_reaches_capacity_16() {
    check_reaches_capacity(16)
}
#[wasm_bindgen_test]
fn web_check_reaches_capacity_127() {
    check_reaches_capacity(127)
}
#[wasm_bindgen_test]
fn web_check_reaches_capacity_128() {
    check_reaches_capacity(128)
}
#[wasm_bindgen_test]
fn web_check_reaches_capacity_1025() {
    check_reaches_capacity(1025)
}
