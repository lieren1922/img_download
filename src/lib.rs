use std::borrow::Cow;
use std::sync::Arc;

use headless_chrome::{Browser, LaunchOptions};
use reqwest::header::{HeaderMap, HeaderValue};
use scraper::{Html, Selector};
use tokio::fs;
use tokio::io::AsyncWriteExt;

#[derive(Debug)]
pub enum GalleryError {
    RequestFailed,
    ParseError,
    IoError(std::io::Error),
    BrowserError(String),
    ReqwestError(reqwest::Error),
    TokioJoinError(tokio::task::JoinError),
}

impl From<reqwest::Error> for GalleryError {
    fn from(err: reqwest::Error) -> Self {
        GalleryError::ReqwestError(err)
    }
}

impl From<tokio::task::JoinError> for GalleryError {
    fn from(err: tokio::task::JoinError) -> Self {
        GalleryError::TokioJoinError(err)
    }
}

impl From<std::io::Error> for GalleryError {
    fn from(err: std::io::Error) -> Self {
        GalleryError::IoError(err)
    }
}

/// 异步下载图库
pub async fn download_gallery(
    url: impl AsRef<str> + Send + 'static,
    save_dir: impl AsRef<str>,
) -> Result<(), GalleryError> {
    let save_dir = save_dir.as_ref();
    let url = url.as_ref();
    let url_string = url.to_string();

    // 创建异步 HTTP 客户端
    let client = create_http_client().await?;

    // 在阻塞线程中执行无头浏览器操作
    let html = tokio::task::spawn_blocking(move || fetch_dynamic_html(&url_string)).await??; // 双重解包：先 await JoinHandle，再解 Result

    let image_urls = parse_image_urls(&html, url)?;
    image_urls.iter().for_each(|url| println!("{url}"));

    fs::create_dir_all(&save_dir).await?;
    download_images(&client, &image_urls, url, save_dir).await?;

    Ok(())
}

/// 创建异步 HTTP 客户端
async fn create_http_client() -> Result<reqwest::Client, GalleryError> {
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

    reqwest::Client::builder()
        .default_headers(headers)
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .map_err(GalleryError::ReqwestError)
}

/// 使用无头浏览器获取动态生成的HTML
fn fetch_dynamic_html(url: &str) -> Result<String, GalleryError> {
    // 配置浏览器选项
    let options = LaunchOptions {
        headless: true,
        sandbox: false,
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

    // 等待页面加载
    tab.wait_until_navigated()
        .map_err(|e| GalleryError::BrowserError(e.to_string()))?;

    // 可选：等待特定元素出现
    let _ = tab.wait_for_element("img").ok();

    // 获取页面HTML内容
    let html = tab
        .get_content()
        .map_err(|e| GalleryError::BrowserError(e.to_string()))?;

    Ok(html)
}

/// 解析图片URL
use url::Url;

fn parse_image_urls(html: &str, base_url: &str) -> Result<Vec<String>, GalleryError> {
    // 解析基础URL
    let base_url = Url::parse(base_url).map_err(|_| GalleryError::ParseError)?;

    let document = Html::parse_document(html);
    let selector = Selector::parse("img").map_err(|_| GalleryError::ParseError)?;

    let mut image_urls = Vec::new();
    for element in document.select(&selector) {
        if let Some(src) = element
            .value()
            .attr("data-src")
            .or_else(|| element.value().attr("src"))
        {
            // 尝试解析为绝对URL
            match Url::parse(src) {
                // 已经是绝对URL
                Ok(absolute_url) => image_urls.push(absolute_url.to_string()),
                // 相对路径，拼接基础URL
                Err(_) => {
                    if let Ok(absolute_url) = base_url.join(src) {
                        image_urls.push(absolute_url.to_string());
                    } else {
                        eprintln!("Failed to join URL: base={base_url}, rel={src}");
                    }
                }
            }
        }
    }

    if image_urls.is_empty() {
        return Err(GalleryError::ParseError);
    }

    Ok(image_urls)
}

/// 异步下载所有图片
async fn download_images(
    client: &reqwest::Client,
    image_urls: &[String],
    referer: &str,
    save_dir: &str,
) -> Result<(), GalleryError> {
    // 使用 Arc 共享不可变数据
    let referer_arc = Arc::new(referer.to_string());
    let save_dir_arc = Arc::new(save_dir.to_string());

    // 将 URL 列表转换为 Arc<str> 的 Vec
    let image_urls_arc: Vec<Arc<str>> = image_urls
        .iter()
        .map(|url| Arc::from(url.as_str()))
        .collect();

    let mut tasks = Vec::with_capacity(image_urls_arc.len());

    for (index, img_url_arc) in image_urls_arc.iter().enumerate() {
        let client = client.clone(); // 轻量级克隆
        let img_url_arc = Arc::clone(img_url_arc); // 增加引用计数
        let referer_arc = Arc::clone(&referer_arc);
        let save_dir_arc = Arc::clone(&save_dir_arc);

        tasks.push(tokio::spawn(async move {
            download_image_single(
                &client,
                &img_url_arc, // 自动解引用为 &str
                &referer_arc,
                &save_dir_arc,
                index,
            )
            .await
        }));
    }

    // 等待所有任务完成
    for task in tasks {
        task.await??; // 处理 JoinError 和 GalleryError
    }

    Ok(())
}

/// 异步下载单张图片
async fn download_image_single(
    client: &reqwest::Client,
    img_url: &str,
    referer: &str,
    save_dir: &str,
    index: usize,
) -> Result<(), GalleryError> {
    const MAX_RETRIES: u32 = 3;
    let mut retry_count = 0;

    while retry_count < MAX_RETRIES {
        let response = client.get(img_url).header("Referer", referer).send().await;

        match response {
            Ok(resp) if resp.status().is_success() => {
                // let file_name = std::path::Path::new(img_url)
                //     .file_name()
                //     .and_then(|n| n.to_str())
                //     .map(|s| s.to_string())
                //     .unwrap_or_else(|| format!("image_{index}.jpg"));
                let file_name: Cow<str> = std::path::Path::new(img_url)
                    .file_name()
                    .and_then(|n| n.to_str())
                    .map(Cow::Borrowed) // 直接借用URL中的字符串片段
                    .unwrap_or_else(|| Cow::Owned(format!("image_{index}.jpg"))); // 仅在需要时分配

                let path = std::path::Path::new(save_dir).join(&*file_name);

                // 异步写入文件
                let mut file = fs::File::create(&path).await?;
                let mut content = resp.bytes().await?;
                file.write_all_buf(&mut content).await?;

                println!("Downloaded: {file_name}");
                return Ok(());
            }
            _ => {
                retry_count += 1;
                tokio::time::sleep(std::time::Duration::from_secs(2)).await;
            }
        }
    }

    eprintln!("Failed to download: {img_url}");
    Ok(())
}
