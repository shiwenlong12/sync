
use sync::{up::{UPSafeCellRaw, IntrMaskingInfo, UPIntrFreeCell},
            condvar::{Condvar},
            mutex::{MutexBlocking, Mutex},
            semaphore::{Semaphore}
    };
use rcore_task_manage::ThreadId;



#[test]
fn test_up() {
    
    let value = 1;
    unsafe{
        let upsafe1 = UPSafeCellRaw::new(value);
        let b = (& upsafe1).get_mut();
        assert_eq!(1,*b);
    }

    let mut intr1 = IntrMaskingInfo::new();
    (&mut intr1).enter();
    (&mut intr1).exit();

    let value2 = 2;
    let upintr1 = unsafe{
            UPIntrFreeCell::new(value2)
    };
    (& upintr1).exclusive_access();

}

#[test]
fn test_condvar() {
    let condvar1 = Condvar::new();
    let tid1 = ThreadId::new();
    let tid2 = ThreadId::from_usize(0);
    assert_eq!(tid1, tid2);
    (& condvar1).wait_no_sched(tid2);
    (& condvar1).signal();
}

#[test]
fn test_mutex() {
    let mutex1 = MutexBlocking::new();
    let tid2 = ThreadId::from_usize(0);
    let lock1 = (& mutex1).lock(tid2);
    assert_eq!(true, lock1);
    (& mutex1).unlock();
}

#[test]
fn test_semaphore() {
    let res_count = 1;
    let semaphore1 = Semaphore::new(res_count);
    (& semaphore1).up();
    let tid2 = ThreadId::from_usize(2);
    let down1 = (& semaphore1).down(tid2);
    assert_eq!(true, down1);
}
