use std::{error::Error, sync::Arc};
use base64::Engine;
use headless_chrome::protocol::cdp::Page;
use headless_chrome::protocol::cdp::Page::Viewport;
use headless_chrome::{Browser, Tab};

#[allow(dead_code)]
pub struct HtmlRenderer {
    browser: Browser,
    tab: Arc<Tab>,
}

impl HtmlRenderer {
    pub fn new() -> Result<Self, Box<dyn Error>> {
        let browser = Browser::default()?;
        let tab = browser.new_tab()?;
        tab.set_default_timeout(std::time::Duration::from_secs(10));
        Ok(Self { browser, tab })
    }

    pub fn render(&self, html: &str) -> Result<Vec<u8>, Box<dyn Error>> {
        self.render_with_size(html, None)
    }

    pub fn render_with_size(
        &self,
        html: &str,
        viewport_size: Option<(u32, u32)>,
    ) -> Result<Vec<u8>, Box<dyn Error>> {
        let b64 = base64::engine::general_purpose::STANDARD.encode(html);
        let data_url = format!("data:text/html;base64,{}", b64);

        self.tab.navigate_to(&data_url)?;
        self.tab.wait_until_navigated()?;
        self.tab.wait_for_element("body")?;

        let clip = match viewport_size {
            Some((w, h)) => Viewport {
                x: 0.0,
                y: 0.0,
                width: w as f64,
                height: h as f64,
                scale: 1.0,
            },
            None => {
                let (w, h) = self.get_page_size()?;
                Viewport {
                    x: 0.0,
                    y: 0.0,
                    width: w as f64,
                    height: h as f64,
                    scale: 1.0,
                }
            }
        };
        println!("{:?}", clip);

        self.tab.wait_for_element("body")?;
        std::thread::sleep(std::time::Duration::from_millis(500));

        let screenshot = self.tab.capture_screenshot(
            Page::CaptureScreenshotFormatOption::Png,
            None,
            Some(clip),
            true,
        )?;

        Ok(screenshot)
    }

    fn get_page_size(&self) -> Result<(u32, u32), Box<dyn Error>> {
        let width_js = r#"Math.max(
            document.body.scrollWidth,
            document.documentElement.scrollWidth,
            document.body.offsetWidth,
            document.documentElement.offsetWidth,
            document.body.clientWidth,
            document.documentElement.clientWidth
        )"#;

        let height_js = r#"Math.max(
            document.body.scrollHeight,
            document.documentElement.scrollHeight,
            document.body.offsetHeight,
            document.documentElement.offsetHeight,
            document.body.clientHeight,
            document.documentElement.clientHeight
        )"#;

        let width = self.tab
            .evaluate(width_js, true)?
            .value
            .and_then(|v| v.as_f64())
            .ok_or("Failed to get width")? as u32;

        let height = self.tab
            .evaluate(height_js, true)?
            .value
            .and_then(|v| v.as_f64())
            .ok_or("Failed to get height")? as u32;

        Ok((width, height))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_auto_size_rendering() {
        let renderer = HtmlRenderer::new().unwrap();
        let html = r#"<html><body style="width: 80px; height: 600px; margin: 0px; border: 0px; background-color: #FF0000;"><h1>Test</h1></body></html>"#;
        let image = renderer.render(html).unwrap();
        assert!(!image.is_empty());
        std::fs::write("auto_size.png", image).unwrap();
    }

    #[test]
    fn test_manual_size_rendering() {
        let renderer = HtmlRenderer::new().unwrap();
        let html = r#"<html><body><h1>Test</h1></body></html>"#;
        let image = renderer.render_with_size(html, Some((1024, 768))).unwrap();
        assert!(!image.is_empty());
        std::fs::write("manual_size.png", image).unwrap();
    }
}
