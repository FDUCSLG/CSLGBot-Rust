use std::{error::Error, sync::Arc};
use base64::Engine;
use headless_chrome::protocol::cdp::Page;
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
        let b64 = base64::engine::general_purpose::STANDARD.encode(html);
        let data_url = format!("data:text/html;base64,{}", b64);
        
        self.tab.navigate_to(&data_url)?;
        self.tab.wait_until_navigated()?;
        self.tab.wait_for_element("body")?;
        
        std::thread::sleep(std::time::Duration::from_millis(500));
        
        let screenshot = self.tab.capture_screenshot(
            Page::CaptureScreenshotFormatOption::Png,
            None, None, true
        )?;

        Ok(screenshot)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_html_rendering() {
        let renderer = HtmlRenderer::new().unwrap();
        
        let test_html = r#"
            <!DOCTYPE html>
            <html>
                <head>
                    <style>
                        .container {
                            width: 600px;
                            margin: 0px auto;
                            padding: 20px;
                        }
                        h1 { color: #ff4444; }
                        p { font-family: Arial, sans-serif; }
                    </style>
                </head>
                <body>
                    <div class="container">
                        <h1>HTML to Image Test</h1>
                        <p>This is a complex HTML content with:</p>
                        <ul>
                            <li>Styled elements</li>
                            <li>CSS formatting</li>
                            <li>Layout structure</li>
                        </ul>
                        <table>
                            <tr><th>Header 1</th><th>Header 2</th></tr>
                            <tr><td>Data 1</td><td>Data 2</td></tr>
                        </table>
                    </div>
                </body>
            </html>
        "#;

        let image_data = renderer.render(test_html).unwrap();
        assert!(!image_data.is_empty());
        std::fs::write("test.png", image_data).unwrap();
    }
}
