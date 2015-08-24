use std::io::{BufRead, Error};

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Location(pub usize, pub usize);

pub struct PeekableBuffer<R: BufRead> {
    source: R,
    buffer: String,
    line: usize,
    position: usize,
    peeked: Option<(char, Location)>,
}

impl<R: BufRead> PeekableBuffer<R> {
    pub fn new(source: R) -> PeekableBuffer<R> {
        PeekableBuffer {
            source: source,
            buffer: String::with_capacity(256),
            line: 0,
            position: 0,
            peeked: None,
        }
    }

    pub fn peek(&mut self) -> Option<char> {
        if let Some(_) = self.fill_buffer() {
            return None;
        }

        if self.peeked.is_none() {
            let location = self.location();
            self.peeked = self.next().map(|c| (c, location));
        }

        self.peeked.map(|c| c.0)
    }

    pub fn consume(&mut self) {
        if self.peeked.is_some() {
            self.peeked = None;
        } else {
            self.next();
        }
    }

    pub fn location(&self) -> Location {
        if let Some((_, location)) = self.peeked {
            location
        } else {
            Location(self.line, self.position+1)
        }
    }

    #[inline]
    fn fill_buffer(&mut self) -> Option<Error> {
        if self.position >= self.buffer.len() {
            self.line += 1;
            self.position = 0;
            self.buffer.clear();
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

        if let Some((c, _)) = self.peeked {
            self.peeked = None;
            Some(c)
        } else {
            let value = self.buffer.chars().nth(self.position);
            self.position += 1;
            value
        }
    }
}

#[cfg(test)]
mod test {
    use super::{Location, PeekableBuffer};

    #[test]
    fn location() {
        let source = "a\nbc".as_bytes();
        let mut buffer = PeekableBuffer::new(source);
        buffer.next();
        buffer.next();
        buffer.next();
        assert_eq!(buffer.location(), Location(2, 2))
    }

    #[test]
    fn next_reads_values() {
        let source = "abc".as_bytes();
        let mut buffer = PeekableBuffer::new(source);
        assert_eq!(buffer.next(), Some('a'));
        assert_eq!(buffer.next(), Some('b'));
        assert_eq!(buffer.next(), Some('c'));
        assert_eq!(buffer.next(), None);
    }

    #[test]
    fn next_reloads_buffer() {
        let source = "a\nbc".as_bytes();
        let mut buffer = PeekableBuffer::new(source);
        assert_eq!(buffer.next(), Some('a'));
        assert_eq!(buffer.next(), Some('\n'));
        assert_eq!(buffer.next(), Some('b'));
        assert_eq!(buffer.next(), Some('c'));
        assert_eq!(buffer.next(), None);
    }

    #[test]
    fn next_consumes_value() {
        let source = "abc".as_bytes();
        let mut buffer = PeekableBuffer::new(source);
        buffer.next();
        assert_eq!(buffer.peek(), Some('b'));
    }

    #[test]
    fn next_returns_peeked() {
        let source = "abc".as_bytes();
        let mut buffer = PeekableBuffer::new(source);
        buffer.peek();
        assert_eq!(buffer.next(), Some('a'));
    }

    #[test]
    fn next_consumes_peeked() {
        let source = "abc".as_bytes();
        let mut buffer = PeekableBuffer::new(source);
        buffer.peek();
        assert_eq!(buffer.next(), Some('a'));
        assert_eq!(buffer.next(), Some('b'));
    }

    #[test]
    fn peek_returns_first_value() {
        let source = "abc".as_bytes();
        let mut buffer = PeekableBuffer::new(source);
        assert_eq!(buffer.peek(), Some('a'));
    }

    #[test]
    fn peek_returns_existing_peeked_value() {
        let source = "abc".as_bytes();
        let mut buffer = PeekableBuffer::new(source);
        buffer.peek();
        assert_eq!(buffer.peek(), Some('a'));
    }

    #[test]
    fn peek_returns_second_peeked_value() {
        let source = "abc".as_bytes();
        let mut buffer = PeekableBuffer::new(source);
        buffer.next();
        buffer.peek();
        assert_eq!(buffer.peek(), Some('b'));
    }

    #[test]
    fn consume() {
        let source = "abc".as_bytes();
        let mut buffer = PeekableBuffer::new(source);
        buffer.peek();
        buffer.consume();
        assert_eq!(buffer.peek(), Some('b'));
    }
}
