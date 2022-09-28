

pub fn init_console() {
    printlib::init_console(&Console);
    printlib::set_log_level(option_env!("LOG"));
    printlib::test_log();
}

struct Console;

impl printlib::Console for Console {
    #[inline]
    fn put_char(&self, c: u8) {
        #[allow(deprecated)]
        sbi_rt::legacy::console_putchar(c as _);
    }
}