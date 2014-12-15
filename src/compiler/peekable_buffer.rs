use std::io::{IoResult, IoErrorKind};

pub struct PeekableBuffer<B: Buffer> {
    source: B,
    backlog: Vec<char>,
}

impl<B: Buffer> PeekableBuffer<B> {
    pub fn new(source: B) -> PeekableBuffer<B> {
        PeekableBuffer {
            source: source,
            backlog: Vec::new(),
        }
    }

    pub fn consume(&mut self) -> IoResult<()> {
        if self.backlog.is_empty() {
            try!(self.source.read_char());
        } else {
            self.backlog.remove(0).unwrap();
        }
        Ok(())
    }

    pub fn is_eof(&mut self) -> bool {
        match self.peek_char() {
            Err(ref e) if e.kind == IoErrorKind::EndOfFile => true,
            _ => false,
        }
    }

    pub fn peek_char(&mut self) -> IoResult<char> {
        let value = if self.backlog.is_empty() {
            let c = try!(self.source.read_char());
            self.backlog.push(c);
            c
        } else {
            self.backlog[0]
        };
        Ok(value)
    }

    pub fn peek_peek_char(&mut self) -> IoResult<char> {
        match self.backlog.len() {
            0 => {
                let c1 = try!(self.source.read_char());
                let c2 = try!(self.source.read_char());
                self.backlog.push(c1);
                self.backlog.push(c2);
            }
            1 => {
                let c = try!(self.source.read_char());
                self.backlog.push(c);
            }
            _ => (),
        }

        Ok(self.backlog[1])
    }

    pub fn read_char(&mut self) -> IoResult<char> {
        let value = if self.backlog.is_empty() {
            try!(self.source.read_char())
        } else {
            self.backlog.remove(0).unwrap()
        };
        Ok(value)
    }
}

#[cfg(test)]
mod test {
    use super::PeekableBuffer;

    #[test]
    fn consume_consumes_char_from_source() {
        let source = "abc".as_bytes();
        let mut buffer = PeekableBuffer::new(source);
        buffer.consume().unwrap();
        assert_eq!(buffer.read_char(), Ok('b'));
        assert_eq!(buffer.read_char(), Ok('c'));
    }

    #[test]
    fn consume_consumes_char_from_peek_buffer() {
        let source = "abc".as_bytes();
        let mut buffer = PeekableBuffer::new(source);
        assert_eq!(buffer.peek_char(), Ok('a'));
        buffer.consume().unwrap();
        assert_eq!(buffer.read_char(), Ok('b'));
        assert_eq!(buffer.read_char(), Ok('c'));
    }

    #[test]
    fn is_eof_returns_true_if_eof() {
        let source = "".as_bytes();
        let mut buffer = PeekableBuffer::new(source);
        assert!(buffer.is_eof());
    }

    #[test]
    fn is_eof_returns_false_if_not_eof() {
        let source = "a".as_bytes();
        let mut buffer = PeekableBuffer::new(source);
        assert!(!buffer.is_eof());
    }

    #[test]
    fn peek_char_does_not_consume() {
        let source = "abc".as_bytes();
        let mut buffer = PeekableBuffer::new(source);
        assert_eq!(buffer.peek_char(), Ok('a'));
        assert_eq!(buffer.read_char(), Ok('a'));
        assert_eq!(buffer.read_char(), Ok('b'));
        assert_eq!(buffer.read_char(), Ok('c'));
    }

    #[test]
    fn peek_peek_char_looks_ahead() {
        let source = "abc".as_bytes();
        let mut buffer = PeekableBuffer::new(source);
        assert_eq!(buffer.peek_peek_char(), Ok('b'));
    }

    #[test]
    fn peek_char_saves_result() {
        let source = "ab".as_bytes();
        let mut buffer = PeekableBuffer::new(source);
        assert_eq!(buffer.peek_char(), Ok('a'));
        assert_eq!(buffer.peek_char(), Ok('a'));
        assert_eq!(buffer.read_char(), Ok('a'));
        assert_eq!(buffer.read_char(), Ok('b'));
    }

    #[test]
    fn read_char_consumes_char() {
        let source = "abc".as_bytes();
        let mut buffer = PeekableBuffer::new(source);
        assert_eq!(buffer.read_char(), Ok('a'));
        assert_eq!(buffer.read_char(), Ok('b'));
        assert_eq!(buffer.read_char(), Ok('c'));
    }
}
