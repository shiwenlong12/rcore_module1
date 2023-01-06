
/// 将传给 `console` 的控制台对象。
/// 
/// 这是一个 Unit struct，它不需要空间。否则需要传一个 static 对象。
struct Console;

/// 为 `Console` 实现 `console::Console` trait。
impl rcore_console::Console for Console {
    fn put_char(&self, c: u8) {
        #[allow(deprecated)]
        legacy::console_putchar(c as _);
    }
}

use super::*;
#[test]
fn ch1_println(){
    println!("");
    assert!(true);
}
