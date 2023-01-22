#![allow(dead_code, unused_imports)]
mod mpu;
use mpu::*;

fn main() {
    test_madwick_mpu();
    test_raw_mpu();
}
