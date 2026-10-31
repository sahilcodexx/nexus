use gpui::{
    div, img, prelude::*, px, rgb, rgba, size, App, Application, Bounds, Context, FocusHandle,
    FontWeight, KeyDownEvent, MouseButton, MouseDownEvent, ObjectFit, Render, TitlebarOptions,
    Window, WindowBackgroundAppearance, WindowBounds, WindowDecorations, WindowKind, WindowOptions,
};
use nexus_storage::{ClipboardEntry, ContentType, SqliteStorage};
use nexus_ui::{
    get_emoji_list, get_gif_list, get_images_cache_dir, get_kaomoji_list, get_symbol_list,
    get_system_clipboard_text, relative_time_string, resolve_emojic_shortcode, NexusState,
    ThemeMode, ViewMode,
};
use std::path::PathBuf;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

/// Color palette for Windows 11 Fluent interface with modern acrylic/glassmorphism
#[derive(Clone, Copy)]
struct Palette {
    window_bg: gpui::Rgba,
    window_border: gpui::Rgba,
    header_text: gpui::Rgba,
    text_primary: gpui::Rgba,
    text_secondary: gpui::Rgba,
    text_muted: gpui::Rgba,
    accent_blue: gpui::Rgba,
    nav_icon_idle: gpui::Rgba,
    nav_icon_hover: gpui::Rgba,
    nav_indicator: gpui::Rgba,
    card_bg: gpui::Rgba,
    card_border: gpui::Rgba,
    card_hover_border: gpui::Rgba,
    card_hover_bg: gpui::Rgba,
    search_bg: gpui::Rgba,
    search_border: gpui::Rgba,
    button_bg: gpui::Rgba,
    button_border: gpui::Rgba,
    button_hover: gpui::Rgba,
    pin_active_bg: gpui::Rgba,
    pin_active_text: gpui::Rgba,
    #[allow(dead_code)]
    drag_handle: gpui::Rgba,
    pill_active_bg: gpui::Rgba,
    pill_active_text: gpui::Rgba,
    pill_idle_bg: gpui::Rgba,
    pill_idle_text: gpui::Rgba,
}

impl Palette {
    /// Windows 11 Fluent Light theme with 96.5% background glass opacity
    fn light() -> Self {
        Self {
            window_bg: rgba(0xf4f4f7f6),        // 96.5% opacity (0xf6 / 255 = 0.965)
            window_border: rgba(0xd4d4dee8),    // 91% opacity crisp border
            header_text: rgb(0x1a1a1a),
            text_primary: rgb(0x1a1a1a),
            text_secondary: rgb(0x595962),
            text_muted: rgb(0x8a8a93),
            accent_blue: rgb(0x0067c0),
            nav_icon_idle: rgb(0x3c3c43),
            nav_icon_hover: rgba(0xe5e5ebcc),
            nav_indicator: rgb(0x0067c0),
            card_bg: rgba(0xfffffffa),          // 98% opacity high contrast card
            card_border: rgba(0xdcdce6ee),
            card_hover_border: rgb(0x0067c0),
            card_hover_bg: rgba(0xffffffff),
            search_bg: rgba(0xfffffffa),
            search_border: rgba(0xd4d4dcee),
            button_bg: rgba(0xe8e8f0f0),
            button_border: rgba(0xd0d0d8ee),
            button_hover: rgba(0xdedee6fa),
            pin_active_bg: rgb(0xe0eefc),
            pin_active_text: rgb(0x0067c0),
            drag_handle: rgba(0xb0b0ba99),
            pill_active_bg: rgb(0x0067c0),
            pill_active_text: rgb(0xffffff),
            pill_idle_bg: rgba(0xe6e6edf0),
            pill_idle_text: rgb(0x404046),
        }
    }

    /// Windows 11 Fluent Dark theme with 96.5% background glass opacity
    fn dark() -> Self {
        Self {
            window_bg: rgba(0x202024f6),        // 96.5% opacity (0xf6 / 255 = 0.965)
            window_border: rgba(0x404047e8),    // 91% opacity crisp border
            header_text: rgb(0xffffff),
            text_primary: rgb(0xf4f4f6),
            text_secondary: rgb(0xaaaab2),
            text_muted: rgb(0x73737a),
            accent_blue: rgb(0x60cdff),
            nav_icon_idle: rgb(0xd0d0d6),
            nav_icon_hover: rgba(0x323238cc),
            nav_indicator: rgb(0x60cdff),
            card_bg: rgba(0x2b2b31fa),          // 98% opacity high contrast card
            card_border: rgba(0x3e3e46ee),
            card_hover_border: rgb(0x60cdff),
            card_hover_bg: rgba(0x383840ff),
            search_bg: rgba(0x2b2b31fa),
            search_border: rgba(0x464650ee),
            button_bg: rgba(0x303036f0),
            button_border: rgba(0x464650ee),
            button_hover: rgba(0x3c3c44fa),
            pin_active_bg: rgb(0x19334d),
            pin_active_text: rgb(0x60cdff),
            drag_handle: rgba(0x5a5a6299),
            pill_active_bg: rgb(0x60cdff),
            pill_active_text: rgb(0x1a1a1a),
            pill_idle_bg: rgba(0x2e2e34f0),
            pill_idle_text: rgb(0xd0d0d6),
        }
    }
}

/// The main Windows 11-style Clipboard & Input flyout panel view
struct NexusPanel {
    state: NexusState,
    focus_handle: FocusHandle,
}

impl NexusPanel {
    fn new(cx: &mut Context<Self>, window: &mut Window, initial_mode: ViewMode) -> Self {
        let mut state = NexusState::new();
        state.set_view_mode(initial_mode);
        let focus_handle = cx.focus_handle();
        focus_handle.focus(window);

        // Active background timer to immediately catch any newly copied items while the window is open (150ms check)
        let mut async_cx = cx.to_async();
        let this = cx.weak_entity();
        let sync_task = cx.foreground_executor().spawn(async move {
            loop {
                async_cx
                    .background_executor()
                    .timer(std::time::Duration::from_millis(150))
                    .await;
                let update_res = this.update(&mut async_cx, |panel, cx| {
                    if panel.state.check_for_database_updates() {
                        cx.notify();
                    }
                });
                if update_res.is_err() {
                    break;
                }
            }
        });
        sync_task.detach();

        Self {
            state,
            focus_handle,
        }
    }

    fn palette(&self) -> Palette {
        match self.state.theme {
            ThemeMode::Light => Palette::light(),
            ThemeMode::Dark => Palette::dark(),
        }
    }
}

impl Render for NexusPanel {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let palette = self.palette();
        self.focus_handle.focus(window);

        // Check if there is new content in the Linux system clipboard (live sync text & images)
        self.state.check_system_clipboard_sync();

        div()
            .flex()
            .flex_col()
            .size_full()
            .bg(palette.window_bg)
            .border_1()
            .border_color(palette.window_border)
            .rounded_xl()
            .overflow_hidden()
            .track_focus(&self.focus_handle)
            .on_mouse_down(
                MouseButton::Left,
                cx.listener(|this, _: &MouseDownEvent, window, _cx| {
                    this.focus_handle.focus(window);
                }),
            )
            .on_key_down(cx.listener(|this, event: &KeyDownEvent, _window, cx| {
                let key = event.keystroke.key.as_str();
                match key {
                    "escape" => {
                        // Esc always closes the app immediately, even if touched/clicked anywhere inside
                        cx.quit();
                    }
                    "backspace" => {
                        if !this.state.search_query.is_empty() {
                            this.state.search_query.pop();
                            cx.notify();
                        }
                    }
                    k if k.chars().count() == 1 => {
                        let c = k.chars().next().unwrap();
                        if !c.is_control() {
                            this.state.search_query.push(c);
                            cx.notify();
                        }
                    }
                    _ => {}
                }
            }))
            .child(self.render_navigation_header(palette, cx))
            .child(self.render_sub_header(palette, cx))
            .child(self.render_search_bar(palette, cx))
            .child(self.render_content(palette, cx))
            .child(self.render_footer(palette, cx))
    }
}

impl NexusPanel {
    /// Unified navigation header bar:
    /// - Left: 6 Fluent tabs ([📋 Clipboard] [😊 Emoji] [GIF] [;-) Kaomoji] [% Symbols] [♡ Favorites])
    /// - Right: Theme toggle (☀️/🌙) and Close button (✕)
    fn render_navigation_header(&self, palette: Palette, cx: &mut Context<Self>) -> impl IntoElement {
        let tabs = [
            (ViewMode::Clipboard, "📋", "Clipboard"),
            (ViewMode::Emoji, "😊", "Emoji"),
            (ViewMode::Gif, "GIF", "GIFs"),
            (ViewMode::Kaomoji, ";-)", "Kaomoji"),
            (ViewMode::Symbols, "% π", "Symbols"),
            (ViewMode::Favorites, "♡", "Favorites"),
        ];

        div()
            .flex()
            .items_center()
            .justify_between()
            .px_3()
            .pt_2p5()
            .pb_1p5()
            .border_b_1()
            .border_color(palette.card_border)
            .child(
                // Left: Navigation tabs with Windows 11 Fluent active underline
                div()
                    .flex()
                    .items_center()
                    .gap_1()
                    .children(tabs.into_iter().map(|(mode, icon, _title)| {
                        let is_active = self.state.view_mode == mode;

                        div()
                            .flex()
                            .flex_col()
                            .items_center()
                            .justify_center()
                            .px_2p5()
                            .py_1()
                            .rounded_md()
                            .cursor(gpui::CursorStyle::PointingHand)
                            .bg(if is_active {
                                palette.nav_icon_hover
                            } else {
                                rgba(0x00000000)
                            })
                            .hover(|s| s.bg(palette.nav_icon_hover))
                            .child(
                                div()
                                    .flex()
                                    .items_center()
                                    .justify_center()
                                    .h(px(22.0))
                                    .text_sm()
                                    .font_weight(if is_active {
                                        FontWeight::BOLD
                                    } else {
                                        FontWeight::NORMAL
                                    })
                                    .text_color(if is_active {
                                        palette.accent_blue
                                    } else {
                                        palette.nav_icon_idle
                                    })
                                    .child(icon),
                            )
                            // Windows 11 Active Indicator Underline Pill
                            .child(
                                div()
                                    .w(px(16.0))
                                    .h(px(2.5))
                                    .rounded_full()
                                    .bg(if is_active {
                                        palette.nav_indicator
                                    } else {
                                        rgba(0x00000000)
                                    }),
                            )
                            .on_mouse_down(
                                MouseButton::Left,
                                cx.listener(move |this, _: &MouseDownEvent, window, cx| {
                                    this.state.set_view_mode(mode);
                                    this.focus_handle.focus(window);
                                    cx.notify();
                                }),
                            )
                    })),
            )
            .child(
                // Right: Theme Toggle & Close button seamlessly integrated in header
                div()
                    .flex()
                    .items_center()
                    .gap_1()
                    .child(
                        // Dark / Light Theme switch button
                        div()
                            .flex()
                            .items_center()
                            .justify_center()
                            .w_7()
                            .h_7()
                            .rounded_md()
                            .cursor(gpui::CursorStyle::PointingHand)
                            .hover(|s| s.bg(palette.nav_icon_hover))
                            .text_sm()
                            .child(if self.state.theme == ThemeMode::Light {
                                "🌙"
                            } else {
                                "☀️"
                            })
                            .on_mouse_down(
                                MouseButton::Left,
                                cx.listener(|this, _: &MouseDownEvent, window, cx| {
                                    this.state.toggle_theme();
                                    this.focus_handle.focus(window);
                                    cx.notify();
                                }),
                            ),
                    )
                    .child(
                        // Close button (✕)
                        div()
                            .flex()
                            .items_center()
                            .justify_center()
                            .w_7()
                            .h_7()
                            .rounded_md()
                            .cursor(gpui::CursorStyle::PointingHand)
                            .hover(|s| s.bg(rgb(0xe81123)).text_color(rgb(0xffffff)))
                            .text_xs()
                            .text_color(palette.text_secondary)
                            .child("✕")
                            .on_mouse_down(
                                MouseButton::Left,
                                cx.listener(|_this, _: &MouseDownEvent, _window, cx| {
                                    cx.quit();
                                }),
                            ),
                    ),
            )
    }

    /// Sub-header matching Windows 11: shows Title ("Clipboard") and "Clear all" button only on Clipboard tab
    fn render_sub_header(&self, palette: Palette, cx: &mut Context<Self>) -> impl IntoElement {
        if self.state.view_mode != ViewMode::Clipboard {
            return div();
        }

        let total_count = self.state.clipboard_entries.len();

        div()
            .flex()
            .items_center()
            .justify_between()
            .px_4()
            .pt_2()
            .pb_1()
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_2()
                    .child(
                        div()
                            .text_sm()
                            .font_weight(FontWeight::BOLD)
                            .text_color(palette.header_text)
                            .child("Clipboard"),
                    )
                    .child(
                        div()
                            .text_xs()
                            .text_color(palette.text_muted)
                            .child(format!("{} items", total_count)),
                    ),
            )
            .child(
                // "Clear all" button
                div()
                    .px_2p5()
                    .py_1()
                    .rounded_md()
                    .bg(palette.button_bg)
                    .border_1()
                    .border_color(palette.button_border)
                    .hover(|s| s.bg(palette.button_hover))
                    .cursor(gpui::CursorStyle::PointingHand)
                    .text_xs()
                    .font_weight(FontWeight::MEDIUM)
                    .text_color(palette.text_secondary)
                    .child("Clear all")
                    .on_mouse_down(
                        MouseButton::Left,
                        cx.listener(|this, _: &MouseDownEvent, window, cx| {
                            this.state.clear_unpinned();
                            this.state.toast_message = Some("Cleared unpinned items".to_string());
                            this.focus_handle.focus(window);
                            cx.notify();
                        }),
                    ),
            )
    }

    /// Search input bar matching Windows 11 search styling
    fn render_search_bar(&self, palette: Palette, cx: &mut Context<Self>) -> impl IntoElement {
        let placeholder = match self.state.view_mode {
            ViewMode::Clipboard => "Search in Clipboard (or press Esc to exit)...",
            ViewMode::Emoji => "Search Emojis or :shortcode: (e.g. :cat:, :fire:)...",
            ViewMode::Gif => "Search GIFs...",
            ViewMode::Kaomoji => "Search Kaomoji emoticons...",
            ViewMode::Symbols => "Search Symbols & Math...",
            ViewMode::Favorites => "Search Favorites...",
        };

        div()
            .px_4()
            .py_1p5()
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_2()
                    .px_3()
                    .py_2()
                    .bg(palette.search_bg)
                    .border_1()
                    .border_color(palette.search_border)
                    .rounded_md()
                    .child(
                        // Magnifying glass icon
                        div()
                            .text_sm()
                            .text_color(palette.text_muted)
                            .child("🔍"),
                    )
                    .child(
                        // Query text or placeholder
                        div()
                            .flex_1()
                            .text_sm()
                            .text_color(if self.state.search_query.is_empty() {
                                palette.text_muted
                            } else {
                                palette.text_primary
                            })
                            .child(if self.state.search_query.is_empty() {
                                placeholder.to_string()
                            } else {
                                self.state.search_query.clone()
                            }),
                    )
                    .child(
                        // Clear search 'x' button when query is not empty
                        if !self.state.search_query.is_empty() {
                            div()
                                .cursor(gpui::CursorStyle::PointingHand)
                                .text_xs()
                                .text_color(palette.text_muted)
                                .hover(|s| s.text_color(palette.text_primary))
                                .child("✕")
                                .on_mouse_down(
                                    MouseButton::Left,
                                    cx.listener(|this, _: &MouseDownEvent, window, cx| {
                                        this.state.search_query.clear();
                                        this.focus_handle.focus(window);
                                        cx.notify();
                                    }),
                                )
                        } else {
                            div()
                        },
                    ),
            )
    }

    /// Main content scroll area
    fn render_content(&self, palette: Palette, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .id("content-scroll-container")
            .flex_1()
            .overflow_y_scroll()
            .px_4()
            .py_2()
            .on_mouse_down(
                MouseButton::Left,
                cx.listener(|this, _: &MouseDownEvent, window, _cx| {
                    this.focus_handle.focus(window);
                }),
            )
            .child(match self.state.view_mode {
                ViewMode::Clipboard => self.render_clipboard_view(palette, cx).into_any_element(),
                ViewMode::Emoji => self.render_emoji_view(palette, cx).into_any_element(),
                ViewMode::Gif => self.render_gif_view(palette, cx).into_any_element(),
                ViewMode::Kaomoji => self.render_kaomoji_view(palette, cx).into_any_element(),
                ViewMode::Symbols => self.render_symbols_view(palette, cx).into_any_element(),
                ViewMode::Favorites => self.render_favorites_view(palette, cx).into_any_element(),
            })
    }

    /// Render Clipboard history list (cards matching Windows 11 with screenshot and image previews)
    fn render_clipboard_view(&self, palette: Palette, cx: &mut Context<Self>) -> impl IntoElement {
        let query = self.state.search_query.to_lowercase();
        let entries: Vec<_> = self
            .state
            .clipboard_entries
            .iter()
            .filter(|e| {
                if query.is_empty() {
                    true
                } else {
                    e.content.to_lowercase().contains(&query)
                }
            })
            .collect();

        if entries.is_empty() {
            return div()
                .flex()
                .flex_col()
                .items_center()
                .justify_center()
                .h_full()
                .py_12()
                .gap_2()
                .child(
                    div()
                        .text_3xl()
                        .child("📋"),
                )
                .child(
                    div()
                        .text_sm()
                        .font_weight(FontWeight::SEMIBOLD)
                        .text_color(palette.header_text)
                        .child("Your clipboard is empty"),
                )
                .child(
                    div()
                        .text_xs()
                        .text_color(palette.text_muted)
                        .child("When you copy text, links, or take screenshots, they will appear here."),
                );
        }

        div()
            .flex()
            .flex_col()
            .gap_2()
            .children(entries.into_iter().map(|entry| {
                let entry_id = entry.id;
                let entry_content = entry.content.clone();
                let is_pinned = entry.is_pinned;
                let relative_time = relative_time_string(entry.created_at);
                let is_image = entry.content_type == ContentType::Image;
                let is_url = entry.content_type == ContentType::Url
                    || entry.content.starts_with("http://")
                    || entry.content.starts_with("https://");

                div()
                    .flex()
                    .flex_col()
                    .p_3()
                    .bg(palette.card_bg)
                    .border_1()
                    .border_color(palette.card_border)
                    .rounded_lg()
                    .hover(|s| s.border_color(palette.card_hover_border).bg(palette.card_hover_bg))
                    // Card Top Row: Type / Timestamp on left, delete / options on right
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .justify_between()
                            .pb_1()
                            .child(
                                div()
                                    .flex()
                                    .items_center()
                                    .gap_2()
                                    .child(
                                        div()
                                            .text_xs()
                                            .font_weight(FontWeight::MEDIUM)
                                            .text_color(palette.text_muted)
                                            .child(if is_image {
                                                "🖼️ Screenshot / Image"
                                            } else if is_url {
                                                "🔗 Link"
                                            } else {
                                                "📄 Text"
                                            }),
                                    )
                                    .child(
                                        div()
                                            .text_xs()
                                            .text_color(palette.text_muted)
                                            .child(format!("• {}", relative_time)),
                                    ),
                            )
                            .child(
                                // Delete icon button (🗑️)
                                div()
                                    .px_1p5()
                                    .py_0p5()
                                    .rounded_md()
                                    .cursor(gpui::CursorStyle::PointingHand)
                                    .hover(|s| s.bg(palette.button_hover).text_color(rgb(0xe81123)))
                                    .text_xs()
                                    .text_color(palette.text_muted)
                                    .child("🗑️")
                                    .on_mouse_down(
                                        MouseButton::Left,
                                        cx.listener(move |this, _: &MouseDownEvent, window, cx| {
                                            this.state.delete_entry(entry_id);
                                            this.state.toast_message = Some("Entry deleted".to_string());
                                            this.focus_handle.focus(window);
                                            cx.notify();
                                        }),
                                    ),
                            ),
                    )
                    // Card Content Body (click copies content and auto-closes panel)
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .cursor(gpui::CursorStyle::PointingHand)
                            .child(
                                if is_image {
                                    let img_path = PathBuf::from(&entry.content);
                                    if img_path.exists() {
                                        div()
                                            .py_1p5()
                                            .w_full()
                                            .child(
                                                img(img_path)
                                                    .w_full()
                                                    .max_h(px(140.0))
                                                    .object_fit(ObjectFit::Contain)
                                                    .rounded_md()
                                                    .border_1()
                                                    .border_color(palette.card_border),
                                            )
                                    } else {
                                        div()
                                            .py_2()
                                            .text_sm()
                                            .text_color(palette.text_muted)
                                            .child("🖼️ Image [cache file removed]")
                                    }
                                } else {
                                    div()
                                        .py_1()
                                        .text_sm()
                                        .line_height(px(20.0))
                                        .text_color(palette.text_primary)
                                        .child(
                                            if entry.content.len() > 110 {
                                                format!("{}…", &entry.content[..107])
                                            } else {
                                                entry.content.clone()
                                            },
                                        )
                                },
                            )
                            .on_mouse_down(
                                MouseButton::Left,
                                {
                                    let entry_content = entry_content.clone();
                                    cx.listener(move |this, _: &MouseDownEvent, _window, cx| {
                                        if is_image {
                                            this.state.copy_image(&entry_content);
                                        } else {
                                            this.state.copy_text(&entry_content);
                                        }
                                        cx.quit();
                                    })
                                },
                            ),
                    )
                    // Card Bottom Row: Click hint on left, Pin icon on bottom right
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .justify_between()
                            .pt_1()
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(palette.text_muted)
                                    .child("Click to copy & close"),
                            )
                            .child(
                                // Pushpin icon button (📌)
                                div()
                                    .flex()
                                    .items_center()
                                    .justify_center()
                                    .w_6()
                                    .h_6()
                                    .rounded_md()
                                    .cursor(gpui::CursorStyle::PointingHand)
                                    .bg(if is_pinned {
                                        palette.pin_active_bg
                                    } else {
                                        rgba(0x00000000)
                                    })
                                    .hover(|s| s.bg(palette.button_hover))
                                    .text_xs()
                                    .text_color(if is_pinned {
                                        palette.pin_active_text
                                    } else {
                                        palette.text_muted
                                    })
                                    .child("📌")
                                    .on_mouse_down(
                                        MouseButton::Left,
                                        cx.listener(move |this, _: &MouseDownEvent, window, cx| {
                                            this.state.toggle_pin(entry_id);
                                            this.state.toast_message = Some(if is_pinned {
                                                "Unpinned entry".to_string()
                                            } else {
                                                "Pinned entry".to_string()
                                            });
                                            this.focus_handle.focus(window);
                                            cx.notify();
                                        }),
                                    ),
                            ),
                    )
            }))
    }

    /// Render Emoji Picker view powered by emojic catalog with full category pills, shortcode search, and dedicated top Recent box
    fn render_emoji_view(&self, palette: Palette, cx: &mut Context<Self>) -> impl IntoElement {
        let categories = [
            "All", "Smileys", "People", "Animals", "Food", "Travel", "Activities", "Objects", "Symbols", "Flags",
        ];
        let selected_cat = self.state.selected_category.clone();
        let query = self.state.search_query.to_lowercase();
        let recent_emojis = self.state.recent_emojis.clone();

        let all_emojis = get_emoji_list();
        let resolved_alias = resolve_emojic_shortcode(&query);

        let filtered_emojis: Vec<_> = all_emojis
            .into_iter()
            .filter(|e| {
                let matches_cat = selected_cat == "All" || e.category == selected_cat;
                let matches_query = query.is_empty()
                    || e.name.to_lowercase().contains(&query)
                    || e.keywords.iter().any(|k| k.to_lowercase().contains(&query))
                    || resolved_alias.map(|r| r == e.char).unwrap_or(false);
                matches_cat && matches_query
            })
            .collect();

        div()
            .flex()
            .flex_col()
            .gap_3()
            // 1. Separate "Recently Used" Box at the top (stays distinct and does not alter the main grid)
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap_2()
                    .p_2p5()
                    .bg(palette.card_bg)
                    .border_1()
                    .border_color(palette.card_border)
                    .rounded_lg()
                    .child(
                        // Header row for Recent Box
                        div()
                            .flex()
                            .items_center()
                            .justify_between()
                            .child(
                                div()
                                    .flex()
                                    .items_center()
                                    .gap_1p5()
                                    .child(
                                        div()
                                            .text_xs()
                                            .child("🕒"),
                                    )
                                    .child(
                                        div()
                                            .text_xs()
                                            .font_weight(FontWeight::SEMIBOLD)
                                            .text_color(palette.header_text)
                                            .child("Recently used"),
                                    ),
                            )
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(palette.text_muted)
                                    .child(format!("{} recent", recent_emojis.len())),
                            ),
                    )
                    .child(
                        // Recent emojis row
                        div()
                            .flex()
                            .items_center()
                            .gap_1p5()
                            .flex_wrap()
                            .children(recent_emojis.into_iter().map(|emoji_str| {
                                let item_emoji = emoji_str.clone();
                                div()
                                    .flex()
                                    .items_center()
                                    .justify_center()
                                    .w(px(34.0))
                                    .h(px(34.0))
                                    .rounded_md()
                                    .cursor(gpui::CursorStyle::PointingHand)
                                    .hover(|s| s.bg(palette.nav_icon_hover))
                                    .text_xl()
                                    .font_family("Noto Color Emoji")
                                    .child(emoji_str)
                                    .on_mouse_down(
                                        MouseButton::Left,
                                        cx.listener(move |this, _: &MouseDownEvent, _window, cx| {
                                            this.state.use_emoji(&item_emoji);
                                            cx.quit();
                                        }),
                                    )
                            })),
                    ),
            )
            // 2. Category pills row
            .child(
                div()
                    .flex()
                    .gap_1()
                    .flex_wrap()
                    .children(categories.into_iter().map(|cat| {
                        let is_active = selected_cat == cat;
                        let cat_str = cat.to_string();

                        div()
                            .px_2p5()
                            .py_1()
                            .rounded_full()
                            .cursor(gpui::CursorStyle::PointingHand)
                            .bg(if is_active {
                                palette.pill_active_bg
                            } else {
                                palette.pill_idle_bg
                            })
                            .text_color(if is_active {
                                palette.pill_active_text
                            } else {
                                palette.pill_idle_text
                            })
                            .text_xs()
                            .font_weight(if is_active {
                                FontWeight::BOLD
                            } else {
                                FontWeight::NORMAL
                            })
                            .child(cat)
                            .on_mouse_down(
                                MouseButton::Left,
                                cx.listener(move |this, _: &MouseDownEvent, window, cx| {
                                    this.state.selected_category = cat_str.clone();
                                    this.focus_handle.focus(window);
                                    cx.notify();
                                }),
                            )
                    })),
            )
            // 3. Main Emoji Grid
            .child(
                div()
                    .flex()
                    .flex_wrap()
                    .gap_1p5()
                    .children(filtered_emojis.into_iter().map(|item| {
                        let emoji_char = item.char;

                        div()
                            .flex()
                            .items_center()
                            .justify_center()
                            .w(px(38.0))
                            .h(px(38.0))
                            .rounded_md()
                            .cursor(gpui::CursorStyle::PointingHand)
                            .hover(|s| s.bg(palette.nav_icon_hover))
                            .text_2xl()
                            .font_family("Noto Color Emoji")
                            .child(emoji_char)
                            .on_mouse_down(
                                MouseButton::Left,
                                cx.listener(move |this, _: &MouseDownEvent, _window, cx| {
                                    this.state.use_emoji(emoji_char);
                                    cx.quit();
                                }),
                            )
                    })),
            )
    }

    /// Render Kaomoji Picker view
    fn render_kaomoji_view(&self, palette: Palette, cx: &mut Context<Self>) -> impl IntoElement {
        let categories = ["All", "Classic", "Happy", "Greeting", "Cute", "Sad", "Angry"];
        let selected_cat = self.state.selected_category.clone();
        let query = self.state.search_query.to_lowercase();

        let all_kaomojis = get_kaomoji_list();
        let filtered_kaomojis: Vec<_> = all_kaomojis
            .into_iter()
            .filter(|k| {
                let matches_cat = selected_cat == "All" || k.category == selected_cat;
                let matches_query = query.is_empty()
                    || k.text.to_lowercase().contains(&query)
                    || k.name.to_lowercase().contains(&query);
                matches_cat && matches_query
            })
            .collect();

        div()
            .flex()
            .flex_col()
            .gap_3()
            .child(
                // Category pills row
                div()
                    .flex()
                    .gap_1()
                    .flex_wrap()
                    .children(categories.into_iter().map(|cat| {
                        let is_active = selected_cat == cat;
                        let cat_str = cat.to_string();

                        div()
                            .px_2p5()
                            .py_1()
                            .rounded_full()
                            .cursor(gpui::CursorStyle::PointingHand)
                            .bg(if is_active {
                                palette.pill_active_bg
                            } else {
                                palette.pill_idle_bg
                            })
                            .text_color(if is_active {
                                palette.pill_active_text
                            } else {
                                palette.pill_idle_text
                            })
                            .text_xs()
                            .font_weight(if is_active {
                                FontWeight::BOLD
                            } else {
                                FontWeight::NORMAL
                            })
                            .child(cat)
                            .on_mouse_down(
                                MouseButton::Left,
                                cx.listener(move |this, _: &MouseDownEvent, window, cx| {
                                    this.state.selected_category = cat_str.clone();
                                    this.focus_handle.focus(window);
                                    cx.notify();
                                }),
                            )
                    })),
            )
            .child(
                // Kaomoji list cards
                div()
                    .flex()
                    .flex_wrap()
                    .gap_2()
                    .children(filtered_kaomojis.into_iter().map(|item| {
                        let kaomoji_text = item.text;
                        let item_name = item.name;

                        div()
                            .flex()
                            .flex_col()
                            .items_center()
                            .justify_center()
                            .px_3()
                            .py_2()
                            .bg(palette.card_bg)
                            .border_1()
                            .border_color(palette.card_border)
                            .rounded_md()
                            .cursor(gpui::CursorStyle::PointingHand)
                            .hover(|s| s.border_color(palette.accent_blue).bg(palette.card_hover_bg))
                            .child(
                                div()
                                    .text_sm()
                                    .font_weight(FontWeight::MEDIUM)
                                    .text_color(palette.text_primary)
                                    .child(kaomoji_text),
                            )
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(palette.text_muted)
                                    .child(item_name),
                            )
                            .on_mouse_down(
                                MouseButton::Left,
                                cx.listener(move |this, _: &MouseDownEvent, _window, cx| {
                                    this.state.copy_text(kaomoji_text);
                                    cx.quit();
                                }),
                            )
                    })),
            )
    }

    /// Render Unicode Symbols Picker view
    fn render_symbols_view(&self, palette: Palette, cx: &mut Context<Self>) -> impl IntoElement {
        let categories = ["All", "Punctuation", "Currency", "Math", "Arrows", "Greek", "Special"];
        let selected_cat = self.state.selected_category.clone();
        let query = self.state.search_query.to_lowercase();

        let all_symbols = get_symbol_list();
        let filtered_symbols: Vec<_> = all_symbols
            .into_iter()
            .filter(|s| {
                let matches_cat = selected_cat == "All" || s.category == selected_cat;
                let matches_query = query.is_empty()
                    || s.value.to_lowercase().contains(&query)
                    || s.name.to_lowercase().contains(&query);
                matches_cat && matches_query
            })
            .collect();

        div()
            .flex()
            .flex_col()
            .gap_3()
            .child(
                // Category pills row
                div()
                    .flex()
                    .gap_1()
                    .flex_wrap()
                    .children(categories.into_iter().map(|cat| {
                        let is_active = selected_cat == cat;
                        let cat_str = cat.to_string();

                        div()
                            .px_2p5()
                            .py_1()
                            .rounded_full()
                            .cursor(gpui::CursorStyle::PointingHand)
                            .bg(if is_active {
                                palette.pill_active_bg
                            } else {
                                palette.pill_idle_bg
                            })
                            .text_color(if is_active {
                                palette.pill_active_text
                            } else {
                                palette.pill_idle_text
                            })
                            .text_xs()
                            .font_weight(if is_active {
                                FontWeight::BOLD
                            } else {
                                FontWeight::NORMAL
                            })
                            .child(cat)
                            .on_mouse_down(
                                MouseButton::Left,
                                cx.listener(move |this, _: &MouseDownEvent, window, cx| {
                                    this.state.selected_category = cat_str.clone();
                                    this.focus_handle.focus(window);
                                    cx.notify();
                                }),
                            )
                    })),
            )
            .child(
                // Symbols Grid
                div()
                    .flex()
                    .flex_wrap()
                    .gap_1p5()
                    .children(filtered_symbols.into_iter().map(|item| {
                        let sym_val = item.value;

                        div()
                            .flex()
                            .items_center()
                            .justify_center()
                            .w(px(38.0))
                            .h(px(38.0))
                            .rounded_md()
                            .cursor(gpui::CursorStyle::PointingHand)
                            .hover(|s| s.bg(palette.nav_icon_hover))
                            .text_xl()
                            .text_color(palette.text_primary)
                            .child(sym_val)
                            .on_mouse_down(
                                MouseButton::Left,
                                cx.listener(move |this, _: &MouseDownEvent, _window, cx| {
                                    this.state.copy_text(sym_val);
                                    cx.quit();
                                }),
                            )
                    })),
            )
    }

    /// Render Reaction GIF picker view
    fn render_gif_view(&self, palette: Palette, cx: &mut Context<Self>) -> impl IntoElement {
        let query = self.state.search_query.to_lowercase();
        let gifs = get_gif_list();
        let filtered_gifs: Vec<_> = gifs
            .into_iter()
            .filter(|g| {
                query.is_empty()
                    || g.title.to_lowercase().contains(&query)
                    || g.tag.to_lowercase().contains(&query)
            })
            .collect();

        div()
            .flex()
            .flex_col()
            .gap_2()
            .children(filtered_gifs.into_iter().map(|gif| {
                let gif_url = gif.url;
                let gif_title = gif.title;

                div()
                    .flex()
                    .items_center()
                    .justify_between()
                    .p_3()
                    .bg(palette.card_bg)
                    .border_1()
                    .border_color(palette.card_border)
                    .rounded_md()
                    .cursor(gpui::CursorStyle::PointingHand)
                    .hover(|s| s.border_color(palette.accent_blue).bg(palette.card_hover_bg))
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_3()
                            .child(
                                div()
                                    .text_2xl()
                                    .child(gif.preview_emoji),
                            )
                            .child(
                                div()
                                    .flex()
                                    .flex_col()
                                    .child(
                                        div()
                                            .text_sm()
                                            .font_weight(FontWeight::MEDIUM)
                                            .text_color(palette.text_primary)
                                            .child(gif_title),
                                    )
                                    .child(
                                        div()
                                            .text_xs()
                                            .text_color(palette.text_muted)
                                            .child(format!("Reaction • {}", gif.tag)),
                                    ),
                            ),
                    )
                    .child(
                        div()
                            .px_2()
                            .py_1()
                            .rounded_md()
                            .bg(palette.button_bg)
                            .text_xs()
                            .text_color(palette.text_secondary)
                            .child("Copy GIF"),
                    )
                    .on_mouse_down(
                        MouseButton::Left,
                        cx.listener(move |this, _: &MouseDownEvent, _window, cx| {
                            this.state.copy_text(gif_url);
                            cx.quit();
                        }),
                    )
            }))
    }

    /// Render Favorites & Recently Used tab
    fn render_favorites_view(&self, palette: Palette, cx: &mut Context<Self>) -> impl IntoElement {
        let pinned_entries: Vec<_> = self
            .state
            .clipboard_entries
            .iter()
            .filter(|e| e.is_pinned)
            .collect();

        if pinned_entries.is_empty() {
            return div()
                .flex()
                .flex_col()
                .items_center()
                .justify_center()
                .h_full()
                .py_12()
                .gap_2()
                .child(
                    div()
                        .text_3xl()
                        .child("♡"),
                )
                .child(
                    div()
                        .text_sm()
                        .font_weight(FontWeight::SEMIBOLD)
                        .text_color(palette.header_text)
                        .child("No favorites yet"),
                )
                .child(
                    div()
                        .text_xs()
                        .text_color(palette.text_muted)
                        .child("Click the pin 📌 icon on any clipboard item or symbol to keep it here."),
                );
        }

        div()
            .flex()
            .flex_col()
            .gap_2()
            .children(pinned_entries.into_iter().map(|entry| {
                let _entry_id = entry.id;
                let entry_content = entry.content.clone();
                let is_image = entry.content_type == ContentType::Image;

                div()
                    .flex()
                    .items_center()
                    .justify_between()
                    .p_3()
                    .bg(palette.card_bg)
                    .border_1()
                    .border_color(palette.accent_blue)
                    .rounded_lg()
                    .cursor(gpui::CursorStyle::PointingHand)
                    .hover(|s| s.bg(palette.card_hover_bg))
                    .child(
                        if is_image {
                            div()
                                .flex()
                                .items_center()
                                .gap_2()
                                .child(div().text_sm().child("🖼️"))
                                .child(
                                    div()
                                        .text_xs()
                                        .font_weight(FontWeight::MEDIUM)
                                        .text_color(palette.text_primary)
                                        .child("Pinned Screenshot"),
                                )
                        } else {
                            div()
                                .flex_1()
                                .text_sm()
                                .text_color(palette.text_primary)
                                .child(
                                    if entry.content.len() > 60 {
                                        format!("{}…", &entry.content[..57])
                                    } else {
                                        entry.content.clone()
                                    },
                                )
                        },
                    )
                    .child(
                        div()
                            .text_sm()
                            .text_color(palette.accent_blue)
                            .child("📌"),
                    )
                    .on_mouse_down(
                        MouseButton::Left,
                        cx.listener(move |this, _: &MouseDownEvent, _window, cx| {
                            if is_image {
                                this.state.copy_image(&entry_content);
                            } else {
                                this.state.copy_text(&entry_content);
                            }
                            cx.quit();
                        }),
                    )
            }))
    }

    /// Render bottom footer with feedback toast and status bar
    fn render_footer(&self, palette: Palette, cx: &mut Context<Self>) -> impl IntoElement {
        let total_count = self.state.clipboard_entries.len();
        let pinned_count = self.state.clipboard_entries.iter().filter(|e| e.is_pinned).count();

        div()
            .flex()
            .flex_col()
            .px_4()
            .py_2()
            .border_t_1()
            .border_color(palette.window_border)
            .child(
                // Floating copy feedback toast notification
                if let Some(toast) = &self.state.toast_message {
                    let toast_text = toast.clone();
                    div()
                        .flex()
                        .items_center()
                        .justify_between()
                        .px_3()
                        .py_1p5()
                        .mb_1p5()
                        .bg(rgb(0x107c41))
                        .text_color(rgb(0xffffff))
                        .rounded_md()
                        .text_xs()
                        .font_weight(FontWeight::MEDIUM)
                        .child(toast_text)
                        .child(
                            div()
                                .cursor(gpui::CursorStyle::PointingHand)
                                .text_xs()
                                .text_color(rgb(0xaaaab2))
                                .hover(|s| s.text_color(rgb(0xffffff)))
                                .child("✕")
                                .on_mouse_down(
                                    MouseButton::Left,
                                    cx.listener(|this, _: &MouseDownEvent, _window, cx| {
                                        this.state.toast_message = None;
                                        cx.notify();
                                    }),
                                ),
                        )
                } else {
                    div()
                },
            )
            .child(
                // Status bar line with item counts and theme indicator
                div()
                    .flex()
                    .items_center()
                    .justify_between()
                    .child(
                        div()
                            .text_xs()
                            .text_color(palette.text_muted)
                            .child(format!(
                                "{} mode • Esc to close",
                                match self.state.theme {
                                    ThemeMode::Light => "Light",
                                    ThemeMode::Dark => "Dark",
                                }
                            )),
                    )
                    .child(
                        div()
                            .text_xs()
                            .text_color(palette.text_muted)
                            .child(format!("{} items • {} pinned", total_count, pinned_count)),
                    ),
            )
    }
}

fn handle_ingest_text() {
    use std::io::Read;
    let mut text = String::new();
    if std::io::stdin().read_to_string(&mut text).is_ok() {
        if !text.is_empty() {
            let db_dir = dirs::data_dir()
                .unwrap_or_else(|| PathBuf::from("."))
                .join("nexus");
            let _ = std::fs::create_dir_all(&db_dir);
            let db_path = db_dir.join("nexus.db");
            if let Ok(mut storage) = SqliteStorage::open(&db_path) {
                let now = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs() as i64;
                let ctype = if text.starts_with("http://") || text.starts_with("https://") {
                    ContentType::Url
                } else {
                    ContentType::Text
                };
                let entry = ClipboardEntry {
                    id: 0,
                    content_type: ctype,
                    content: text,
                    content_hash: String::new(),
                    created_at: now,
                    last_used_at: None,
                    is_pinned: false,
                };
                let _ = storage.save_entry(&entry);
            }
        }
    }
}

fn handle_ingest_image() {
    use std::io::Read;
    let mut img_bytes = Vec::new();
    if std::io::stdin().read_to_end(&mut img_bytes).is_ok() {
        if !img_bytes.is_empty() {
            let db_dir = dirs::data_dir()
                .unwrap_or_else(|| PathBuf::from("."))
                .join("nexus");
            let _ = std::fs::create_dir_all(&db_dir);
            let db_path = db_dir.join("nexus.db");
            let cache_dir = get_images_cache_dir();
            let now = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs() as i64;
            let len = img_bytes.len();
            let file_path = cache_dir.join(format!("clip_{}_{}.png", now, len));

            if std::fs::write(&file_path, &img_bytes).is_ok() {
                let path_str = file_path.to_string_lossy().to_string();
                if let Ok(mut storage) = SqliteStorage::open(&db_path) {
                    let entry = ClipboardEntry {
                        id: 0,
                        content_type: ContentType::Image,
                        content: path_str,
                        content_hash: format!("img_{:x}", len),
                        created_at: now,
                        last_used_at: None,
                        is_pinned: false,
                    };
                    let _ = storage.save_entry(&entry);
                }
            }
        }
    }
}

fn run_daemon() {
    println!("▶️ Nexus Clipboard Daemon running in background...");
    let self_exe = std::env::current_exe().unwrap_or_else(|_| PathBuf::from("nexus-ui"));

    // Check if on Wayland and wl-paste is available
    let has_wl_paste = std::env::var_os("WAYLAND_DISPLAY").is_some()
        && Command::new("wl-paste")
            .arg("-v")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);

    if has_wl_paste {
        println!("   ⚡ Native event-driven Wayland clipboard engine active (0ms latency).");

        let spawn_text = || {
            Command::new("wl-paste")
                .arg("-t")
                .arg("text/plain")
                .arg("--watch")
                .arg(&self_exe)
                .arg("--ingest-text")
                .spawn()
        };

        let spawn_img = || {
            Command::new("wl-paste")
                .arg("-t")
                .arg("image/png")
                .arg("--watch")
                .arg(&self_exe)
                .arg("--ingest-image")
                .spawn()
        };

        let mut text_child = spawn_text().ok();
        let mut img_child = spawn_img().ok();

        loop {
            // Monitor and respawn watchers if terminated
            if let Some(ref mut c) = text_child {
                if let Ok(Some(_)) = c.try_wait() {
                    *c = spawn_text().unwrap_or_else(|e| panic!("Failed to respawn text watcher: {:?}", e));
                }
            } else {
                text_child = spawn_text().ok();
            }
            if let Some(ref mut c) = img_child {
                if let Ok(Some(_)) = c.try_wait() {
                    *c = spawn_img().unwrap_or_else(|e| panic!("Failed to respawn image watcher: {:?}", e));
                }
            } else {
                img_child = spawn_img().ok();
            }

            std::thread::sleep(std::time::Duration::from_secs(3));
        }
    } else {
        println!("   Monitoring X11 clipboard via polling...");
        let db_dir = dirs::data_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("nexus");
        let _ = std::fs::create_dir_all(&db_dir);
        let db_path = db_dir.join("nexus.db");

        let mut storage = match SqliteStorage::open(&db_path) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("Failed to open storage: {:?}", e);
                return;
            }
        };

        let mut last_text: Option<String> = get_system_clipboard_text();

        loop {
            if let Some(text) = get_system_clipboard_text() {
                if !text.is_empty() && last_text.as_deref() != Some(&text) {
                    last_text = Some(text.clone());
                    let now = SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_secs() as i64;
                    let ctype = if text.starts_with("http://") || text.starts_with("https://") {
                        ContentType::Url
                    } else {
                        ContentType::Text
                    };
                    let entry = ClipboardEntry {
                        id: 0,
                        content_type: ctype,
                        content: text,
                        content_hash: String::new(),
                        created_at: now,
                        last_used_at: None,
                        is_pinned: false,
                    };
                    let _ = storage.save_entry(&entry);
                }
            }
            std::thread::sleep(std::time::Duration::from_millis(200));
        }
    }
}

fn main() {
    tracing_subscriber::fmt::init();

    if std::env::args().any(|a| a == "--ingest-text") {
        handle_ingest_text();
        return;
    }

    if std::env::args().any(|a| a == "--ingest-image") {
        handle_ingest_image();
        return;
    }

    if std::env::args().any(|a| a == "--daemon" || a == "-d") {
        run_daemon();
        return;
    }

    let initial_mode = if std::env::args().any(|a| a == "--emoji" || a == "-e") {
        ViewMode::Emoji
    } else if std::env::args().any(|a| a == "--symbols" || a == "-s") {
        ViewMode::Symbols
    } else if std::env::args().any(|a| a == "--gif" || a == "-g") {
        ViewMode::Gif
    } else if std::env::args().any(|a| a == "--kaomoji" || a == "-k") {
        ViewMode::Kaomoji
    } else {
        ViewMode::Clipboard
    };

    Application::new().run(move |cx: &mut App| {
        // Floating compact window matching Windows 11 flyout dimensions with transparent glass backing
        let bounds = Bounds::centered(None, size(px(420.0), px(580.0)), cx);

        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                titlebar: Some(TitlebarOptions {
                    title: Some("Nexus Panel".into()),
                    appears_transparent: true,
                    traffic_light_position: None,
                }),
                kind: WindowKind::PopUp,
                is_resizable: false,
                app_id: Some("nexus-ui".into()),
                window_decorations: Some(WindowDecorations::Client),
                window_background: WindowBackgroundAppearance::Transparent,
                ..Default::default()
            },
            move |window, cx| cx.new(|cx| NexusPanel::new(cx, window, initial_mode)),
        )
        .unwrap();

        cx.activate(true);
    });
}
