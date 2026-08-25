pub trait ClipboardService {
    fn write(&self, text: &str) -> Result<(), String>;
}

pub trait Toast {
    fn show(&self, message: &str, variant: ToastVariant);
    fn error(&self, err: &dyn std::error::Error);
}

#[derive(Clone, Copy, Debug)]
pub enum ToastVariant {
    Info,
    Success,
    Warning,
    Error,
}

pub trait FocusableSelectionTarget {
    fn has_selection(&self) -> bool;
    fn get_clipboard_text(&self, text: &str) -> Option<String>;
}

pub struct SelectionInfo {
    pub selected_text: String,
    pub selected_renderables: Vec<Box<dyn FocusableSelectionTarget>>,
}

pub trait Renderer {
    fn get_selection(&self) -> Option<SelectionInfo>;
    fn clear_selection(&mut self);
    fn current_focused_renderable(&self) -> Option<&dyn FocusableSelectionTarget>;
}

#[derive(Clone, Debug)]
pub struct SelectionKeyEvent {
    pub ctrl: bool,
    pub name: String,
}

pub fn copy<R: Renderer, T: Toast, C: ClipboardService>(
    renderer: &mut R,
    toast: &T,
    clipboard: &C,
) -> bool {
    let selection = match renderer.get_selection() {
        Some(s) => s,
        None => return false,
    };
    if selection.selected_text.is_empty() {
        return false;
    }
    let clipboard_text = selection.selected_text.clone();
    match clipboard.write(&clipboard_text) {
        Ok(()) => {
            toast.show("Copied to clipboard", ToastVariant::Info);
            renderer.clear_selection();
            true
        }
        Err(e) => {
            toast.error(&std::io::Error::new(std::io::ErrorKind::Other, e) as &dyn std::error::Error);
            renderer.clear_selection();
            false
        }
    }
}

pub fn handle_selection_key<R: Renderer, T: Toast, C: ClipboardService>(
    renderer: &mut R,
    toast: &T,
    event: &SelectionKeyEvent,
    clipboard: &C,
) {
    if renderer.get_selection().is_none() {
        return;
    }
    if event.ctrl && event.name == "c" {
        if !copy(renderer, toast, clipboard) {
            renderer.clear_selection();
        }
        return;
    }
    if event.name == "escape" {
        renderer.clear_selection();
        return;
    }
    let has_focus_selection = renderer
        .current_focused_renderable()
        .map(|f| f.has_selection())
        .unwrap_or(false);
    if has_focus_selection {
        return;
    }
    renderer.clear_selection();
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;

    struct MockClipboard {
        written: RefCell<String>,
    }
    impl ClipboardService for MockClipboard {
        fn write(&self, text: &str) -> Result<(), String> {
            *self.written.borrow_mut() = text.to_string();
            Ok(())
        }
    }

    struct MockToast {
        message: RefCell<String>,
    }
    impl Toast for MockToast {
        fn show(&self, message: &str, _variant: ToastVariant) {
            *self.message.borrow_mut() = message.to_string();
        }
        fn error(&self, _err: &dyn std::error::Error) {}
    }

    struct MockFocus;
    impl FocusableSelectionTarget for MockFocus {
        fn has_selection(&self) -> bool {
            false
        }
        fn get_clipboard_text(&self, text: &str) -> Option<String> {
            Some(text.to_string())
        }
    }

    struct MockRenderer {
        has_selection: bool,
        cleared: RefCell<bool>,
    }
    impl Renderer for MockRenderer {
        fn get_selection(&self) -> Option<SelectionInfo> {
            if self.has_selection {
                Some(SelectionInfo {
                    selected_text: "test".to_string(),
                    selected_renderables: vec![],
                })
            } else {
                None
            }
        }
        fn clear_selection(&mut self) {
            *self.cleared.borrow_mut() = true;
        }
        fn current_focused_renderable(&self) -> Option<&dyn FocusableSelectionTarget> {
            None
        }
    }

    #[test]
    fn test_copy() {
        let clipboard = MockClipboard {
            written: RefCell::new(String::new()),
        };
        let toast = MockToast {
            message: RefCell::new(String::new()),
        };
        let mut renderer = MockRenderer {
            has_selection: true,
            cleared: RefCell::new(false),
        };
        assert!(copy(&mut renderer, &toast, &clipboard));
        assert_eq!(*clipboard.written.borrow(), "test");
        assert_eq!(*toast.message.borrow(), "Copied to clipboard");
        assert!(*renderer.cleared.borrow());
    }

    #[test]
    fn test_copy_no_selection() {
        let clipboard = MockClipboard {
            written: RefCell::new(String::new()),
        };
        let toast = MockToast {
            message: RefCell::new(String::new()),
        };
        let mut renderer = MockRenderer {
            has_selection: false,
            cleared: RefCell::new(false),
        };
        assert!(!copy(&mut renderer, &toast, &clipboard));
    }

    #[test]
    fn test_escape_clears() {
        let clipboard = MockClipboard {
            written: RefCell::new(String::new()),
        };
        let toast = MockToast {
            message: RefCell::new(String::new()),
        };
        let mut renderer = MockRenderer {
            has_selection: true,
            cleared: RefCell::new(false),
        };
        handle_selection_key(
            &mut renderer,
            &toast,
            &SelectionKeyEvent {
                ctrl: false,
                name: "escape".to_string(),
            },
            &clipboard,
        );
        assert!(*renderer.cleared.borrow());
    }
}
