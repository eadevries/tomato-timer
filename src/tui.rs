use cursive::reexports::enumset::EnumSet;
use cursive::theme::{Color, ColorStyle, Style};
use cursive::utils::span::SpannedString;


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
