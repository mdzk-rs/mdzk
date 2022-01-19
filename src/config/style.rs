use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug)]
#[serde(default, rename_all = "kebab-case")]
pub struct StyleConfig {
    fg: String,
    bg: String,
    links: String,
    sidebar_width: String,
    page_padding: String,
    content_max_width: String,
    menu_bar_height: String,
}

impl StyleConfig {
    pub fn css_bytes(&self) -> Vec<u8> {
        let mut css = ":root {\n".to_owned();
        css.push_str(&format!("--fg: {};\n", self.fg));
        css.push_str(&format!("--bg: {};\n", self.bg));
        css.push_str(&format!("--links: {};\n", self.links));

        css.push_str(&format!("--sidebar-width: {};\n", self.sidebar_width));
        css.push_str(&format!("--page-padding: {};\n", self.page_padding));
        css.push_str(&format!("--content-max-width: {};\n", self.content_max_width));
        css.push_str(&format!("--menu-bar-height: {};\n", self.menu_bar_height));
        css.push_str("}");
        css.as_bytes().to_owned()
    }
}

impl Default for StyleConfig {
    fn default() -> Self {
        Self {
            fg: "#000".to_owned(),
            bg: "#fff".to_owned(),
            links: "#008cff".to_owned(),
            sidebar_width: "300px".to_owned(),
            page_padding: "15px".to_owned(),
            content_max_width: "800px".to_owned(),
            menu_bar_height: "70px".to_owned(),
        }
    }
}
