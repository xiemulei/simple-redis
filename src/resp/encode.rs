use super::{
    BulkString, RespArray, RespEncode, RespMap, RespNull, RespNullArray, RespNullBulkString,
    RespSet, SimpleError, SimpleString,
};

const BUF_CAP: usize = 4096;

impl RespEncode for SimpleString {
    fn encode(self) -> Vec<u8> {
        format!("+{}\r\n", self.0).into_bytes()
    }
}

impl RespEncode for SimpleError {
    fn encode(self) -> Vec<u8> {
        format!("-{}\r\n", self.0).into_bytes()
    }
}

impl RespEncode for i64 {
    fn encode(self) -> Vec<u8> {
        let sign = if self < 0 { "" } else { "+" };
        format!(":{}{}\r\n", sign, self).into_bytes()
    }
}

impl RespEncode for BulkString {
    fn encode(self) -> Vec<u8> {
        let mut buf = Vec::with_capacity(self.len() + 16);
        buf.extend_from_slice(&format!("${}\r\n", self.len()).into_bytes());
        buf.extend_from_slice(&self);
        buf.extend_from_slice(b"\r\n");
        buf
    }
}

impl RespEncode for RespNullBulkString {
    fn encode(self) -> Vec<u8> {
        b"$-1\r\n".to_vec()
    }
}

impl RespEncode for RespArray {
    fn encode(self) -> Vec<u8> {
        let mut buf = Vec::with_capacity(BUF_CAP);
        buf.extend_from_slice(&format!("*{}\r\n", self.0.len()).into_bytes());
        for item in self.0 {
            buf.extend_from_slice(&item.encode());
        }
        buf
    }
}

impl RespEncode for RespNull {
    fn encode(self) -> Vec<u8> {
        b"_\r\n".to_vec()
    }
}

impl RespEncode for RespNullArray {
    fn encode(self) -> Vec<u8> {
        b"*-1\r\n".to_vec()
    }
}

impl RespEncode for bool {
    fn encode(self) -> Vec<u8> {
        format!("#{}\r\n", if self { "t" } else { "f" }).into_bytes()
    }
}

impl RespEncode for f64 {
    fn encode(self) -> Vec<u8> {
        let mut buf = Vec::with_capacity(32);
        let ret = if self.abs() > 1e+8 || self.abs() < 1e-8 {
            format!(",{:+e}\r\n", self)
        } else {
            let sign = if self < 0.0 { "" } else { "+" };
            format!(",{}{}\r\n", sign, self)
        };
        buf.extend_from_slice(&ret.into_bytes());
        buf
    }
}

impl RespEncode for RespMap {
    fn encode(self) -> Vec<u8> {
        let mut buf = Vec::with_capacity(BUF_CAP);
        buf.extend_from_slice(&format!("%{}\r\n", self.len()).into_bytes());
        for (key, value) in self.0 {
            buf.extend_from_slice(&SimpleString::new(key).encode());
            buf.extend_from_slice(&value.encode());
        }
        buf
    }
}

impl RespEncode for RespSet {
    fn encode(self) -> Vec<u8> {
        let mut buf = Vec::with_capacity(BUF_CAP);
        buf.extend_from_slice(&format!("~{}\r\n", self.len()).into_bytes());
        for item in self.0 {
            buf.extend_from_slice(&item.encode());
        }
        buf
    }
}

#[cfg(test)]
mod tests {
    use crate::RespFrame;

    use super::*;

    #[test]
    fn test_encode_simple_string() {
        let frame: RespFrame = SimpleString::new("OK").into();
        assert_eq!(frame.encode(), b"+OK\r\n".to_vec());
    }

    #[test]
    fn test_encode_simple_error() {
        let frame: RespFrame = SimpleError::new("ERR").into();
        assert_eq!(frame.encode(), b"-ERR\r\n".to_vec());
    }

    #[test]
    fn test_encode_integer() {
        let frame: RespFrame = 42.into();
        assert_eq!(frame.encode(), b":+42\r\n".to_vec());
    }

    #[test]
    fn test_encode_bulk_string() {
        let frame: RespFrame = BulkString::new(b"Hello, World!".to_vec()).into();
        assert_eq!(frame.encode(), b"$13\r\nHello, World!\r\n");
    }

    #[test]
    fn test_encode_null_bulk_string() {
        let frame: RespFrame = RespNullBulkString.into();
        assert_eq!(frame.encode(), b"$-1\r\n".to_vec());
    }

    #[test]
    fn test_encode_array() {
        let frame: RespFrame = RespArray(vec![
            SimpleString::new("set").into(),
            SimpleString::new("hello").into(),
            SimpleString::new("world").into(),
        ])
        .into();
        assert_eq!(
            frame.encode(),
            b"*3\r\n+set\r\n+hello\r\n+world\r\n".to_vec()
        );
    }

    #[test]
    fn test_encode_null() {
        let frame: RespFrame = RespNull.into();
        assert_eq!(frame.encode(), b"_\r\n".to_vec());
    }

    #[test]
    fn test_encode_null_array() {
        let frame: RespFrame = RespNullArray.into();
        assert_eq!(frame.encode(), b"*-1\r\n".to_vec());
    }

    #[test]
    fn test_encode_bool() {
        let frame: RespFrame = true.into();
        assert_eq!(frame.encode(), b"#t\r\n");

        let frame: RespFrame = false.into();
        assert_eq!(frame.encode(), b"#f\r\n");
    }

    #[test]
    fn test_encode_double() {
        let frame: RespFrame = 1.234567.into();
        assert_eq!(frame.encode(), b",+1.234567\r\n");

        let frame: RespFrame = 1.23456e+8.into();
        assert_eq!(frame.encode(), b",+1.23456e8\r\n");

        let frame: RespFrame = (-1.23456e-9).into();
        assert_eq!(frame.encode(), b",-1.23456e-9\r\n");
    }

    #[test]
    fn test_encode_map() {
        let mut map = RespMap::new();
        map.insert("hello".to_string(), BulkString::new("world").into());
        map.insert("foo".to_string(), (-1.23456789).into());

        let frame: RespFrame = map.into();
        assert_eq!(
            frame.encode(),
            b"%2\r\n+foo\r\n,-1.23456789\r\n+hello\r\n$5\r\nworld\r\n"
        )
    }

    #[test]
    fn test_encode_set() {
        let frame: RespSet = RespSet::new([
            RespArray::new([1234.into(), true.into()]).into(),
            BulkString::new("world".to_string()).into(),
        ]);
        assert_eq!(
            String::from_utf8_lossy(&frame.encode()),
            "~2\r\n*2\r\n:+1234\r\n#t\r\n$5\r\nworld\r\n"
        )
    }
}
