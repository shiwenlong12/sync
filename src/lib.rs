//! 同步互斥模块
//Sync类型是可以安全的在线程间共享不可变引用，那么可以标记为Sync 
//实现Send的类型可以在线程间安全的传递其所有权
#![no_std]
//#![deny(warnings, missing_docs)]

mod condvar;
mod mutex;
mod semaphore;
mod up;

extern crate alloc;

pub use condvar::Condvar;
pub use mutex::{Mutex, MutexBlocking};
pub use semaphore::Semaphore;
pub use up::{UPIntrFreeCell, UPIntrRefMut};

pub use up::{UPSafeCellRaw, IntrMaskingInfo};



use rcore_task_manage::ThreadId;
use alloc::{sync::Arc};
#[test]
fn test_condvar1() {
    let condvar1 = Condvar::new();
    let mutex1 = MutexBlocking::new();

    let tid0 = ThreadId::from_usize(0);
    let tid1 = ThreadId::from_usize(1);
    let tid2 = ThreadId::from_usize(2);
    //获取锁，并改变locked的值
    let lock2 = (& mutex1).lock(tid0);
    assert_eq!(lock2, true);
    //获取锁，并向等待队列添加线程1、3
    let lock3 = (& mutex1).lock(tid1);
    assert_eq!(lock3, false);
    let lock4 = (& mutex1).lock(tid2);
    assert_eq!(lock4, false);
    //从 mutex 的锁中释放一个线程，并将其阻塞在条件变量的等待队列中，
    //等待其他线程运行完毕，当前的线程tid0再试图获取这个锁
    let arc = Arc::new(mutex1);
    let option1 = condvar1.wait_with_mutex(tid0,arc);
    assert_eq!(option1, (false,Some(tid1)));
}