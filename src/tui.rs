use cursive::{CursiveRunnable, Rect, Vec2, View};
use cursive::reexports::enumset::EnumSet;
use cursive::theme::{Color, ColorStyle, PaletteColor, Style};
use cursive::utils::span::SpannedString;
use cursive::view::{Resizable, SizeConstraint};
use cursive::views::{FixedLayout, Layer, OnLayoutView, TextContent, TextView};

use crate::cli::Options;
use crate::util::duration_to_timer_string;


#[derive(Debug)]
pub struct InfoBarItem {
    pub key: String,
    pub description: String,
}

impl InfoBarItem {
    pub fn new(key: &str, description: &str) -> Self {
        InfoBarItem {
            key: key.to_string(),
            description: description.to_string(),
        }
    }
}

pub fn info_bar_text(items: &[InfoBarItem]) -> SpannedString<Style> {
    // Create styled text for the bottom info bar
    let hot_key_style = Style {
        effects: EnumSet::empty(),
        color: ColorStyle::front(Color::Rgb(200, 50, 200)),
    };

    let mut info_bar_text = SpannedString::new();
    if items.len() == 0 {
        return info_bar_text;
    } 
    
    let last_idx = items.len() - 1;

    for (idx, item) in items.iter().enumerate() {
        info_bar_text.append_plain(" (");
        info_bar_text.append_styled(&item.key, hot_key_style);
        let desc_str = if idx == last_idx {
            format!(") {}", item.description)
        } else {
            format!(") {} |", item.description)
        };
        info_bar_text.append_plain(desc_str);
    }

    info_bar_text
}

pub fn init_curses_app(options: &Options) -> (CursiveRunnable, Color, TextContent) {
    let mut curses_app = cursive::default();

    curses_app.load_toml(include_str!("theme.toml")).unwrap();
    let bg_color = curses_app.current_theme().clone().palette[PaletteColor::Background];

    // Create info bar content
    let info_bar_items = vec![
        InfoBarItem::new("s", "start timer"),
        InfoBarItem::new("p", "pause toggle"),
        InfoBarItem::new("r", "reset timer"),
        InfoBarItem::new("q", "quit"),
        InfoBarItem::new("↑↓", "add / sub time"),
    ];
    let info_bar_text = info_bar_text(&info_bar_items);

    // Create an info bar at the bottom of the app to display controls
    curses_app.screen_mut().add_transparent_layer(
        OnLayoutView::new(
            FixedLayout::new().child(
                Rect::from_point(Vec2::zero()),
                Layer::new(TextView::new(info_bar_text))
                    .full_width(),
            ),
            |layout, size| {
                layout.set_child_position(
                    0,
                    Rect::from_size((0, size.y - 1), (size.x, 1)),
                );
                layout.layout(size);
            },
        ).full_screen(),
    );

    // Create a separate TextContent and add it to a TextView so we can keep a
    // reference to it and update it without retrieving the TextView from the
    // cursive app by name.
    let timer_content = TextContent::new(duration_to_timer_string(&options.duration));
    curses_app.add_layer(TextView::new_with_content(timer_content.clone())
        .center()
        .resized(SizeConstraint::Fixed(12), SizeConstraint::Fixed(3)));

    curses_app.set_autorefresh(true);

    (curses_app, bg_color, timer_content)
}
