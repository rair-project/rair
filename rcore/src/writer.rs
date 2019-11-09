/*
 * writer.rs: Abstract implementation for Io::Write stream
 * Copyright (C) 2019  Oddcoder
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU Lesser General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU Lesser General Public License for more details.
 *
 * You should have received a copy of the GNU Lesser General Public License
 * along with this program.  If not, see <http://www.gnu.org/licenses/>.
 */
use std::io;
use std::io::Write;

/// This union acts as thin abstraction layer over over input streams.
/// Its goal is to allow allow seamingless redirection of output to
/// either a buffer, a file or even terminal IO.
pub enum Writer {
    #[doc(hidden)]
    Write(Box<dyn Write>),
    #[doc(hidden)]
    Bytes(Vec<u8>),
}

impl Write for Writer {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        match self {
            Writer::Write(writer) => writer.write(buf),
            Writer::Bytes(bytes) => bytes.write(buf),
        }
    }
    fn flush(&mut self) -> io::Result<()> {
        match self {
            Writer::Write(writer) => writer.flush(),
            Writer::Bytes(bytes) => bytes.flush(),
        }
    }
}

impl Writer {
    /// Creates a new [Writer] backed by object that implements [Write].
    pub fn new_write(out: Box<dyn Write>) -> Self {
        return Writer::Write(out);
    }

    /// Returns a new buffer based [Writer].
    pub fn new_buf() -> Self {
        return Writer::Bytes(Vec::new());
    }
    /// This function consumes the [Writer] object, it returns the
    /// data stored there if the object is buffer based.
    pub fn bytes(self) -> Option<Vec<u8>> {
        if let Writer::Bytes(b) = self {
            return Some(b);
        } else {
            return None;
        }
    }
    /// This function consumes the [Writer] object, it returns UTF-8
    /// String representation of the data stored there if it is buffer.
    /// based and the buffer holds UTF-8 data.
    pub fn utf8_string(self) -> Option<String> {
        if let Writer::Bytes(b) = self {
            if let Ok(s) = String::from_utf8(b) {
                return Some(s);
            } else {
                return None;
            }
        } else {
            return None;
        }
    }
    /// This function returns a reference to the data stored
    /// in respective [Writer] if the object is buffer based.
    pub fn bytes_ref(&self) -> Option<&Vec<u8>> {
        if let Writer::Bytes(b) = self {
            return Some(b);
        } else {
            return None;
        }
    }

    /// This function returns a mutable reference to the data
    /// stored in respective [Writer] if the object is buffer.
    /// based.
    pub fn bytes_mut(&mut self) -> Option<&mut Vec<u8>> {
        if let Writer::Bytes(b) = self {
            return Some(b);
        } else {
            return None;
        }
    }
}

#[cfg(test)]
mod writer_test {
    use super::*;
    #[test]
    fn test_writer_buffer() {
        let mut w = Writer::new_buf();
        let s = "Testing write buffer with utf8 heart ♥";
        let v = s.as_bytes();
        write!(w, "Testing write buffer with utf8 heart ♥").unwrap();
        assert_eq!(w.bytes_ref().unwrap(), &v);
        assert_eq!(w.bytes_mut().unwrap(), &v);
        assert_eq!(w.utf8_string().unwrap(), s);
        w = Writer::new_buf();
        write!(w, "Testing write buffer with utf8 heart ♥").unwrap();
        assert_eq!(w.bytes().unwrap(), v);
    }

    #[test]
    fn test_writer_io() {
        let mut w = Writer::new_write(Box::new(io::stdout()));
        assert_eq!(w.bytes_ref(), None);
        assert_eq!(w.bytes_mut(), None);
        assert_eq!(w.utf8_string(), None);
        w = Writer::new_write(Box::new(io::stdout()));
        assert_eq!(w.bytes(), None);
    }
}
