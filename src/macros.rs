//! Module to contain all the custom marco definitions.


/// Exits the program
///
/// This macro calls exit if running in normal mode, and panics if called in a test.
/// This is because tests require an explicit panic, whereas a user who typed an argument wrong
/// does not want to see a panic.
#[macro_export]
macro_rules! exit {
    ($x: literal) => {
        #[cfg(test)]
        panic!();
        #[cfg(not(test))]
        std::process::exit($x);
    };
    () => {
        #[cfg(test)]
        panic!();
        #[cfg(not(test))]
        std::process::exit(0);
    };
}
