use std::path::Path;
use tokio::fs;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;
use anyhow::Result;

/// Simple static file server for the frontend
pub struct StaticFileServer {
    frontend_dir: String,
}

impl StaticFileServer {
    pub fn new(frontend_dir: &str) -> Self {
        Self {
            frontend_dir: frontend_dir.to_string(),
        }
    }

    pub async fn serve_file(&self, path: &str, mut stream: TcpStream) -> Result<()> {
        let file_path = if path == "/" || path.is_empty() {
            format!("{}/index.html", self.frontend_dir)
        } else {
            format!("{}{}", self.frontend_dir, path)
        };

        // Security: prevent directory traversal
        let canonical_frontend = Path::new(&self.frontend_dir).canonicalize()?;
        let canonical_file = match Path::new(&file_path).canonicalize() {
            Ok(path) => path,
            Err(_) => {
                return self.send_404(stream).await;
            }
        };

        if !canonical_file.starts_with(&canonical_frontend) {
            return self.send_404(stream).await;
        }

        // Check if file exists
        if !canonical_file.exists() {
            return self.send_404(stream).await;
        }

        // Read file content
        let content = match fs::read(&canonical_file).await {
            Ok(content) => content,
            Err(_) => {
                return self.send_500(stream).await;
            }
        };

        // Determine content type
        let content_type = self.get_content_type(&file_path);

        // Send response
        let response = format!(
            "HTTP/1.1 200 OK\r\nContent-Type: {}\r\nContent-Length: {}\r\nAccess-Control-Allow-Origin: *\r\nAccess-Control-Allow-Methods: GET, POST, OPTIONS\r\nAccess-Control-Allow-Headers: Content-Type\r\n\r\n",
            content_type,
            content.len()
        );

        stream.write_all(response.as_bytes()).await?;
        stream.write_all(&content).await?;
        stream.flush().await?;

        Ok(())
    }

    async fn send_404(&self, mut stream: TcpStream) -> Result<()> {
        let response = "HTTP/1.1 404 Not Found\r\nContent-Type: text/html\r\nContent-Length: 47\r\n\r\n<html><body><h1>404 Not Found</h1></body></html>";
        stream.write_all(response.as_bytes()).await?;
        stream.flush().await?;
        Ok(())
    }

    async fn send_500(&self, mut stream: TcpStream) -> Result<()> {
        let response = "HTTP/1.1 500 Internal Server Error\r\nContent-Type: text/html\r\nContent-Length: 64\r\n\r\n<html><body><h1>500 Internal Server Error</h1></body></html>";
        stream.write_all(response.as_bytes()).await?;
        stream.flush().await?;
        Ok(())
    }

    fn get_content_type(&self, file_path: &str) -> &'static str {
        if file_path.ends_with(".html") {
            "text/html; charset=utf-8"
        } else if file_path.ends_with(".css") {
            "text/css"
        } else if file_path.ends_with(".js") {
            "application/javascript"
        } else if file_path.ends_with(".json") {
            "application/json"
        } else if file_path.ends_with(".png") {
            "image/png"
        } else if file_path.ends_with(".jpg") || file_path.ends_with(".jpeg") {
            "image/jpeg"
        } else if file_path.ends_with(".svg") {
            "image/svg+xml"
        } else {
            "text/plain"
        }
    }
}