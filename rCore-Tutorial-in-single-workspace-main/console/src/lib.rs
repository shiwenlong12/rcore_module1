//! 提供可定制实现的 `print!`、`println!` 和 `log::Log`。
#![no_std]
#![deny(warnings, missing_docs)]

// #![feature(custom_test_frameworks)]
// #![test_runner(crate::test_runner)]

use core::{
    fmt::{self, Write},
    str::FromStr,
};
use spin::Once;

/// 向用户提供 `log`。
pub extern crate log;

/// 这个接口定义了向控制台“输出”这件事。
pub trait Console: Sync {
    /// 向控制台放置一个字符。
    fn put_char(&self, c: u8);

    /// 向控制台放置一个字符串。
    ///
    /// 如果使用了锁，覆盖这个实现以免反复获取和释放锁。
    #[inline]
    fn put_str(&self, s: &str) {
        for c in s.bytes() {
            self.put_char(c);
        }
    }
}

/// 库找到输出的方法：保存一个对象引用，这是一种单例。
static CONSOLE: Once<&'static dyn Console> = Once::new();

/// 用户调用这个函数设置输出的方法。
pub fn init_console(console: &'static dyn Console) {
    CONSOLE.call_once(|| console);
    log::set_logger(&Logger).unwrap();
}

/// 根据环境变量设置日志级别。
pub fn set_log_level(env: Option<&str>) {
    use log::LevelFilter as Lv;
    log::set_max_level(env.and_then(|s| Lv::from_str(s).ok()).unwrap_or(Lv::Trace));
}

/// 打印一些测试信息。
pub fn test_log() {
    println!(
        r"
   ______                       __
  / ____/___  ____  _________  / /__
 / /   / __ \/ __ \/ ___/ __ \/ / _ \
/ /___/ /_/ / / / (__  ) /_/ / /  __/
\____/\____/_/ /_/____/\____/_/\___/
===================================="
    );
    log::trace!("LOG TEST >> Hello, world!");
    log::debug!("LOG TEST >> Hello, world!");
    log::info!("LOG TEST >> Hello, world!");
    log::warn!("LOG TEST >> Hello, world!");
    log::error!("LOG TEST >> Hello, world!");
    println!("test_hello_world is OK");
    println!();
}

/// 打印。
///
/// 给宏用的，用户不会直接调它。
#[doc(hidden)]
#[inline]
pub fn _print(args: fmt::Arguments) {
    Logger.write_fmt(args).unwrap();
}

/// 格式化打印。
#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => {
        $crate::_print(core::format_args!($($arg)*));
    }
}

/// 格式化打印并换行。
#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => {{
        $crate::_print(core::format_args!($($arg)*));
        $crate::println!();
    }}
}

/// 这个 Unit struct 是 `core::fmt` 要求的。
struct Logger;

/// 实现 [`Write`] trait，格式化的基础。
impl Write for Logger {
    #[inline]
    fn write_str(&mut self, s: &str) -> Result<(), fmt::Error> {
        let _ = CONSOLE.get().unwrap().put_str(s);
        Ok(())
    }
}

/// 实现 `log::Log` trait，提供分级日志。
impl log::Log for Logger {
    #[inline]
    fn enabled(&self, _metadata: &log::Metadata) -> bool {
        true
    }

    #[inline]
    fn log(&self, record: &log::Record) {
        use log::Level::*;
        let color_code: u8 = match record.level() {
            Error => 31,
            Warn => 93,
            Info => 34,
            Debug => 32,
            Trace => 90,
        };
        println!(
            "\x1b[{color_code}m[{:>5}] {}\x1b[0m",
            record.level(),
            record.args(),
        );
    }

    fn flush(&self) {}
}

//单元测试主要测试私有接口
# [cfg(test)]
mod tests{
    use crate::Console;
    use crate::init_console;
    use crate::test_log;
    use crate::set_log_level;
    //use sbi_rt;
    //use core::arch::asm;

    const SBI_CONSOLE_PUTCHAR: usize = 1;
    //which 表示请求 RustSBI 的服务的类型（RustSBI 可以提供多种不同类型的服务），
    // arg0 ~ arg2 表示传递给 RustSBI 的 3 个参数
    #[inline(always)]
    fn sbi_call(which: usize, arg0: usize, arg1: usize, arg2: usize) -> usize {
        let mut ret;
        //Rust 编译器无法判定汇编代码的安全性，所以我们需要将其包裹在 unsafe 块中。 
        // unsafe {
        //     //使用 Rust 提供的 asm! 宏在代码中内嵌汇编。
        //     //x10~x17: 对应 a0~a7 x1 ：对应 ra 所以是以寄存器 a0~a2 来保存系统调用的参数，
        //     //以及寄存器 a7 保存 syscall ID， 返回值通过寄存器 a0 传递给局部变量 ret
        //     core::arch::asm!(
        //         "li x16, 0",
        //         "ecall",
        //         inlateout("x10") arg0 => ret,
        //         in("x11") arg1,
        //         in("x12") arg2,
        //         in("x17") which,
        //     );
        // }
        unsafe {
            core::arch::asm!(
                "ecall",
                inlateout("x10") arg0 => ret,
                in("x11") arg1,
                in("x12") arg2,
                in("x17") which,
            );
        }

        ret
    }

    pub fn console_putchar(ch: usize) {
        sbi_call(SBI_CONSOLE_PUTCHAR, ch, 0, 0);
    }


    struct Console1;

    /// 为 `Console` 实现 `console::Console` trait。
    impl Console for Console1 {

        fn put_char(&self, _c: u8) {
            // #[allow(deprecated)]
            // legacy::console_putchar(c as _);
            console_putchar(_c as u8 as usize);
        }
    }

    #[test]
    fn test_println() {
        init_console(&Console1);
        (&Console1).put_char(0);
        set_log_level(option_env!("LOG"));
        // 测试各种打印
        test_log();
        print!("hell0 ");
        //print!("trivial assertion... ");
        assert_eq!(1, 1);
        //println!("[ok]");
    }
}

