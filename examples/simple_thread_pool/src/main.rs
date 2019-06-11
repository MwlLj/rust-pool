extern crate rust_pool;

use std::thread;
use std::time::Duration;
use rust_pool::thread::simple;

fn main() {
    let pool = simple::CThreadPool::new(10);
    pool.execute(|| {
        println!("thread ...");
    });
    std::thread::spawn(|| {
        println!("std thread ...");
    });
    std::thread::sleep(Duration::from_secs(1));
}
