//! Hello, World! written with the builders of `ferroui-view` (docs/declarative-ui.md, section 4):
//! the text and a button that counts its clicks, the count a signal (section 6) that the text of
//! the button follows.

use ferroui_base::layout::{HorizontalAlignment, VerticalAlignment};
use ferroui_view::prelude::*;

/// The content of the window.
pub fn hello() -> Element {
    let clicks = signal(0);
    stack_panel()
        .horizontal_alignment(HorizontalAlignment::Center)
        .vertical_alignment(VerticalAlignment::Center)
        .spacing(12.0)
        .child(text_block().text("Hello, World!").font_size(28.0))
        .child(
            button()
                .name("Count")
                .horizontal_alignment(HorizontalAlignment::Center)
                .on_click(move |_| clicks.update(|c| *c += 1))
                .child(text_block().text(move || format!("Clicked {} times", clicks.get()))),
        )
        .into()
}
