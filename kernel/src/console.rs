
pub fn init_console() {
    rcore_console::init_console(&Console);
    rcore_console::set_log_level(option_env!("LOG"));
    // rcore_console::test_log();
}

struct Console;

impl rcore_console::Console for Console {
    #[inline]
    fn put_char(&self, c: u8) {
        #[allow(deprecated)]
        sbi_rt::legacy::console_putchar(c as _);
    }
}