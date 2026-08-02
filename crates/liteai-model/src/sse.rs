//! SSE 流式解析（纯函数，可单测）。

/// 增量 SSE 解析器：逐块喂入字节，产出已完整到达的 `data:` 载荷。
#[derive(Default)]
pub struct SseParser {
    buf: Vec<u8>,
}

impl SseParser {
    pub fn new() -> Self {
        Self::default()
    }

    /// 喂入一段字节，返回本次解析出的完整 data 载荷列表（不含 "data:" 前缀）。
    pub fn push(&mut self, bytes: &[u8]) -> Vec<String> {
        self.buf.extend_from_slice(bytes);
        let mut out = Vec::new();
        loop {
            match frame_end(&self.buf) {
                Some(end) => {
                    let frame: Vec<u8> = self.buf.drain(..end).collect();
                    if let Some(data) = extract_data(&frame) {
                        out.push(data);
                    }
                }
                None => break,
            }
        }
        out
    }
}

/// 找到第一个完整帧的结束位置（含分隔符）。支持 \n\n 与 \r\n\r\n。
fn frame_end(buf: &[u8]) -> Option<usize> {
    let n = buf.len();
    for i in 0..n.saturating_sub(1) {
        if buf[i] == b'\n' && buf[i + 1] == b'\n' {
            return Some(i + 2);
        }
        if buf[i] == b'\r'
            && buf[i + 1] == b'\n'
            && n >= i + 4
            && buf[i + 2] == b'\r'
            && buf[i + 3] == b'\n'
        {
            return Some(i + 4);
        }
    }
    None
}

/// 从单帧中提取所有 `data:` 行的内容（去前缀、trim、多行用 \n 拼接）。
fn extract_data(frame: &[u8]) -> Option<String> {
    let text = String::from_utf8_lossy(frame);
    let mut data = String::new();
    for line in text.lines() {
        if let Some(v) = line.strip_prefix("data:") {
            if !data.is_empty() {
                data.push('\n');
            }
            data.push_str(v.trim());
        }
    }
    if data.is_empty() {
        None
    } else {
        Some(data)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_single_and_done() {
        let mut p = SseParser::new();
        let events = p.push(
            b"data: {\"a\":1}\n\ndata: [DONE]\n\n",
        );
        assert_eq!(events, vec![r#"{"a":1}"#.to_string(), "[DONE]".to_string()]);
    }

    #[test]
    fn handles_incremental_chunks() {
        let mut p = SseParser::new();
        let chunk = b"data: hello\ndata: world\n\ndata: [DONE]\n\n";
        // 逐字节喂入，确保增量解析正确
        let mut all = Vec::new();
        for b in chunk {
            all.extend(p.push(&[*b]));
        }
        assert_eq!(all, vec!["hello\nworld".to_string(), "[DONE]".to_string()]);
    }

    #[test]
    fn handles_crlf() {
        let mut p = SseParser::new();
        let events = p.push(b"data: {\"x\":2}\r\n\r\n");
        assert_eq!(events, vec![r#"{"x":2}"#.to_string()]);
    }

    #[test]
    fn incomplete_frame_waits() {
        let mut p = SseParser::new();
        assert!(p.push(b"data: partial").is_empty());
        assert_eq!(p.push(b"\n\n"), vec!["partial".to_string()]);
    }
}
