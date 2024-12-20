use image::DynamicImage;
use kurec_interface::KurecConfig;
use webp::{Encoder, WebPMemory};
use webpage::{Webpage, WebpageOptions};

pub struct OgpAdapter {
    pub config: KurecConfig,
}

impl OgpAdapter {
    pub fn new(config: KurecConfig) -> Self {
        Self { config }
    }

    pub async fn get_ogp_image_urls(&self, url: &str) -> Result<Vec<String>, anyhow::Error> {
        let info = Webpage::from_url(url, WebpageOptions::default())?;
        let images = info
            .html
            .opengraph
            .images
            .iter()
            .map(|i| i.url.clone())
            .collect::<Vec<_>>();
        Ok(images)
    }

    pub async fn get_image_from_url(&self, url: &str) -> Option<DynamicImage> {
        let result = reqwest::Client::new().get(url).send().await;
        let resp = match result {
            Ok(resp) => resp,
            Err(_) => return None,
        };
        let bytes = match resp.bytes().await {
            Ok(bytes) => bytes,
            Err(_) => return None,
        };
        let img = match image::load_from_memory(&bytes) {
            Ok(img) => img,
            Err(_) => return None,
        };
        Some(img)
    }

    pub fn reseize_image(&self, img: DynamicImage, width: u32, height: u32) -> DynamicImage {
        img.resize(width, height, image::imageops::FilterType::Lanczos3)
    }

    pub fn get_webp_from_image(&self, img: DynamicImage, quality: f32) -> WebPMemory {
        let encoder: Encoder = Encoder::from_image(&img).unwrap();
        encoder.encode(quality)
    }

    pub async fn get_resized_ogp_webps_from_url(
        &self,
        url: &str,
        width: u32,
    ) -> Result<Option<WebPMemory>, anyhow::Error> {
        let images = self.get_ogp_image_urls(url).await?;
        if images.is_empty() {
            return Ok(None);
        }
        let img = match self.get_image_from_url(&images[0]).await {
            Some(img) => img,
            None => {
                return Ok(None);
            }
        };

        let resize_height = img.height() * width / img.width();
        let resized_img = self.reseize_image(img, width, resize_height);
        let webp = self.get_webp_from_image(resized_img, 75.0);

        Ok(Some(webp))
    }
}
