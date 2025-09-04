use headless_chrome::{Browser, LaunchOptions};
use reqwest::blocking::Client;
use reqwest::header::{HeaderMap, HeaderValue};
use scraper::{Html, Selector};
use std::fs;
use std::io::copy;
use std::path::Path;
use std::time::Duration;

#[derive(Debug)]
pub enum GalleryError {
    RequestFailed,
    ParseError,
    IoError(std::io::Error),
    BrowserError(String), // 新增错误类型
}

impl From<std::io::Error> for GalleryError {
    fn from(err: std::io::Error) -> Self {
        GalleryError::IoError(err)
    }
}

/// 下载图库中的所有图片
pub fn download_gallery(
    url: impl AsRef<str>,
    save_dir: impl AsRef<str>,
) -> Result<(), GalleryError> {
    let url = url.as_ref();
    let save_dir = save_dir.as_ref();

    // 创建用于下载图片的客户端
    let client = create_http_client()?;

    // 使用无头浏览器获取动态内容
    let html = fetch_dynamic_html(url)?;

    let image_urls = parse_image_urls(&html)?;

    fs::create_dir_all(save_dir)?;
    download_images(&client, &image_urls, url, save_dir)?;

    Ok(())
}

/// 创建HTTP客户端（仅用于下载图片）
fn create_http_client() -> Result<Client, GalleryError> {
    let mut headers = HeaderMap::new();
    headers.insert("User-Agent", HeaderValue::from_static(
        "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/91.0.4472.124 Safari/537.36"
    ));
    headers.insert(
        "Accept",
        HeaderValue::from_static(
            "text/html,application/xhtml+xml,application/xml;q=0.9,image/webp,*/*;q=0.8",
        ),
    );
    headers.insert(
        "Accept-Language",
        HeaderValue::from_static("en-US,en;q=0.5"),
    );
    headers.insert("Connection", HeaderValue::from_static("keep-alive"));

    Client::builder()
        .default_headers(headers)
        .timeout(Duration::from_secs(30))
        .build()
        .map_err(|_| GalleryError::RequestFailed)
}

/// 使用无头浏览器获取动态生成的HTML
fn fetch_dynamic_html(url: &str) -> Result<String, GalleryError> {
    // 配置浏览器选项
    let options = LaunchOptions {
        headless: true,
        sandbox: false, // 某些环境可能需要禁用沙盒
        ..Default::default()
    };

    // 启动浏览器
    let browser = Browser::new(options).map_err(|e| GalleryError::BrowserError(e.to_string()))?;

    // 创建新标签页
    let tab = browser
        .new_tab()
        .map_err(|e| GalleryError::BrowserError(e.to_string()))?;

    // 导航到目标URL
    tab.navigate_to(url)
        .map_err(|e| GalleryError::BrowserError(e.to_string()))?;

    // 等待页面加载（可根据需要调整等待时间）
    tab.wait_until_navigated()
        .map_err(|e| GalleryError::BrowserError(e.to_string()))?;

    // 可选：等待特定元素出现（根据需要调整）
    let _ = tab.wait_for_element("img").ok();

    // 获取页面HTML内容
    let html = tab
        .get_content()
        .map_err(|e| GalleryError::BrowserError(e.to_string()))?;

    Ok(html)
}

/// 解析图片URL
fn parse_image_urls(html: &str) -> Result<Vec<String>, GalleryError> {
    let document = Html::parse_document(html);

    // 同时匹配常规图片和延迟加载图片
    let selector = Selector::parse("img").map_err(|_| GalleryError::ParseError)?;

    let mut image_urls = Vec::new();
    for element in document.select(&selector) {
        // 优先获取data-src（延迟加载），其次是src
        if let Some(src) = element.value().attr("data-src") {
            image_urls.push(src.to_string());
        } else if let Some(src) = element.value().attr("src") {
            image_urls.push(src.to_string());
        }
    }

    if image_urls.is_empty() {
        return Err(GalleryError::ParseError);
    }

    Ok(image_urls)
}

/// 下载所有图片
fn download_images(
    client: &Client,
    image_urls: &[String],
    referer: &str,
    save_dir: &str,
) -> Result<(), GalleryError> {
    for (index, url) in image_urls.iter().enumerate() {
        download_image(client, url, referer, save_dir, index)?;
    }
    Ok(())
}

/// 下载单个图片
fn download_image(
    client: &Client,
    img_url: &str,
    referer: &str,
    save_dir: &str,
    index: usize,
) -> Result<(), GalleryError> {
    const MAX_RETRIES: u32 = 3;
    let mut retry_count = 0;

    while retry_count < MAX_RETRIES {
        let mut request = client.get(img_url);
        request = request.header("Referer", referer);

        match request.send() {
            Ok(mut response) => {
                let file_name = Path::new(img_url)
                    .file_name()
                    .and_then(|n| n.to_str())
                    .map(|s| s.to_string())
                    .unwrap_or_else(|| format!("image_{index}.jpg"));

                let path = Path::new(save_dir).join(&file_name);
                let mut file = fs::File::create(&path)?;

                copy(&mut response, &mut file)?;
                println!("Downloaded: {file_name}");
                return Ok(());
            }
            Err(_) => {
                retry_count += 1;
                std::thread::sleep(Duration::from_secs(2));
            }
        }
    }

    eprintln!("Failed to download: {img_url}");
    Ok(()) // 单个图片失败不中断整个流程
}
