use printlib::*;

/// 将传给 `console` 的控制台对象。
///
/// 这是一个 Unit struct，它不需要空间。否则需要传一个 static 对象。
struct Console;

/// 为 `Console` 实现 `console::Console` trait。
impl printlib::Console for Console {
    fn put_char(&self, c: u8) {
        use sbi_rt::*;
        #[allow(deprecated)]
        legacy::console_putchar(c as _);
    }
}

pub fn init_console() {
    printlib::init_console(&Console);
    log::set_max_level(log::LevelFilter::Trace);
}