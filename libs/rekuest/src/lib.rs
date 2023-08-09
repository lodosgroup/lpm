use std::io::{self, BufRead, BufReader, Read, Write};
use std::net::TcpStream;

pub struct Rekuest {
    host: String,
    request_data: String,
}

pub struct HttpResponse {
    pub headers: Vec<(String, String)>,
    pub body: Vec<u8>,
    pub status_code: u16,
}

impl Rekuest {
    pub fn new(url: &str) -> io::Result<Self> {
        let (host, path) = parse_url(url).ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("Couldn't parse {}", url),
            )
        })?;

        let mut rekuest = Self {
            host,
            request_data: String::new(),
        };

        rekuest
            .request_data
            .push_str(&format!("GET /{} HTTP/1.1", path));

        rekuest.add_header("Host", &rekuest.host.to_string());
        rekuest.add_header("Connection", "close");

        Ok(rekuest)
    }

    pub fn add_header(&mut self, key: &str, value: &str) {
        self.request_data.push_str("\r\n");
        self.request_data.push_str(&format!("{}: {}", key, value));
    }

    pub fn get(self) -> io::Result<HttpResponse> {
        let mut stream = TcpStream::connect(&self.host)?;
        stream.set_nodelay(true)?;

        let mut request_data = self.request_data;
        request_data.push_str("\r\n");
        request_data.push_str("\r\n");

        stream.write_all(request_data.as_bytes())?;

        let mut response = HttpResponse {
            headers: Vec::new(),
            body: Vec::new(),
            status_code: 0,
        };

        let mut headers: Vec<u8> = Vec::new();

        let mut reader = BufReader::new(&stream);
        read_until_nrt(&mut reader, &mut headers)?;

        // ignore '\n'
        reader.consume(1);

        let headers = String::from_utf8(headers)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, format!("{}", e)))?;

        let mut lines = headers.lines();
        if let Some(status_line) = lines.next() {
            let status_code = parse_status_code(status_line)?;
            response.status_code = status_code;
        }

        for line in lines {
            if let Some((header_name, header_value)) = parse_header(line) {
                response.headers.push((header_name, header_value));
            }
        }

        let mut body = Vec::new();
        reader.read_to_end(&mut body)?;

        response.body = body;

        Ok(response)
    }
}

impl HttpResponse {
    pub fn get_header_value(&self, k: &str) -> Option<&str> {
        for (header_key, header_value) in &self.headers {
            if header_key == k {
                return Some(header_value);
            }
        }
        None
    }
}

fn parse_url(url: &str) -> Option<(String, String)> {
    let url = if let Some(without_prefix) = url.strip_prefix("http://") {
        without_prefix
    } else {
        url
    };

    let mut url_parts = url.splitn(2, '/');
    let host_port = url_parts.next()?;
    let path = url_parts.next().unwrap_or_default();

    let mut host_parts = host_port.splitn(2, ':');
    let host = host_parts.next()?;
    let port = host_parts.next().and_then(|p| p.parse().ok()).unwrap_or(80);

    Some((format!("{}:{}", host, port), path.to_owned()))
}

fn parse_status_code(status_line: &str) -> io::Result<u16> {
    let parts: Vec<&str> = status_line.split(' ').collect();
    if parts.len() >= 3 {
        parts[1]
            .parse()
            .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "Invalid status code"))
    } else {
        Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "Parsing status code failed",
        ))
    }
}

fn parse_header(line: &str) -> Option<(String, String)> {
    let mut parts = line.splitn(2, ':');
    if let (Some(name), Some(value)) = (parts.next(), parts.next()) {
        Some((name.trim().to_owned(), value.trim().to_owned()))
    } else {
        None
    }
}

fn read_until_nrt<R: BufRead>(reader: &mut R, buf: &mut Vec<u8>) -> io::Result<()> {
    loop {
        reader.read_until(b'\r', buf)?;
        if buf.ends_with(&[b'\n', b'\r']) {
            buf.pop();
            buf.pop();
            buf.pop();
            break;
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_url() {
        // with default port and with http prefix
        {
            let url = "http://linux-amd64-default.lpm.lodosgroup.org";
            let (host, path) = parse_url(url).unwrap();
            assert_eq!(host, "linux-amd64-default.lpm.lodosgroup.org:80");
            assert_eq!(path, "");

            let url = "http://linux-amd64-default.lpm.lodosgroup.org/index-tracker";
            let (host, path) = parse_url(url).unwrap();
            assert_eq!(host, "linux-amd64-default.lpm.lodosgroup.org:80");
            assert_eq!(path, "index-tracker");

            let url = "http://linux-amd64-default.lpm.lodosgroup.org/index-tracker/health";
            let (host, path) = parse_url(url).unwrap();
            assert_eq!(host, "linux-amd64-default.lpm.lodosgroup.org:80");
            assert_eq!(path, "index-tracker/health");
        }

        // with custom port and without http prefix
        {
            let url = "linux-amd64-default.lpm.lodosgroup.org:6150";
            let (host, path) = parse_url(url).unwrap();
            assert_eq!(host, "linux-amd64-default.lpm.lodosgroup.org:6150");
            assert_eq!(path, "");

            let url = "linux-amd64-default.lpm.lodosgroup.org:6150/index-tracker";
            let (host, path) = parse_url(url).unwrap();
            assert_eq!(host, "linux-amd64-default.lpm.lodosgroup.org:6150");
            assert_eq!(path, "index-tracker");

            let url = "linux-amd64-default.lpm.lodosgroup.org:6150/index-tracker/health";
            let (host, path) = parse_url(url).unwrap();
            assert_eq!(host, "linux-amd64-default.lpm.lodosgroup.org:6150");
            assert_eq!(path, "index-tracker/health");
        }
    }

    #[test]
    fn test_parse_header() {
        let header_line = "Server: nginx/1.18.0";
        let (key, value) = parse_header(header_line).unwrap();
        assert_eq!(key, "Server");
        assert_eq!(value, "nginx/1.18.0");

        let header_line = "Date: Fri, 02 Jun 2023 12:05:51 GMT";
        let (key, value) = parse_header(header_line).unwrap();
        assert_eq!(key, "Date");
        assert_eq!(value, "Fri, 02 Jun 2023 12:05:51 GMT");

        let header_line = "Content-Type: text/plain";
        let (key, value) = parse_header(header_line).unwrap();
        assert_eq!(key, "Content-Type");
        assert_eq!(value, "text/plain");
    }

    #[test]
    fn test_parse_status_code() {
        let status_line = "HTTP/1.1 200 OK";
        let status_code = parse_status_code(status_line).unwrap();
        assert_eq!(status_code, 200);

        let status_line = "HTTP/1.1 301 Moved Permanently";
        let status_code = parse_status_code(status_line).unwrap();
        assert_eq!(status_code, 301);

        let status_line = "HTTP/1.1 404 ";
        let status_code = parse_status_code(status_line).unwrap();
        assert_eq!(status_code, 404);
    }

    #[test]
    fn test_read_until_nrt() {
        let input = "Header1: Value1\r\nHeader2: Value2\r\n\r\nBodyStandsHere";
        let mut reader = io::Cursor::new(input);
        let mut buf = Vec::new();

        read_until_nrt(&mut reader, &mut buf).unwrap();

        let expected_output = b"Header1: Value1\r\nHeader2: Value2";
        assert_eq!(buf, expected_output);
    }
}
