use std::process::exit;

use crate::tokenizer::source_cursor::SourcePtr;

pub fn err_display<S: Into<String>>(msg: S, src_ptr: SourcePtr) -> ! {
    eprintln!(
        "Line {} col {} error: {}",
        src_ptr.line,
        src_ptr.col,
        msg.into()
    );
    exit(1)
}

pub fn err_display_no_source<S: Into<String>>(msg: S) -> ! {
    eprintln!("error: {}", msg.into());
    exit(1)
}
