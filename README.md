# img_download - Asynchronous Image Crawler

A fast asynchronous image crawler written in Rust that downloads images from a specified webpage.

## Features

- Asynchronous design for efficient downloading
- Simple command-line interface
- Customizable download directory
- Easy to use with minimal configuration

## Installation

```bash
# Install from source
cargo install --path .

# Or download precompiled binaries from the releases page
```

## Usage

```bash
img_download [OPTIONS] --url <URL>
```

### Options

| Option | Short | Description | Default |
|--------|-------|-------------|---------|
| --url <URL> | -u | The URL of the webpage to crawl for images (required) | None |
| --directories <DIRECTORIES> | -d | The directory to save downloaded images | downloaded_images |
| --help | -h | Print help information | - |
| --version | -V | Print version information | - |

## Examples

### Basic usage

```bash
img_download -u https://example.com
```

This will download all images from `https://example.com` to the default `downloaded_images` directory.

### Custom directory

```bash
img_download -u https://example.com -d my_images
```

This will download all images from `https://example.com` to the `my_images` directory.

## How it works

1. The crawler fetches the specified webpage
2. It parses the HTML to find all image tags (`<img>`)
3. It extracts the image URLs from the `src` attribute
4. It downloads all found images asynchronously to the specified directory
5. If the directory doesn't exist, it will be created automatically

## Dependencies

- tokio - For asynchronous runtime
- reqwest - For HTTP requests
- scraper - For HTML parsing
- clap - For command-line argument parsing

## License

MIT
