use ratatui::layout::{Constraint, Layout, Rect};

pub fn split_vertical(area: Rect, constraints: Vec<Constraint>) -> Vec<Rect> {
    Layout::vertical(constraints).split(area).to_vec()
}

pub fn split_horizontal(area: Rect, constraints: Vec<Constraint>) -> Vec<Rect> {
    Layout::horizontal(constraints).split(area).to_vec()
}

pub fn margin_top(area: Rect, margin: u16) -> Rect {
    if margin == 0 {
        return area;
    }
    let y = area.y.saturating_add(margin);
    let height = area.height.saturating_sub(margin);
    Rect {
        x: area.x,
        y,
        width: area.width,
        height,
    }
}

pub fn margin_bottom(area: Rect, margin: u16) -> Rect {
    if margin == 0 {
        return area;
    }
    let height = area.height.saturating_sub(margin);
    Rect {
        x: area.x,
        y: area.y,
        width: area.width,
        height,
    }
}

pub fn padding(area: Rect, top: u16, right: u16, bottom: u16, left: u16) -> Rect {
    let y = area.y.saturating_add(top);
    let height = area
        .height
        .saturating_sub(top)
        .saturating_sub(bottom);
    let x = area.x.saturating_add(left);
    let width = area
        .width
        .saturating_sub(left)
        .saturating_sub(right);
    Rect { x, y, width, height }
}

pub fn constrain_to(area: Rect, max_width: Option<u16>, max_height: Option<u16>) -> Rect {
    let width = max_width.map(|m| area.width.min(m)).unwrap_or(area.width);
    let height = max_height.map(|m| area.height.min(m)).unwrap_or(area.height);
    Rect {
        x: area.x,
        y: area.y,
        width,
        height,
    }
}

pub fn center(area: Rect, width: u16, height: u16) -> Rect {
    let x = area.x + (area.width.saturating_sub(width)) / 2;
    let y = area.y + (area.height.saturating_sub(height)) / 2;
    Rect { x, y, width, height }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_split_vertical() {
        let area = Rect::new(0, 0, 80, 24);
        let rects = split_vertical(area, vec![Constraint::Length(10), Constraint::Min(0)]);
        assert_eq!(rects.len(), 2);
        assert_eq!(rects[0].height, 10);
        assert_eq!(rects[1].height, 14);
    }

    #[test]
    fn test_split_horizontal() {
        let area = Rect::new(0, 0, 80, 24);
        let rects = split_horizontal(area, vec![Constraint::Length(20), Constraint::Min(0)]);
        assert_eq!(rects.len(), 2);
        assert_eq!(rects[0].width, 20);
        assert_eq!(rects[1].width, 60);
    }

    #[test]
    fn test_margin_top() {
        let area = Rect::new(0, 0, 80, 24);
        let result = margin_top(area, 2);
        assert_eq!(result.y, 2);
        assert_eq!(result.height, 22);
    }

    #[test]
    fn test_center() {
        let area = Rect::new(0, 0, 80, 24);
        let result = center(area, 40, 10);
        assert_eq!(result.x, 20);
        assert_eq!(result.y, 7);
        assert_eq!(result.width, 40);
        assert_eq!(result.height, 10);
    }

    #[test]
    fn test_padding() {
        let area = Rect::new(0, 0, 80, 24);
        let result = padding(area, 1, 2, 3, 4);
        assert_eq!(result.x, 4);
        assert_eq!(result.y, 1);
        assert_eq!(result.width, 74);
        assert_eq!(result.height, 20);
    }
}
