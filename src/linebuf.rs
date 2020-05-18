pub struct LineBuf {
    buf: Vec<u8>,
    buffer_size: usize,
    pending_cr: bool,
}

const LF: u8 = '\n' as u8;
const CR: u8 = '\r' as u8;

impl LineBuf {
    pub fn new(buffer_size: usize) -> Self {
        let buf = Vec::with_capacity(buffer_size);
        LineBuf { buf, buffer_size, pending_cr: false }
    }

    pub fn accept<T: FnMut(String)>(&mut self, bytes: &[u8], accept_lines: &mut T) {
        debug_assert!(self.buf.capacity() == self.buffer_size);
        for b in bytes.iter() {
            if *b == CR {
                self.pending_cr = true;
            } else if *b == LF {
                if self.pending_cr {
                    // Ignore CR+LF
                    self.pending_cr = false;
                }
                let mut out = self.buf.split_off(0);
                out.push(LF);
                (accept_lines)(String::from_utf8_lossy(&out).into());
            } else {
                if self.pending_cr {
                    self.buf.clear();
                    self.pending_cr = false;
                }
                if self.buf.len() == self.buf.capacity() {
                    (accept_lines)(String::from_utf8_lossy(&self.buf).into());
                    self.buf.clear();
                }
                self.buf.push(*b);
            }
        }
        debug_assert!(self.buf.capacity() == self.buffer_size);
    }

    pub fn close<T: FnMut(String)>(self, accept_lines: &mut T) {
        if self.pending_cr {
            return;
        }
        if !self.buf.is_empty() {
            (accept_lines)(String::from_utf8_lossy(&self.buf).into());
        }
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;

    fn test(s: &str, expected: Vec<&str>) {
        let mut lines = Vec::new();
        let mut buf = LineBuf::new(40);
        buf.accept(&s.bytes().collect::<Vec<_>>(), &mut |line| lines.push(line));
        buf.close(&mut |line| lines.push(line));
        assert_eq!(lines, expected);
    }

    #[test]
    fn test_short_lines() {
        test(
            "hello world\nbye world\n",
            vec!["hello world\n", "bye world\n"],
        );
    }

    #[test]
    fn test_long_lines() {
        test(
            "01234567890123456789012345678901234567890123456789012345678901234567890123456789",
            vec![
                "0123456789012345678901234567890123456789",
                "0123456789012345678901234567890123456789",
            ],
        );
        test(
            "012345678901234567890123456789012345678901234567890123456789012345678901234567890",
            vec![
                "0123456789012345678901234567890123456789",
                "0123456789012345678901234567890123456789",
                "0",
            ],
        );
        test(
            "0123456789012345678901234567890123456789\n0123456789012345678901234567890123456789",
            vec![
                "0123456789012345678901234567890123456789\n",
                "0123456789012345678901234567890123456789",
            ],
        );
        test(
            "0123456789012345678901234567890123456789\n0123456789012345678901234567890123456789\n",
            vec![
                "0123456789012345678901234567890123456789\n",
                "0123456789012345678901234567890123456789\n",
            ],
        );
        test(
            "01234567890123456789012345678901234567890123456789012345678901234567890123456789\n",
            vec![
                "0123456789012345678901234567890123456789",
                "0123456789012345678901234567890123456789\n",
            ],
        );
    }

    #[test]
    fn test_empty() {
        test("", vec![]);
    }

    #[test]
    fn test_no_lf() {
        test("hello world", vec!["hello world"]);
    }

    #[test]
    fn test_cr_lf() {
        // DOS-style CR+LF
        test("hello world\r\n", vec!["hello world\n"]);
        test("hello world\r\nbye world", vec!["hello world\n", "bye world"]);
        test("hello world\r\nbye world\r\n", vec!["hello world\n", "bye world\n"]);
    }

    #[test]
    fn test_cr() {
        // Curl-style CR
        test("test1\rtest2\rtest3\n", vec!["test3\n"]);
        test("test1\rtest2\rtest3", vec!["test3"]);
        test("test1\rtest2\rtest3\r", vec![]);
    }
}
