#[macro_export]
macro_rules! log {
    () => {
        eprintln!("{:>15} {}:{}", chrono::Local::now().time(), file!(), line!())
    };
    ($($arg:tt)*) => {
        eprintln!("{:>15} {}:{}\t{}", chrono::Local::now().time(), file!(), line!(), format_args!($($arg)*))
    };
}
