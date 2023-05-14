use super::UPIntrFreeCell;
use alloc::collections::VecDeque;
use rcore_task_manage::ThreadId;

/// Semaphore信号
pub struct Semaphore {
    /// UPIntrFreeCell<SemaphoreInner>
    pub inner: UPIntrFreeCell<SemaphoreInner>,
}

/// SemaphoreInner
pub struct SemaphoreInner {
    pub count: isize,
    pub wait_queue: VecDeque<ThreadId>,
}

impl Semaphore {
    /// new
    pub fn new(res_count: usize) -> Self {
        Self {
            inner: unsafe {
                UPIntrFreeCell::new(SemaphoreInner {
                    count: res_count as isize,
                    wait_queue: VecDeque::new(),
                })
            },
        }
    }
    /// 当前线程释放信号量表示的一个资源，并唤醒一个阻塞的线程
    pub fn up(&self) -> Option<ThreadId> {
        let mut inner = self.inner.exclusive_access();
        inner.count += 1;
        inner.wait_queue.pop_front()
    }
    /// 当前线程试图获取信号量表示的资源，并返回结果
    pub fn down(&self, tid: ThreadId) -> bool {
        let mut inner = self.inner.exclusive_access();
        inner.count -= 1;
        if inner.count < 0 {
            inner.wait_queue.push_back(tid);
            drop(inner);
            false
        } else {
            true
        }
    }
}



#[test]
fn test_semaphore() {
    //代表线程
    let tid1 = ThreadId::from_usize(1);
    let tid2 = ThreadId::from_usize(2);
    let tid3 = ThreadId::from_usize(3);
    let tid4 = ThreadId::from_usize(4);

    //信号量表示的资源的数量
    let res_count = 1;
    let semaphore1 = Semaphore::new(res_count);
    //当前线程释放信号资源，资源+1,无等待线程
    let up1 = (& semaphore1).up();
    assert_eq!(up1, None);
    //线程1、2获得资源，线程3、4进入等待队列
    let down1 = (& semaphore1).down(tid1);
    assert_eq!(true, down1);
    let down2 = (& semaphore1).down(tid2);
    assert_eq!(true, down2);
    let down3 = (& semaphore1).down(tid3);
    assert_eq!(false, down3);
    let down4 = (& semaphore1).down(tid4);
    assert_eq!(false, down4);
    //释放资源，唤醒线程3、4
    let up2 = (& semaphore1).up();
    assert_eq!(up2, Some(tid3));
    let up3 = (& semaphore1).up();
    assert_eq!(up3, Some(tid4));

    
}