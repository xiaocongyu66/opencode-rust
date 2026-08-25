use std::io::{self, Write};

pub struct Renderer {
    pub is_destroyed: bool,
}

impl Renderer {
    pub fn set_terminal_title(&self, title: &str) {
        print!("\x1b]2;{}\x07", title);
        let _ = io::stdout().flush();
    }

    pub fn destroy(&mut self) {
        self.is_destroyed = true;
    }
}

pub fn destroy_renderer(renderer: &mut Renderer) {
    renderer.set_terminal_title("");
    if renderer.is_destroyed {
        return;
    }
    renderer.destroy();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_destroy_renderer() {
        let mut r = Renderer { is_destroyed: false };
        destroy_renderer(&mut r);
        assert!(r.is_destroyed);
    }

    #[test]
    fn test_destroy_already_destroyed() {
        let mut r = Renderer { is_destroyed: true };
        destroy_renderer(&mut r);
        assert!(r.is_destroyed);
    }
}
