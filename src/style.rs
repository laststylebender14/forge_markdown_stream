/// Trait for styling inline elements.
pub trait InlineStyler {
    fn text(&self, text: &str) -> String;
    fn bold(&self, text: &str) -> String;
    fn italic(&self, text: &str) -> String;
    fn bold_italic(&self, text: &str) -> String;
    fn strikethrough(&self, text: &str) -> String;
    fn underline(&self, text: &str) -> String;
    fn code(&self, text: &str) -> String;
    fn link(&self, text: &str, url: &str) -> String;
    fn image(&self, alt: &str, url: &str) -> String;
    fn footnote(&self, text: &str) -> String;
}
