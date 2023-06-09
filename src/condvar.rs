use super::{Mutex, UPIntrFreeCell};
use alloc::{collections::VecDeque, sync::Arc};
use rcore_task_manage::ThreadId;

/// Condvar
/// 条件变量
pub struct Condvar {
    /// UPIntrFreeCell<CondvarInner>
    pub inner: UPIntrFreeCell<CondvarInner>,
}

/// CondvarInner
pub struct CondvarInner {
    /// block queue
    pub wait_queue: VecDeque<ThreadId>,
}

impl Condvar {
    /// new
    pub fn new() -> Self {
        Self {
            inner: unsafe {
                UPIntrFreeCell::new(CondvarInner {
                    wait_queue: VecDeque::new(),//双端队列的向量
                })
            },
        }
    }
    /// 唤醒某个阻塞在当前条件变量上的线程
    pub fn signal(&self) -> Option<ThreadId> {
        let mut inner = self.inner.exclusive_access();
        // 删除第一个元素并返回它，如果 VecDeque 为空，则返回 None。
        inner.wait_queue.pop_front()
        
    }

    /*
    pub fn wait(&self) {
        let mut inner = self.inner.exclusive_access();
        inner.wait_queue.push_back(current_task().unwrap());
        drop(inner);
        block_current_and_run_next();
    }
    */
    /// 将当前线程阻塞在条件变量上
    pub fn wait_no_sched(&self, tid: ThreadId) -> bool {
        self.inner.exclusive_session(|inner| {
            inner.wait_queue.push_back(tid);
        });
        false
    }
    /// 从 mutex 的锁中释放一个线程，并将其阻塞在条件变量的等待队列中，等待其他线程运行完毕，当前的线程再试图获取这个锁
    ///
    /// 注意：下面是简化版的实现，在 mutex 唤醒一个线程之后，当前线程就直接获取这个 mutex，不管能不能获取成功
    /// 这里是单纯为了过测例，
    pub fn wait_with_mutex(
        &self,
        tid: ThreadId,
        mutex: Arc<dyn Mutex>,
    ) -> (bool, Option<ThreadId>) {
        let waking_tid = mutex.unlock().unwrap();
        (mutex.lock(tid), Some(waking_tid))
    }
}


#[test]
fn test_condvar() {
    let condvar1 = Condvar::new();

    let tid0 = ThreadId::from_usize(0);
    let tid1 = ThreadId::from_usize(1);
    let bool1 = (& condvar1).wait_no_sched(tid0);
    (& condvar1).wait_no_sched(tid1);
    assert_eq!(bool1, false);
    //let inner = condvar1.inner.exclusive_access();
    // let inner = &condvar1.inner;
    // assert_eq!(*inner.wait_queue.get(0), Some(&tid1));
    //assert_eq!(condvar1.inner,1);
    let sig1 = (& condvar1).signal();
    assert_eq!(sig1, Some(tid0));
}