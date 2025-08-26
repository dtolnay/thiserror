use std::{
    fmt::{self, Debug, Display, Pointer},
    io::BufWriter,
    string::String,
    vec::Vec,
};

use backtrace::{Backtrace, BacktraceFmt, PrintFmt};

pub struct PrettyBacktrace {
    bt: Backtrace,
}

use crossterm::style::Stylize;
// Rust, as of now, tries to print errors by trait Debug.
impl std::fmt::Debug for PrettyBacktrace {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let style = if fmt.alternate() {
            PrintFmt::Full
        } else {
            PrintFmt::Short
        };

        // Copied from crate Backtrace, and modified. 

        // When printing paths we try to strip the cwd if it exists, otherwise
        // we just print the path as-is. Note that we also only do this for the
        // short format, because if it's full we presumably want to print
        // everything.
        let cwd = std::env::current_dir();
        let mut print_path =
            move |fmt: &mut fmt::Formatter<'_>, path: backtrace::BytesOrWideString<'_>| {
                let path = path.into_path_buf();
                if style == backtrace::PrintFmt::Full {
                    if let Ok(cwd) = &cwd {
                        if let Ok(suffix) = path.strip_prefix(cwd) {
                            return fmt::Display::fmt(&suffix.display(), fmt);
                        }
                    }
                }
                fmt::Display::fmt(&path.display(), fmt)
            };

        let mut f = BacktraceFmt::new(fmt, style, &mut print_path);
        f.add_context()?;
        
        for frame in self.bt.frames() {
            f.frame().backtrace_frame(frame)?;
        }
        f.finish()?;

        Result::Ok(())
    }
}

impl PrettyBacktrace {
    pub fn new() -> Self {
        PrettyBacktrace {
            bt: Backtrace::new(),
        }
    }
}
