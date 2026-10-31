/// 2D Point structure
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Point {
    pub x: f32,
    pub y: f32,
}

impl Point {
    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }
}

/// 2D Rect structure
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl Rect {
    pub fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self { x, y, width, height }
    }
}

/// Platform trait for display server integration
pub trait PlatformInterface: Send + Sync {
    /// Get the display server type
    fn display_server(&self) -> DisplayServer;
    /// Get available monitors
    fn available_monitors(&self) -> Vec<MonitorInfo>;
    /// Position a popup window
    fn position_popup(
        &self,
        width: f32,
        height: f32,
        x: Option<f32>,
        y: Option<f32>,
    ) -> Point;
    /// Handle global shortcut
    fn handle_shortcut(&self, shortcut: &str) -> bool;
    /// Get current workspace/monitor
    fn current_workspace(&self) -> usize;
}

/// Display server types
#[derive(Debug, Clone, PartialEq)]
pub enum DisplayServer {
    Wayland,
    X11,
    Unknown,
}

/// Monitor information
#[derive(Debug, Clone)]
pub struct MonitorInfo {
    /// Monitor name/identifier
    pub name: String,
    /// Monitor geometry (x, y, width, height)
    pub geometry: Rect,
    /// Whether it's the primary monitor
    pub is_primary: bool,
}

/// Default platform implementation
pub struct DefaultPlatform;

impl Default for DefaultPlatform {
    fn default() -> Self {
        Self
    }
}

impl PlatformInterface for DefaultPlatform {
    fn display_server(&self) -> DisplayServer {
        DisplayServer::Wayland
    }

    fn available_monitors(&self) -> Vec<MonitorInfo> {
        vec![MonitorInfo {
            name: "Default".to_string(),
            geometry: Rect::new(0.0, 0.0, 1920.0, 1080.0),
            is_primary: true,
        }]
    }

    fn position_popup(
        &self,
        width: f32,
        height: f32,
        x: Option<f32>,
        y: Option<f32>,
    ) -> Point {
        let x = x.unwrap_or(0.0);
        let y = y.unwrap_or(0.0);
        Point::new(x + width / 2.0, y + height / 2.0)
    }

    fn handle_shortcut(&self, shortcut: &str) -> bool {
        shortcut == "super+v" || shortcut == "super+."
    }

    fn current_workspace(&self) -> usize {
        0
    }
}

/// Wayland-specific platform implementation
pub struct WaylandPlatform {
    /// Compositor type
    compositor: CompositorType,
}

/// Compositor types
#[derive(Debug, Clone, PartialEq)]
pub enum CompositorType {
    Hyprland,
    Niri,
    Sway,
    KDE,
    GNOME,
    Other,
}

impl WaylandPlatform {
    /// Detect the running Wayland compositor
    pub fn detect_compositor() -> CompositorType {
        if let Ok(compositor) = std::env::var("XDG_CURRENT_DESKTOP") {
            match compositor.to_lowercase().as_str() {
                "hyprland" => CompositorType::Hyprland,
                "niri" => CompositorType::Niri,
                "sway" => CompositorType::Sway,
                _ => CompositorType::Other,
            }
        } else {
            CompositorType::Other
        }
    }

    /// Query Hyprland monitors
    fn query_hyprland_monitors(&self) -> Vec<MonitorInfo> {
        vec![MonitorInfo {
            name: "Hyprland".to_string(),
            geometry: Rect::new(0.0, 0.0, 1920.0, 1080.0),
            is_primary: true,
        }]
    }

    /// Query Niri monitors
    fn query_niri_monitors(&self) -> Vec<MonitorInfo> {
        vec![MonitorInfo {
            name: "Niri".to_string(),
            geometry: Rect::new(0.0, 0.0, 1920.0, 1080.0),
            is_primary: true,
        }]
    }

    /// Query Sway monitors
    fn query_sway_monitors(&self) -> Vec<MonitorInfo> {
        vec![MonitorInfo {
            name: "Sway".to_string(),
            geometry: Rect::new(0.0, 0.0, 1920.0, 1080.0),
            is_primary: true,
        }]
    }

    /// Query default monitors
    fn query_default_monitors(&self) -> Vec<MonitorInfo> {
        vec![MonitorInfo {
            name: "Default".to_string(),
            geometry: Rect::new(0.0, 0.0, 1920.0, 1080.0),
            is_primary: true,
        }]
    }
}

impl Default for WaylandPlatform {
    fn default() -> Self {
        let compositor = Self::detect_compositor();
        Self { compositor }
    }
}

impl PlatformInterface for WaylandPlatform {
    fn display_server(&self) -> DisplayServer {
        DisplayServer::Wayland
    }

    fn available_monitors(&self) -> Vec<MonitorInfo> {
        match self.compositor {
            CompositorType::Hyprland => self.query_hyprland_monitors(),
            CompositorType::Niri => self.query_niri_monitors(),
            CompositorType::Sway => self.query_sway_monitors(),
            _ => self.query_default_monitors(),
        }
    }

    fn position_popup(
        &self,
        _width: f32,
        _height: f32,
        x: Option<f32>,
        y: Option<f32>,
    ) -> Point {
        let x = x.unwrap_or(0.0);
        let y = y.unwrap_or(0.0);
        Point::new(x, y)
    }

    fn handle_shortcut(&self, shortcut: &str) -> bool {
        shortcut == "super+v" || shortcut == "super+."
    }

    fn current_workspace(&self) -> usize {
        0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wayland_platform_creation() {
        let platform = WaylandPlatform::default();
        assert_eq!(platform.display_server(), DisplayServer::Wayland);
    }

    #[test]
    fn test_display_server_detection() {
        let platform = DefaultPlatform::default();
        assert!(matches!(platform.display_server(), DisplayServer::Wayland | DisplayServer::X11));
    }

    #[test]
    fn test_shortcut_handling() {
        let platform = WaylandPlatform::default();
        assert!(platform.handle_shortcut("super+v"));
        assert!(platform.handle_shortcut("super+."));
        assert!(!platform.handle_shortcut("ctrl+c"));
    }
}
