use std::io::{BufRead, Error};

pub struct PeekableBuffer<R: BufRead> {
    source: R,
    buffer: String,
    pos: usize,
    peeked: Option<char>,
}

impl<R: BufRead> PeekableBuffer<R> {
    pub fn new(source: R) -> PeekableBuffer<R> {
        PeekableBuffer {
            source: source,
            buffer: String::new(),
            pos: 0,
            peeked: None,
        }
    }

    pub fn peek(&mut self) -> Option<char> {
        if let Some(_) = self.fill_buffer() {
            return None;
        }

        if self.peeked.is_none() {
            self.peeked = self.next();
        }

        self.peeked
    }

    pub fn consume(&mut self) {
        if self.peeked.is_some() {
            self.peeked = None;
        } else {
            self.next();
        }
    }

    #[inline]
    fn fill_buffer(&mut self) -> Option<Error> {
        if self.pos >= self.buffer.len() {
            self.source.read_line(&mut self.buffer).err()
        } else {
            None
        }
    }
}

impl<R: BufRead> Iterator for PeekableBuffer<R> {
    type Item = char;

    fn next(&mut self) -> Option<char> {
        if let Some(_) = self.fill_buffer() {
            return None;
        }

        if let Some(c) = self.peeked {
            self.peeked = None;
            Some(c)
        } else {
            let value = self.buffer.chars().nth(self.pos);
            self.pos += 1;
            value
        }
    }
}

#[cfg(test)]
mod test {
    use super::PeekableBuffer;

    #[test]
    fn test_next_reads_values() {
        let source = "abc".as_bytes();
        let mut buffer = PeekableBuffer::new(source);
        assert_eq!(buffer.next(), Some('a'));
        assert_eq!(buffer.next(), Some('b'));
        assert_eq!(buffer.next(), Some('c'));
        assert_eq!(buffer.next(), None);
    }

    #[test]
    fn test_next_reloads_buffer() {
        let source = "a\nbc".as_bytes();
        let mut buffer = PeekableBuffer::new(source);
        assert_eq!(buffer.next(), Some('a'));
        assert_eq!(buffer.next(), Some('\n'));
        assert_eq!(buffer.next(), Some('b'));
        assert_eq!(buffer.next(), Some('c'));
        assert_eq!(buffer.next(), None);
    }

    #[test]
    fn test_next_consumes_value() {
        let source = "abc".as_bytes();
        let mut buffer = PeekableBuffer::new(source);
        buffer.next();
        assert_eq!(buffer.peek(), Some('b'));
    }

    #[test]
    fn test_next_returns_peeked() {
        let source = "abc".as_bytes();
        let mut buffer = PeekableBuffer::new(source);
        buffer.peek();
        assert_eq!(buffer.next(), Some('a'));
    }

    #[test]
    fn test_next_consumes_peeked() {
        let source = "abc".as_bytes();
        let mut buffer = PeekableBuffer::new(source);
        buffer.peek();
        assert_eq!(buffer.next(), Some('a'));
        assert_eq!(buffer.next(), Some('b'));
    }

    #[test]
    fn test_peek_returns_first_value() {
        let source = "abc".as_bytes();
        let mut buffer = PeekableBuffer::new(source);
        assert_eq!(buffer.peek(), Some('a'));
    }

    #[test]
    fn test_peek_returns_existing_peeked_value() {
        let source = "abc".as_bytes();
        let mut buffer = PeekableBuffer::new(source);
        buffer.peek();
        assert_eq!(buffer.peek(), Some('a'));
    }

    #[test]
    fn test_peek_returns_second_peeked_value() {
        let source = "abc".as_bytes();
        let mut buffer = PeekableBuffer::new(source);
        buffer.next();
        buffer.peek();
        assert_eq!(buffer.peek(), Some('b'));
    }

    #[test]
    fn test_consume() {
        let source = "abc".as_bytes();
        let mut buffer = PeekableBuffer::new(source);
        buffer.peek();
        buffer.consume();
        assert_eq!(buffer.peek(), Some('b'));
    }
}
