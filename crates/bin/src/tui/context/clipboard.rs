use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct ClipboardContent {
    pub data: String,
    pub mime: String,
}

pub trait ClipboardService: Send + Sync {
    fn read(&self) -> Option<ClipboardContent> {
        None
    }
    fn write(&self, _text: &str) {}
}

pub type ClipboardHandle = Arc<dyn ClipboardService>;

pub struct DefaultClipboard;

impl ClipboardService for DefaultClipboard {}

pub fn default_clipboard() -> ClipboardHandle {
    Arc::new(DefaultClipboard) as ClipboardHandle
}
