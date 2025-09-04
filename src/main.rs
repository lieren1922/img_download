use std::time::Instant;

use img_download::GalleryError;
use img_download::download_gallery;

use clap::Parser;

#[derive(Parser)]
#[command(name = "img_download")]
#[command(author = "Pipe_2U")]
#[command(version = "0.1")]
struct Args {
    #[arg(short, long)]
    url: String,

    #[arg(short, long, default_value = "downloaded_images")]
    directories: String,
}

// let url = "https://meizi1.com/12304.html";
// let save_dir = "downloaded_images";

#[tokio::main]
async fn main() -> Result<(), GalleryError> {
    let args = Args::parse();

    // 记录开始时间
    let start = Instant::now();

    match download_gallery(args.url, args.directories).await {
        Ok(_) => println!("Download completed successfully."),
        Err(e) => match e {
            GalleryError::RequestFailed => eprintln!("RequestFailed"),
            GalleryError::ParseError => eprintln!("ParseError"),
            GalleryError::IoError(_error) => eprintln!("IoError"),
            GalleryError::BrowserError(_) => eprintln!("BrowserError"),
            GalleryError::ReqwestError(_error) => eprintln!("ReqwestError"),
            GalleryError::TokioJoinError(_join_error) => eprintln!("TokioJoinError"),
        },
    }

    // 计算耗时
    let duration = start.elapsed();
    // 输出结果（自动选择合适单位）
    println!("耗时: {} 毫秒", duration.as_millis());

    Ok(())
}

// fn main() -> Result<(), GalleryError> {
//     let args = Args::parse();

//     // 记录开始时间
//     let start = Instant::now();

//     let _res = download_gallery(args.url, args.save_dir);

//     // 计算耗时
//     let duration = start.elapsed();
//     // 输出结果（自动选择合适单位）
//     println!("耗时: {} 毫秒", duration.as_millis());

//     println!("All images downloaded successfully!");
//     Ok(())
// }
