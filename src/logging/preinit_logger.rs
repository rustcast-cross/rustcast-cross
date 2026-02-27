//! Ad-hoc logger for before tracing inits
//!
//! Doesn't do anything fancy with macros, just because there isn't *that* much of a point for
//! something only run a handful of times.
//!
//! Not using ANSI escapes is a deliberate choice (there are only a few logs here, and it's better
//! to have a few colourless lines than to have incredibly borked output where ANSI escapes can't
//! be used)

pub fn info(text: &str) {
    println!("INFO  {text}");
}
pub fn warn(text: &str) {
    println!("WARN  {text}");
}
pub fn error(text: &str) {
    println!("ERROR {text}");
}
