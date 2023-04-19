//! 内核上下文控制。

#![no_std]
#![feature(naked_functions, asm_const)]
#![deny(warnings, missing_docs)]

/// 不同地址空间的上下文控制。
#[cfg(feature = "foreign")]
pub mod foreign;

/// 线程上下文。
#[derive(Clone)]
#[repr(C)]
pub struct LocalContext {
    sctx: usize,
    x: [usize; 31],
    sepc: usize,
    /// 是否以特权态切换。
    pub supervisor: bool,
    /// 线程中断是否开启。
    pub interrupt: bool,
}

impl LocalContext {
    /// 创建空白上下文。
    #[inline]
    pub const fn empty() -> Self {
        Self {
            sctx: 0,
            x: [0; 31],
            supervisor: false,
            interrupt: false,
            sepc: 0,
        }
    }

    /// 初始化指定入口的用户上下文。
    ///
    /// 切换到用户态时会打开内核中断。
    #[inline]
    pub const fn user(pc: usize) -> Self {
        Self {
            sctx: 0,
            x: [0; 31],
            supervisor: false,
            interrupt: true,
            sepc: pc,
        }
    }

    /// 初始化指定入口的内核上下文。
    #[inline]
    pub const fn thread(pc: usize, interrupt: bool) -> Self {
        Self {
            sctx: 0,
            x: [0; 31],
            supervisor: true,
            interrupt,
            sepc: pc,
        }
    }

    /// 读取用户通用寄存器。
    #[inline]
    pub fn x(&self, n: usize) -> usize {
        self.x[n - 1]
    }

    /// 修改用户通用寄存器。
    #[inline]
    pub fn x_mut(&mut self, n: usize) -> &mut usize {
        &mut self.x[n - 1]
    }

    /// 读取用户参数寄存器。
    #[inline]
    pub fn a(&self, n: usize) -> usize {
        self.x(n + 10)
    }

    /// 修改用户参数寄存器。
    #[inline]
    pub fn a_mut(&mut self, n: usize) -> &mut usize {
        self.x_mut(n + 10)
    }

    /// 读取用户栈指针。
    #[inline]
    pub fn ra(&self) -> usize {
        self.x(1)
    }

    /// 读取用户栈指针。
    #[inline]
    pub fn sp(&self) -> usize {
        self.x(2)
    }

    /// 修改用户栈指针。
    #[inline]
    pub fn sp_mut(&mut self) -> &mut usize {
        self.x_mut(2)
    }

    /// 当前上下文的 pc。
    #[inline]
    pub fn pc(&self) -> usize {
        self.sepc
    }

    /// 修改上下文的 pc。
    #[inline]
    pub fn pc_mut(&mut self) -> &mut usize {
        &mut self.sepc
    }

    /// 将 pc 移至下一条指令。
    ///
    /// # Notice
    ///
    /// 假设这一条指令不是压缩版本。
    #[inline]
    pub fn move_next(&mut self) {
        self.sepc = self.sepc.wrapping_add(4);
    }

    /// 执行此线程，并返回 `sstatus`。
    ///
    /// # Safety
    ///
    /// 将修改 `sscratch`、`sepc`、`sstatus` 和 `stvec`。
    #[inline]
    pub unsafe fn execute(&mut self) -> usize {
        let mut sstatus = build_sstatus(self.supervisor, self.interrupt);
        core::arch::asm!(
            "   csrrw {sscratch}, sscratch, {sscratch}
                csrw  sepc    , {sepc}
                csrw  sstatus , {sstatus}
                addi  sp, sp, -8
                sd    ra, (sp)
                call  {execute_naked}
                ld    ra, (sp)
                addi  sp, sp,  8
                csrw  sscratch, {sscratch}
                csrr  {sepc}   , sepc
                csrr  {sstatus}, sstatus
            ",
            sscratch      = in       (reg) self,
            sepc          = inlateout(reg) self.sepc,
            sstatus       = inlateout(reg) sstatus,
            execute_naked = sym execute_naked,
        );
        sstatus
    }
}

#[inline]
fn build_sstatus(supervisor: bool, interrupt: bool) -> usize {
    let mut sstatus: usize;
    // 只是读 sstatus，安全的
    unsafe { core::arch::asm!("csrr {}, sstatus", out(reg) sstatus) };
    const PREVILEGE_BIT: usize = 1 << 8;
    const INTERRUPT_BIT: usize = 1 << 5;
    match supervisor {
        false => sstatus &= !PREVILEGE_BIT,
        true => sstatus |= PREVILEGE_BIT,
    }
    match interrupt {
        false => sstatus &= !INTERRUPT_BIT,
        true => sstatus |= INTERRUPT_BIT,
    }
    sstatus
}

/// 线程切换核心部分。
///
/// 通用寄存器压栈，然后从预存在 `sscratch` 里的上下文指针恢复线程通用寄存器。
///
/// # Safety
///
/// 裸函数。
#[naked]
unsafe extern "C" fn execute_naked() {
    core::arch::asm!(
        r"  .altmacro
            .macro SAVE n
                sd x\n, \n*8(sp)
            .endm
            .macro SAVE_ALL
                sd x1, 1*8(sp)
                .set n, 3
                .rept 29
                    SAVE %n
                    .set n, n+1
                .endr
            .endm

            .macro LOAD n
                ld x\n, \n*8(sp)
            .endm
            .macro LOAD_ALL
                ld x1, 1*8(sp)
                .set n, 3
                .rept 29
                    LOAD %n
                    .set n, n+1
                .endr
            .endm
        ",
        // 位置无关加载
        "   .option push
            .option nopic
        ",
        // 保存调度上下文
        "   addi sp, sp, -32*8
            SAVE_ALL
        ",
        // 设置陷入入口
        "   la   t0, 1f
            csrw stvec, t0
        ",
        // 保存调度上下文地址并切换上下文
        "   csrr t0, sscratch
            sd   sp, (t0)
            mv   sp, t0
        ",
        // 恢复线程上下文
        "   LOAD_ALL
            ld   sp, 2*8(sp)
        ",
        // 执行线程
        "   sret",
        // 陷入
        "   .align 2",
        // 切换上下文
        "1: csrrw sp, sscratch, sp",
        // 保存线程上下文
        "   SAVE_ALL
            csrrw t0, sscratch, sp
            sd    t0, 2*8(sp)
        ",
        // 切换上下文
        "   ld sp, (sp)",
        // 恢复调度上下文
        "   LOAD_ALL
            addi sp, sp, 32*8
        ",
        // 返回调度
        "   ret",
        "   .option pop",
        options(noreturn)
    )
}



# [cfg(test)]
mod tests{
    use crate::LocalContext;
    //use crate::build_sstatus;
    //use crate::foreign::ForeignContext;
    //use crate::foreign::MonoForeignPortal;

    #[test]
    fn test_context() {
        //测试empty()函数
        let mut _a = LocalContext::empty();
        assert_eq!(false,_a.supervisor);
        assert_eq!(false,_a.interrupt);
        assert_eq!(0,_a.pc());
        //测试user()函数,初始化指定入口的用户上下文。
        _a = LocalContext::user(04);
        assert_eq!(false,_a.supervisor);
        assert_eq!(true,_a.interrupt);
        assert_eq!(04,_a.pc());
        //thread()函数,初始化指定入口的内核上下文。
        _a = LocalContext::thread(04, false);
        assert_eq!(true,_a.supervisor);
        assert_eq!(false,_a.interrupt);
        assert_eq!(04,_a.pc());
        _a = LocalContext::thread(04, true);
        assert_eq!(true,_a.supervisor);
        assert_eq!(true,_a.interrupt);
        assert_eq!(04,_a.pc());

        //测试读取类函数
        let _x =  _a.x(1);
        assert_eq!(0,_a.x(1));
        assert_eq!(0,_a.a(1));
        assert_eq!(0,_a.ra());
        assert_eq!(0,_a.sp());
        assert_eq!(04,_a.pc());

        //测试move_next,将 pc 移至下一条指令。
        _a.move_next();
        assert_eq!(08,_a.pc());

        //测试修改类函数
        let mut _b = LocalContext {
            sctx: 0,
            x: [0,1,2,3,4,5,6,7,8,9,10,11,12,13,14,15,16,17,18,19,20,21,22,23,24,25,26,27,28,29,30],
            supervisor: false,
            interrupt: false,
            sepc: 0,
        };
        (&mut _b).x_mut(1);

        assert_eq!(0,_b.x(1));
        assert_eq!(0,*(&mut _b).x_mut(1));
        assert_eq!(10,*(&mut _b).a_mut(1));
        assert_eq!(1,*(&mut _b).sp_mut());
        assert_eq!(0,*(&mut _b).pc_mut());

    }

    use crate::build_sstatus;
    #[test]
    fn test_build_sstatus() {
        //build_sstatus(false,false);
        //build_sstatus(false,true);
    }

    //use crate::foreign::{PortalCache, ForeignContext};
    #[test]
    fn test_foreign() {
        // let mut _a = LocalContext::empty();
        // let mut _b = ForeignContext {
        //     /// 目标地址空间上的线程上下文。
        //     context: _a,
        //     /// 目标地址空间。
        //     satp: 0,
        // };
        
        // execute(&mut _b,);
    }

    #[test]
    fn test_multislot_portal() {
        //计算包括 `slots` 个插槽的传送门总长度。
        //calculate_size(1);
    }

}