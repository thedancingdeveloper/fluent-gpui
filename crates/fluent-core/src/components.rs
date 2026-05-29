/// Component geometry and interaction-size defaults.
///
/// These values define the framework's Fluent/Microsoft-app style density:
/// compact command surfaces, 28px menu rows, 32px form controls, and stable
/// icon gutters for aligned menus and popups.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ComponentTokens {
    pub menu_bar_height: f32,
    pub menu_item_height: f32,
    pub menu_min_width: f32,
    pub popup_icon_slot: f32,
    pub popup_separator_height: f32,
    pub dropdown_height: f32,
    pub dropdown_option_height: f32,
    pub dropdown_min_width: f32,
    pub dropdown_max_height: f32,
    pub text_input_height: f32,
    pub text_input_focus_indicator_height: f32,
    pub dock_header_height: f32,
    pub dock_action_size: f32,
    pub settings_nav_width: f32,
    pub settings_nav_section_height: f32,
    pub settings_nav_item_height: f32,
    pub settings_nav_accent_width: f32,
    pub settings_nav_item_indent: f32,
    pub command_palette_search_height: f32,
    pub command_palette_footer_height: f32,
    pub command_palette_icon_slot: f32,
    pub content_tab_strip_height: f32,
    pub content_tab_icon_size: f32,
    pub content_tab_close_size: f32,
    pub ribbon_tab_height: f32,
    pub ribbon_content_button_height: f32,
    pub ribbon_group_label_height: f32,
    pub ribbon_large_icon_size: f32,
    pub ribbon_small_icon_size: f32,
    pub ribbon_contextual_band_height: f32,
}

impl Default for ComponentTokens {
    fn default() -> Self {
        Self {
            menu_bar_height: 28.0,
            menu_item_height: 28.0,
            menu_min_width: 200.0,
            popup_icon_slot: 16.0,
            popup_separator_height: 1.0,
            dropdown_height: 32.0,
            dropdown_option_height: 28.0,
            dropdown_min_width: 180.0,
            dropdown_max_height: 240.0,
            text_input_height: 32.0,
            text_input_focus_indicator_height: 2.0,
            dock_header_height: 28.0,
            dock_action_size: 20.0,
            settings_nav_width: 200.0,
            settings_nav_section_height: 28.0,
            settings_nav_item_height: 32.0,
            settings_nav_accent_width: 3.0,
            settings_nav_item_indent: 28.0,
            command_palette_search_height: 44.0,
            command_palette_footer_height: 28.0,
            command_palette_icon_slot: 16.0,
            content_tab_strip_height: 32.0,
            content_tab_icon_size: 14.0,
            content_tab_close_size: 14.0,
            ribbon_tab_height: 28.0,
            ribbon_content_button_height: 64.0,
            ribbon_group_label_height: 16.0,
            ribbon_large_icon_size: 28.0,
            ribbon_small_icon_size: 16.0,
            ribbon_contextual_band_height: 3.0,
        }
    }
}
