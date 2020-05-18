pub struct LineBuf<T: FnMut(String)> {
    accept_lines: T,
    buf: Vec<u8>,
}

const LF: u8 = '\n' as u8;

impl<T: FnMut(String)> LineBuf<T> {
    pub fn new(buffer_size: usize, accept_lines: T) -> Self {
        let buf = Vec::with_capacity(buffer_size);
        LineBuf { accept_lines, buf }
    }

    pub fn accept(&mut self, bytes: &[u8]) {
        for b in bytes.iter() {
            if *b == LF {
                let mut out = self.buf.split_off(0);
                out.push(LF);
                (self.accept_lines)(String::from_utf8_lossy(&out).into());
            } else {
                self.buf.push(*b);
                if self.buf.len() == self.buf.capacity() {
                    (self.accept_lines)(String::from_utf8_lossy(&self.buf).into());
                    self.buf.clear();
                }
            }
        }
    }

    pub fn close(mut self) {
        if !self.buf.is_empty() {
            (self.accept_lines)(String::from_utf8_lossy(&self.buf).into());
        }
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;

    fn test(s: &str, expected: Vec<&str>) {
        let mut lines = Vec::new();
        let mut buf = LineBuf::new(40, |line| lines.push(line));
        buf.accept(&s.bytes().collect::<Vec<_>>());
        buf.close();
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
            "0123456789012345678901234567890123456789012345678901234567890123456789",
            vec![
                "0123456789012345678901234567890123456789",
                "0123456789012345678901234567890123456789",
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
}
