use core::cell::{RefCell, RefMut, UnsafeCell};
use core::ops::{Deref, DerefMut};
#[cfg(feature = "kernel")]
use riscv::register::sstatus;
use spin::Lazy;

/*
/// Wrap a static data structure inside it so that we are
/// able to access it without any `unsafe`.
///
/// We should only use it in uniprocessor.
///
/// In order to get mutable reference of inner data, call
/// `exclusive_access`.
pub struct UPSafeCell<T> {
    /// inner data
    inner: RefCell<T>,
}

unsafe impl<T> Sync for UPSafeCell<T> {}

impl<T> UPSafeCell<T> {
    /// User is responsible to guarantee that inner struct is only used in
    /// uniprocessor.
    pub unsafe fn new(value: T) -> Self {
        Self {
            inner: RefCell::new(value),
        }
    }
    /// Panic if the data has been borrowed.
    pub fn exclusive_access(&self) -> RefMut<'_, T> {
        self.inner.borrow_mut()
    }
}
*/

/// interior mutability
/// 内部可变性
pub struct UPSafeCellRaw<T> {
    inner: UnsafeCell<T>,
}

unsafe impl<T> Sync for UPSafeCellRaw<T> {}

impl<T> UPSafeCellRaw<T> {
    pub unsafe fn new(value: T) -> Self {
        Self {
            inner: UnsafeCell::new(value),
        }
    }
    pub fn get_mut(&self) -> &mut T {
        unsafe { &mut (*self.inner.get()) }
    }
}

/// 中断屏蔽信息
pub struct IntrMaskingInfo {
    nested_level: usize,
    sie_before_masking: bool,
}

pub static INTR_MASKING_INFO: Lazy<UPSafeCellRaw<IntrMaskingInfo>> =
    Lazy::new(|| unsafe { UPSafeCellRaw::new(IntrMaskingInfo::new()) });

impl IntrMaskingInfo {
    pub fn new() -> Self {
        Self {
            nested_level: 0,
            sie_before_masking: false,
        }
    }

    pub fn enter(&mut self) {
        #[cfg(feature = "kernel")]
        let sie = sstatus::read().sie();
        #[cfg(feature = "kernel")]
        unsafe {
            sstatus::clear_sie();
        }
        #[cfg(feature = "user")]
        let sie = true;
        if self.nested_level == 0 {
            self.sie_before_masking = sie;
        }
        self.nested_level += 1;
    }

    pub fn exit(&mut self) {
        self.nested_level -= 1;
        if self.nested_level == 0 && self.sie_before_masking {
            #[cfg(feature = "kernel")]
            unsafe {
                // 开启 SIE（不是 sie 寄存器），允许内核态被中断打断
                sstatus::set_sie();
            }
            // #[cfg(feature = "user")]
            // let _SIE = true;
        }
    }
}

/// A mutable memory location with dynamically checked borrow rules
//具有动态检查借用规则的可变内存位置
pub struct UPIntrFreeCell<T> {
    /// inner data
    inner: RefCell<T>,
}

unsafe impl<T> Sync for UPIntrFreeCell<T> {}

/// A wrapper type for a mutably borrowed value from a RefCell<T>
// 从 RefCell<T> 可变借用值的包装器类型
pub struct UPIntrRefMut<'a, T>(Option<RefMut<'a, T>>);
// 允许我们在 单核 上安全使用可变全局变量。
impl<T> UPIntrFreeCell<T> {
    ///
    pub unsafe fn new(value: T) -> Self {
        Self {
            inner: RefCell::new(value),
        }
    }

    /// Panic if the data has been borrowed.
    /// 能获取其内部对象的可变引用
    /// 如果程序运行中同时存在多个这样的引用，会触发 panic
    pub fn exclusive_access(&self) -> UPIntrRefMut<'_, T> {
        INTR_MASKING_INFO.get_mut().enter();
        UPIntrRefMut(Some(self.inner.borrow_mut()))
    }
    /// exclusive_session
    pub fn exclusive_session<F, V>(&self, f: F) -> V
    where
        F: FnOnce(&mut T) -> V,
    {
        let mut inner = self.exclusive_access();
        f(inner.deref_mut())
    }
}

impl<'a, T> Drop for UPIntrRefMut<'a, T> {
    fn drop(&mut self) {
        self.0 = None;
        INTR_MASKING_INFO.get_mut().exit();
    }
}

impl<'a, T> Deref for UPIntrRefMut<'a, T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        self.0.as_ref().unwrap().deref()
    }
}
impl<'a, T> DerefMut for UPIntrRefMut<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.0.as_mut().unwrap().deref_mut()
    }
}



#[test]
fn test_up() {
    
    let value = 1;
    unsafe{
        let upsafe1 = UPSafeCellRaw::new(value);
        let b = (& upsafe1).get_mut();
        assert_eq!(1,*b);
    }

    //判断enter()、exit()是否修改IntrMaskingInfo的成员值
    let mut intr1 = IntrMaskingInfo::new();
    assert_eq!(intr1.nested_level, 0); 
    assert_eq!(intr1.sie_before_masking, false);
    (&mut intr1).enter();
    assert_eq!(intr1.nested_level, 1); 
    assert_eq!(intr1.sie_before_masking, true);
    (&mut intr1).exit();
    assert_eq!(intr1.nested_level, 0); 

    let value2 = 2;
    let upintr1 = unsafe{
            UPIntrFreeCell::new(value2)
    };
    (& upintr1).exclusive_access();

}