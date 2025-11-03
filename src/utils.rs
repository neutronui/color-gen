pub fn hyperlink(link: impl core::fmt::Display, text: impl core::fmt::Display) -> String {
  format!("\x1b]8;;{link}\x1b\\{text}\x1b]8;;\x1b\\")
}