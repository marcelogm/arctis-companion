pub fn line_at(text: &str, offset: usize) -> usize {
    text[..offset].matches('\n').count() + 1
}
