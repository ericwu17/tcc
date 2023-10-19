use crate::tokenizer::source_cursor::SourcePtr;
use crate::tokenizer::Token;

pub struct TokenCursor {
    contents: Vec<(Token, SourcePtr)>,
    index: usize,
}

impl TokenCursor {
    pub fn new(contents: Vec<(Token, SourcePtr)>) -> Self {
        TokenCursor { contents, index: 0 }
    }

    pub fn peek(&self) -> Option<&Token> {
        self.contents.get(self.index).map(|(token, _)| token)
    }
    pub fn peek_nth(&self, n: usize) -> Option<&Token> {
        // peek_nth(1) is equivalent to peek()
        self.contents
            .get(self.index + n - 1)
            .map(|(token, _)| token)
    }

    // pub fn peek_ptr(&self) -> Option<&SourcePtr> {
    //     self.contents.get(self.index).map(|(_, src_ptr)| src_ptr)
    // }

    // pub fn peek_nth_ptr(&self, n: usize) -> Option<&SourcePtr> {
    //     self.contents
    //         .get(self.index + n - 1)
    //         .map(|(_, src_ptr)| src_ptr)
    // }

    pub fn next(&mut self) -> Option<&Token> {
        if self.index <= self.contents.len() {
            self.index += 1;
        }
        self.contents.get(self.index - 1).map(|(token, _)| token)
    }

    pub fn last(&self) -> Option<&Token> {
        self.contents.get(self.index - 1).map(|(token, _)| token)
    }

    pub fn get_last_ptr(&self) -> SourcePtr {
        let optional_ptr = self.contents.get(self.index - 1).map(|(_, ptr)| *ptr);
        return optional_ptr.unwrap_or(SourcePtr::new());
    }
}
