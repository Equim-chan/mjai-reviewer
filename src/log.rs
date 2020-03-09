#[macro_export]
macro_rules! log {
    () => {
        eprintln!("{} {}:{}", chrono::Local::now().time(), file!(), line!())
    };
    ($($arg:tt)*) => {
        eprintln!("{} {}:{}\t{}", chrono::Local::now().time(), file!(), line!(), format_args!($($arg)*))
    };
}
