use core::time;
use std::{
    collections::HashMap,
    sync::{Arc, Condvar, Mutex, RwLock},
    thread,
    time::Duration,
};

use alloy::primitives::bytes::buf;

fn mutex_example() {
    let mut counter = Arc::new(Mutex::new(0));

    let mut handles = vec![];

    for _ in 0..5 {
        let counter = Arc::clone(&counter);
        let handle = thread::spawn(move || {
            for j in 0..1000 {
                let mut count = counter.lock().unwrap();

                *count += 1;
            }
        });

        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }

    println!("Final count: {}", *counter.lock().unwrap());
}

fn rwlock_example() {
    println!("   创建一个配置映射，多个读者，少数写者...");

    let config = Arc::new(RwLock::new(HashMap::new()));

    // 初始化配置
    {
        let mut map = config.write().unwrap();
        map.insert("server_port".to_string(), "8080".to_string());
        map.insert("max_connections".to_string(), "1000".to_string());
        map.insert("timeout".to_string(), "30".to_string());
    }

    let mut handles = vec![];

    for _ in 0..3 {
        let config = Arc::clone(&config);
        let handle = thread::spawn(move || {
            for _ in 0..3 {
                let map = config.read().unwrap();
                for (key, value) in map.iter() {
                    println!(" 读取配置: {}: {}", key, value)
                }
                thread::sleep(Duration::from_millis(100));
            }
        });

        handles.push(handle);
    }

    let write_config = Arc::clone(&config);
    let writer_handler = thread::spawn(move || {
        thread::sleep(Duration::from_millis(200));
        let mut map = write_config.write().unwrap();
        println!("   ✏️  写者线程正在更新配置...");
        map.insert("server_port".to_string(), "8081".to_string());
        map.insert("max_connections".to_string(), "2000".to_string());
        map.insert("timeout".to_string(), "60".to_string());
        println!("   ✅ 配置更新完成");
    });
    handles.push(writer_handler);

    for handle in handles {
        handle.join().unwrap();
    }
}

fn producer_consumer_example() {
    let buffer = Arc::new(Mutex::new(Vec::new()));
    let not_empty = Arc::new(Condvar::new());
    let not_full = Arc::new(Condvar::new());

    const BUFFER_SIZE: usize = 5;

    let mut handles = vec![];

    for i in 0..2 {
        let buffer = Arc::clone(&buffer);
        let not_empty = Arc::clone(&not_empty);
        let not_full = Arc::clone(&not_full);

        let handle = thread::spawn(move || {
            for j in 0..5 {
                let mut buf = buffer.lock().unwrap();
                while buf.len() >= BUFFER_SIZE {
                    println!("   🔴 生产者 {} 等待：缓冲区已满", i);
                    buf = not_full.wait(buf).unwrap();
                }

                let item = format!("P{}-Item{}", i, j);
                buf.push(item.clone());

                println!("生产者 {} 生产了: {} (缓冲区大小: {})", i, item, buf.len());

                not_empty.notify_one();
                thread::sleep(Duration::from_millis(300));
            }
        });
        handles.push(handle);
    }

    for i in 0..2 {
        let buffer = Arc::clone(&buffer);
        let not_empty = Arc::clone(&not_empty);
        let not_full = Arc::clone(&not_full);
        let handle = thread::spawn(move || {
            for _ in 0..5 {
                let mut buf = buffer.lock().unwrap();

                while buf.is_empty() {
                    println!("   🔵 消费者 {} 等待：缓冲区为空", i);
                    buf = not_empty.wait(buf).unwrap();
                }

                let item = buf.remove(0);
                println!("消费者 {} 消费了: {} (缓冲区大小: {})", i, item, buf.len());

                not_full.notify_one();
                thread::sleep(Duration::from_millis(400));
            }
        });
        handles.push(handle);
    }

    // 等待所有线程完成
    for handle in handles {
        handle.join().unwrap();
    }

    println!("   🎯 生产者-消费者示例完成");

    // 显示最终缓冲区状态
    let final_buffer = buffer.lock().unwrap();
    println!("   📊 最终缓冲区状态: {:?}", *final_buffer);
}

fn main() {
    println!("\n📌 1. Mutex (互斥锁) 示例");
    mutex_example();

    println!("\n📌 2. RwLock (读写锁) 示例");
    rwlock_example();

    println!("\n📌 7. 综合示例 - 生产者消费者模式");
    producer_consumer_example();
}
