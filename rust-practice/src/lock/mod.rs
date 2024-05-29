//!
//! 锁有两种，一种是同步锁，一种是异步锁。一般情况，同步锁比异步锁性能更好一些。
//! 同步锁建议用 parking_lot::Mutex；
//! 异步锁建议用 tokio::sync::Mutex;
//! 
//! 同步锁在 同步 和 异步 代码中都可以使用， 异步锁只能在异步代码上下文中使用。
//! 尽量避免同时拥有两把锁，如果两把锁上锁的顺序不一样就会死锁
//! 

use std::time::Duration;


#[derive(Debug, Default, Clone)]
pub struct Foo {
    shared: std::sync::Arc<Shared>, // 共享部分用Arc包起来
}

#[derive(Debug, Default)]
struct Shared {
    // 只读属性不需要用锁保护, 因为永远都不会去修改它
    id: String, 

    // 读写属性用Mutex保护
    sync_state: parking_lot::Mutex<SyncState>,    

    async_state: tokio::sync::Mutex<AsyncState>,
}

#[derive(Debug, Default)]
struct SyncState {
    num_sync_requests: u64,
}

#[derive(Debug, Default)]
struct AsyncState {
    num_async_requests: u64,
}


impl Foo {
    pub fn id(&self) -> &String {
        &self.shared.id
    }

    // 同步锁在同步代码中使用
    pub fn add_sync_request1(&self, delta: u64) -> u64 {
        // 用花括弧 {} 包起来使用锁是一个好习惯，明确了锁的作用范围
        let num = {
            // lock 返回的是一个 Guard 对象，在Guard的生命周期里，一直会持有锁
            let mut state = self.shared.sync_state.lock();

            // Guard 对象实现了 Deref 和 DerefMut，所以可以像使用原类型一样访问属性
            state.num_sync_requests += delta;

            state.num_sync_requests

        };

        num
    }

    // 同步锁在也可以在异步代码中使用
    pub async fn add_sync_request2(&self, delta: u64) -> u64 {

        let num = {

            let mut state = self.shared.sync_state.lock();

            println!("num  {}", state.num_sync_requests);

            // 在拿到同步锁期间，不能调用 await
            // tokio::time::sleep(Duration::from_millis(50)).await;

            state.num_sync_requests += delta;

            state.num_sync_requests
        };

        num
    }

    // 异步锁必须在异步上下文中使用
    pub async fn add_async_request(&self, delta: u64) -> u64 {

        let num = {

            let mut state = self.shared.async_state.lock().await;

            // 在拿到异步锁期间，可以调用 await
            tokio::time::sleep(Duration::from_millis(50)).await;

            state.num_async_requests += delta;

            state.num_async_requests
        };

        num
    }

    // 先锁 self，再锁 other
    pub fn lock_twice_1(&self, other: &Self) {
        {
            println!("lock_twice_1: try lock self");
            let mut self_state = self.shared.sync_state.lock();
    
            {
                println!("lock_twice_1: wait for 2 seconds");
                std::thread::sleep(Duration::from_millis(2000));
                
                println!("lock_twice_1: try lock other");
                let other_state = other.shared.sync_state.lock();

                
                println!("lock_twice_1: hold lock and add");
                self_state.num_sync_requests += other_state.num_sync_requests;
            }
            println!("lock_twice_1: unlock other");
            
        }
        println!("lock_twice_1: unlock self");
    }

    // 先锁 other，再锁 self
    pub fn lock_twice_2(&self, other: &Self) {

        {
            println!("lock_twice_2: try lock other");
            let mut other_state = other.shared.sync_state.lock();
    
            {
                println!("lock_twice_2: try lock self");
                let self_state = self.shared.sync_state.lock();

                
                println!("lock_twice_2: hold lock and add");
                other_state.num_sync_requests += self_state.num_sync_requests;
            }
            println!("lock_twice_2: unlock self");
            
        }
        println!("lock_twice_2: unlock other");

        
        
        
        
    }
}

#[tokio::test]
async fn test() {
    let foo = Foo::default();

    assert_eq!(foo.id(), "");
    assert_eq!(foo.add_sync_request1(3), 3);
    assert_eq!(foo.add_sync_request2(3).await, 6);
    assert_eq!(foo.add_async_request(3).await, 3);

    
}

// 当 self 和 other 为不同对象时，不会死锁
pub fn test_dead_lock0() {
    let foo1 = Foo::default();
    let foo2 = Foo::default();
    foo1.lock_twice_2(&foo2);
}


// 当 self 和 other 为同一个对象时，会死锁
pub fn test_dead_lock1() {
    let foo1 = Foo::default();
    foo1.lock_twice_2(&foo1);
}

// 两个锁的顺序不一样导致的死锁
pub fn test_dead_lock2() {
    
    let foo1 = Foo::default();
    let foo2 = Foo::default();

    let thread1 = {
        let foo1 = foo1.clone();
        let foo2 = foo2.clone();
        std::thread::spawn(move || {
            foo1.lock_twice_1(&foo2);
        })
    };

    let thread2 = {
        let foo1 = foo1.clone();
        let foo2 = foo2.clone();
        std::thread::spawn(move || {
            foo1.lock_twice_2(&foo2);
        })
    };

    thread1.join().unwrap();
    thread2.join().unwrap();
}