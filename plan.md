Nexus Panel
Complete Project Plan & Technical Blueprint
Tagline: Everything you copy, find and insert — in one place.
Property Details
Project Name Nexus Panel
Project Type Open-source Linux desktop utility
Primary Language Rust
UI Framework GPUI
Target OS Linux
Primary Display Server Wayland
Initial Compositors Hyprland and Niri
Database SQLite
Architecture Modular Rust workspace
Main Goal Lightweight, fast and universal Linux input hub

1. Project Overview
   1.1 What is Nexus Panel?

Nexus Panel is a lightweight, modern and unified input-management application for Linux.

Its purpose is to combine everyday desktop features into one integrated interface instead of requiring users to install and configure multiple independent applications.

Windows provides familiar functionality such as:

Win + V — Clipboard history.
Win + . — Emoji, GIF and symbols panel.

Linux users frequently need separate applications for these features.

Nexus aims to solve this problem by bringing clipboard management, emoji selection, GIF search, symbols and other input-related features into one unified application.

Core philosophy
One application.
One interface.
One background engine.
Multiple integrated features.
Minimal resource consumption.
Universal Linux compatibility.

Nexus should feel like a natural part of the Linux desktop rather than another third-party application.

1.2 Main Objectives
Build a unified clipboard and emoji experience.
Support multiple Linux distributions.
Support multiple Wayland compositors.
Provide a beautiful, modern and customizable UI.
Maintain extremely low RAM consumption.
Achieve fast startup and responsive interactions.
Support offline functionality.
Avoid Electron and Chromium.
Keep the core independent of the UI framework.
Build an open-source project that the Linux community can use. 2. Main Features
2.1 Clipboard Manager

The clipboard manager is the primary feature and the first module to develop.

Features
Automatic clipboard monitoring.
Persistent clipboard history.
Search clipboard entries.
Copy previous entries.
Pin frequently used entries.
Delete individual entries.
Clear clipboard history.
Duplicate-entry detection.
Configurable history limits.
Recent clipboard entries.
Private mode.
Configurable retention period.
Supported content types

Initial support:

Plain text.
URLs.

Future support:

Images.
Files and file references.
Rich text.
Additional MIME types.
Clipboard UI

Each clipboard entry should include:

Content preview.
Content type.
Timestamp.
Copy button.
Pin button.
Context menu.

Long clipboard entries should be truncated in the list and displayed fully when selected.

Privacy

Clipboard data can contain passwords, authentication tokens and sensitive information.

Nexus must provide:

Local storage by default.
Private mode.
Configurable history retention.
Clear-history functionality.
Application exclusion rules where possible.
No automatic cloud synchronization.
No clipboard-content analytics.
2.2 Emoji Picker

A fully integrated emoji picker.

Features
Emoji search.
Emoji categories.
Recently used emojis.
Frequently used emojis.
Favorite emojis.
Skin-tone variations.
Unicode emoji support.
Emoji sequences.
Categories
Smileys and emotions.
People.
Animals and nature.
Food and drink.
Travel and places.
Activities.
Objects.
Symbols.
Flags.
Search examples
happy → 😀 😃 😄
fire → 🔥
heart → ❤️ 💙 💚
rocket → 🚀

Emoji metadata should be stored locally to support offline searching.

2.3 GIF Picker

GIF support will be an optional module.

Features
GIF search.
GIF previews.
GIF grid.
Recently used GIFs.
Favorite GIFs.
Copy or insert supported GIF content.
Possible providers
Tenor.
Other compatible GIF providers.
Future self-hosted provider.

GIF functionality should remain inactive until requested.

API keys, licensing and provider terms must be reviewed before implementation.

2.4 Symbols Picker

A searchable Unicode symbol library.

Categories
Mathematical symbols.
Currency.
Arrows.
Greek alphabet.
Technical symbols.
Punctuation.
Superscripts.
Subscripts.
Special Unicode characters.

Examples:

© ® ™ € £ ¥ → ← ↑ ↓ ∞ ≠ ≤ ≥ ∑ √

All basic symbol data should be available offline.

2.5 Kaomoji Picker

A dedicated section for text-based expressions.

Examples:

(╯°□°）╯︵ ┻━┻
¯\_(ツ)_/¯
(｡♥‿♥｡)
(ง'̀-'́)ง
ʕ•ᴥ•ʔ

Users should be able to search, copy and favorite expressions.

1. Technology Stack
   3.1 Primary Technologies
   Component Technology
   Programming language Rust
   UI framework GPUI
   Database SQLite
   Serialization Serde
   IPC Unix Domain Sockets
   Async runtime Tokio, where needed
   Clipboard integration Wayland protocols
   Emoji metadata Local Unicode dataset
   Configuration TOML
   Build system Cargo
   Testing Rust testing framework

Dependencies should be kept minimal.

Every dependency must have a clear purpose.

3.2 Rust

Rust will power the entire application.

Reasons:

Memory safety.
High performance.
Strong type system.
Excellent concurrency support.
No mandatory garbage collector.
Suitable for low-level Linux integration.
Good ecosystem for desktop utilities.
3.3 GPUI

GPUI will be the primary UI framework.

Reasons:

Rust-based development.
GPU-accelerated rendering.
Custom UI layouts.
Fine-grained rendering control.
Smooth animations.
Custom visual identity.
No embedded Chromium requirement.

Official resources:

<https://gpui.rs/>
<https://github.com/zed-industries/zed>

GPUI is still evolving, so its APIs and Linux integration must be evaluated during development.

3.4 SQLite

SQLite will store:

Clipboard history.
Favorites.
Recently used emojis.
Recently used symbols.
Application settings.
Metadata.

Benefits:

Embedded database.
No separate database server.
Local persistence.
Efficient searching.
Transaction support.
3.5 IPC

The UI and background core should communicate through a local IPC interface.

Possible operations:

text
GET_HISTORY
SEARCH_HISTORY
COPY_ENTRY
DELETE_ENTRY
PIN_ENTRY
CLEAR_HISTORY
GET_SETTINGS
UPDATE_SETTINGS

The IPC protocol should be versioned and protected with appropriate local filesystem permissions.

1. Application Architecture

Nexus should use a modular architecture.

text
NEXUS PANEL
|
+------------+------------+
| |
NEXUS CORE NEXUS UI
| |
Clipboard Engine GPUI
Storage Engine Views
Search Engine Components
Emoji Engine Animations
IPC Server Navigation
|
PLATFORM LAYER
|
+-----+------+
| |
Wayland X11 (later)
|
Compositor Adapters
|
+--+----------------+
| |
Hyprland Niri

4.1 Nexus Core

Responsibilities:

Clipboard monitoring.
Clipboard processing.
History management.
Database operations.
Search.
Favorites.
Privacy controls.
Configuration.
IPC communication.

The core must not depend on GPUI.

4.2 Nexus UI

Responsibilities:

Main panel.
Navigation.
Search interface.
Clipboard history display.
Emoji picker.
GIF picker.
Symbols picker.
Kaomoji picker.
Animations.
Theme management.
Keyboard navigation.

The UI should communicate with the core through defined interfaces.

4.3 Platform Layer

Responsibilities:

Clipboard integration.
Keyboard shortcut integration.
Window activation.
Popup positioning.
Focus management.
Desktop detection.
Compositor-specific functionality.

Platform-specific functionality must remain isolated from the core.

1. UI Design
   5.1 Design Philosophy

Nexus should have its own visual identity.

The UI should combine:

Windows 11-style usability.
Modern Linux aesthetics.
Clean typography.
Smooth animations.
Minimal visual clutter.
Keyboard-first navigation.
Customizable themes.

Avoid unnecessary visual effects that consume additional resources.

5.2 Main Layout
Header
Nexus branding.
Search bar.
Settings button.
Optional module controls.
Navigation
Clipboard.
Emoji.
GIFs.
Symbols.
Kaomoji.
Main Content

The main area changes depending on the selected module.

Footer
Keyboard shortcut hints.
Current module.
Optional status information.
5.3 UI Behavior
Open quickly.
Close with Escape.
Support keyboard navigation.
Support mouse interaction.
Remember the previous section.
Adapt to different resolutions.
Support multiple monitors.
Support dark and light themes.
Allow configurable dimensions.
5.4 UI Components

Suggested components:

text
Header
SearchBar
NavigationBar
ClipboardList
ClipboardItem
EmojiGrid
EmojiCategory
GIFGrid
SymbolGrid
KaomojiGrid
SettingsPanel
ContextMenu
PopupWindow

1. Wayland Integration

Wayland integration is one of the most important technical challenges.

Unlike X11, Wayland does not provide unrestricted clipboard monitoring to every application.

Nexus must use supported protocols.

6.1 Clipboard Protocols

Investigate:

ext-data-control-v1.
wlr-data-control-unstable-v1.
Wayland data-device protocols.
Clipboard MIME types.

Potential Rust library:

lamco-data-control

Repository:

<https://github.com/lamco-admin/lamco-rs>

Verify its current API and supported protocols before implementation.

6.2 Initial Compositor Support

First targets:

Hyprland.
Niri.

Later targets:

Sway.
KDE Plasma.
GNOME.
Other compatible Wayland compositors.

Support must be tested individually rather than assumed.

6.3 Global Shortcuts

Desired shortcuts:

text
Super + V Open clipboard
Super + . Open emoji picker
Super + Shift+V Open full Nexus panel
Escape Close panel

Possible integration methods:

XDG Global Shortcuts portal.
Compositor-specific configurations.
Desktop-specific adapters.
User-configurable external shortcuts.

Nexus should not assume that all compositors provide identical global shortcut functionality.

6.4 Popup Behavior

The popup should support:

Keyboard focus.
Correct positioning.
Multiple monitors.
Escape-to-close.
Focus restoration.
Proper fullscreen behavior.

A normal application window and a compositor-managed overlay are different things.

GPUI's actual Wayland window capabilities must be tested before selecting the final popup implementation.

6.5 X11 Support

X11 support should be added after Wayland functionality is stable.

The core must remain independent of the display server.

1. Performance Engineering

Performance is a primary requirement.

Nexus should remain lightweight even when its UI is closed.

7.1 Performance Principles
No Electron.
No Chromium.
No unnecessary background workers.
No constant database polling.
No unnecessary allocations.
No unnecessary network requests.
No continuous UI rendering while hidden.
No expensive image processing on the UI thread.
No unnecessary caching of large clipboard objects.
7.2 Idle Behavior

When the panel is closed:

No UI animation.
No unnecessary rendering.
No repeated database queries.
No network requests.
Clipboard monitoring remains event-driven.
Unused image previews are released.
Only essential core resources remain active.
7.3 Initial Performance Targets

These are targets, not guaranteed benchmarks.

Metric Target
Idle core RAM Under 15 MB
UI while open Under 60 MB
Popup opening Under 100 ms
Search response Under 30 ms
CPU while idle Near zero beyond essential event handling

Actual measurements must be collected on real hardware.

GPU resources and renderer memory must be included in the measurements.

7.4 Benchmarking Tools
/usr/bin/time -v
htop
btop
hyperfine
perf
heaptrack

Benchmark:

Cold startup.
Warm startup.
Idle RAM.
Open-panel RAM.
Clipboard capture latency.
Search latency.
CPU usage.
Large clipboard histories.
7.5 Performance Test Scenarios

Test with:

100 clipboard entries.
1,000 clipboard entries.
10,000 clipboard entries.
Large text entries.
Duplicate entries.
Rapid clipboard changes.
Large image clipboard entries.

Optimize based on actual profiling data.

1. Database Design

Initial SQLite schema:

Clipboard Entries
sql
CREATE TABLE clipboard_entries (
id INTEGER PRIMARY KEY,
content_type TEXT NOT NULL,
content TEXT,
content_hash TEXT NOT NULL,
created_at INTEGER NOT NULL,
last_used_at INTEGER,
is_pinned INTEGER DEFAULT 0
);

Favorites
sql
CREATE TABLE favorites (
id INTEGER PRIMARY KEY,
item_type TEXT NOT NULL,
item_value TEXT NOT NULL,
created_at INTEGER NOT NULL
);

Settings
sql
CREATE TABLE settings (
key TEXT PRIMARY KEY,
value TEXT NOT NULL
);

This is an initial schema.

Future versions should support binary clipboard content, MIME metadata, retention policies and database migrations.

1. Project Directory Structure

Use a Cargo workspace.

text
nexus/
│
├── Cargo.toml
├── README.md
├── LICENSE
├── CONTRIBUTING.md
├── CHANGELOG.md
│
├── crates/
│ ├── nexus-core/
│ ├── nexus-ui/
│ ├── nexus-clipboard/
│ ├── nexus-storage/
│ ├── nexus-search/
│ ├── nexus-emoji/
│ ├── nexus-symbols/
│ ├── nexus-gif/
│ ├── nexus-ipc/
│ ├── nexus-platform/
│ └── nexus-config/
│
├── assets/
│ ├── icons/
│ ├── themes/
│ └── emoji/
│
├── packaging/
│ ├── arch/
│ ├── debian/
│ ├── fedora/
│ ├── flatpak/
│ └── nix/
│
├── docs/
│ ├── architecture.md
│ ├── development.md
│ ├── performance.md
│ └── security.md
│
└── tests/
├── integration/
└── benchmarks/

Keep the project modular without creating unnecessary independent crates for trivial functionality.

 1. Development Roadmap
    Phase 0 — Research and Technical Validation

Tasks:

Create a minimal GPUI project.
Test GPUI on Hyprland.
Test GPUI on Niri.
Verify window focus behavior.
Test popup behavior.
Test Wayland clipboard access.
Measure baseline RAM.
Test shortcut integration.

Deliverable:

A working GPUI prototype on both target compositors.

Phase 1 — Core Infrastructure

Tasks:

Create Rust workspace.
Build configuration system.
Implement SQLite storage.
Implement IPC protocol.
Create background service.
Implement logging.
Add error handling.

Deliverable:

Working background core.

Phase 2 — Clipboard MVP

Tasks:

Implement Wayland clipboard listener.
Capture text entries.
Store clipboard history.
Detect duplicates.
Search entries.
Copy old entries.
Delete entries.
Pin entries.
Clear history.
Add basic privacy controls.

Deliverable:

Functional clipboard manager.

Phase 3 — GPUI Interface

Tasks:

Build main panel.
Build search bar.
Build navigation.
Build clipboard list.
Add keyboard navigation.
Implement popup behavior.
Add dark theme.
Add light theme.

Deliverable:

Polished clipboard interface.

Phase 4 — Emoji and Symbols

Tasks:

Integrate local emoji dataset.
Implement emoji search.
Add emoji categories.
Add recent emojis.
Add favorites.
Add skin-tone support.
Implement symbol search.
Implement kaomoji support.

Deliverable:

Unified clipboard and emoji panel.

Phase 5 — GIF Integration

Tasks:

Create GIF provider abstraction.
Configure API provider.
Implement GIF search.
Add previews.
Add favorites.
Add recent GIFs.

Deliverable:

Optional online GIF functionality.

Phase 6 — Desktop Integration

Tasks:

Implement Hyprland integration.
Implement Niri integration.
Add shortcut configuration.
Improve window placement.
Implement focus management.
Add multiple monitor support.

Deliverable:

Proper desktop integration.

Phase 7 — Performance Optimization

Tasks:

Profile startup.
Measure idle RAM.
Measure open-panel RAM.
Profile CPU usage.
Optimize search.
Optimize clipboard processing.
Optimize database access.
Optimize UI rendering.

Deliverable:

Performance-tested release candidate.

Phase 8 — Distribution and Release

Tasks:

Build Flatpak package.
Build Arch package.
Build DEB package.
Build RPM package.
Build Nix package.
Write installation instructions.
Prepare documentation.
Publish source code.

Deliverable:

Nexus Panel v1.0.

 1. Distribution Support

Nexus should be independent of distribution-specific package managers.

Initial Targets
Distribution Package
Arch Linux PKGBUILD
CachyOS Arch-compatible
Manjaro Arch-compatible, tested separately
Ubuntu DEB
Debian DEB
Fedora RPM
openSUSE RPM
NixOS Nix
Universal option Flatpak

Flatpak sandbox permissions must be considered carefully because clipboard monitoring and global shortcut integration may be restricted.

Native packages should also be available.

 1. Security and Privacy

Clipboard data is potentially sensitive.

Privacy Requirements
Local clipboard storage by default.
No cloud synchronization by default.
No automatic clipboard uploads.
No clipboard-content analytics.
Configurable retention.
Private mode.
Clear-history option.
No unnecessary network requests.
Security Requirements
Validate IPC messages.
Restrict IPC socket access.
Avoid logging clipboard contents.
Limit clipboard entry sizes.
Handle malformed MIME data.
Protect database permissions.
Never execute clipboard contents.
Review dependencies regularly.

Password-manager integration and sensitive-content detection should be treated carefully.

Nexus must never claim that all sensitive clipboard data can be detected automatically.

 1. Open-Source Strategy

Nexus should be developed as an open-source community project.

Recommended repository files:

text
README.md
CONTRIBUTING.md
LICENSE
CODE_OF_CONDUCT.md
SECURITY.md
CHANGELOG.md
ROADMAP.md

Suggested licenses:

MIT.
Apache-2.0.

Apache-2.0 is worth considering because of its explicit patent provisions.

Before publishing:

Review dependency licenses.
Verify emoji dataset licensing.
Verify GIF provider terms.
Document API requirements.
Provide build instructions. 14. Future Features

These are future possibilities, not MVP requirements.

Clipboard
Image clipboard history.
File references.
Rich text.
Clipboard collections.
Text snippets.
Clipboard templates.
Optional synchronization.
UI
Custom themes.
Custom animations.
Configurable shortcuts.
Custom positioning.
Compact mode.
Multiple layouts.
Productivity
Frequently used text.
Quick text transformations.
Developer symbols.
Unicode inspector.
Searchable clipboard collections.
Platform
GNOME integration.
KDE integration.
COSMIC integration.
Sway support.
X11 support.
Other compatible Wayland compositors. 15. MVP Requirements

The first public version should focus on reliability and performance.

Required MVP Features
GPUI floating panel.
Rust background core.
Wayland clipboard monitoring.
Persistent text clipboard history.
Clipboard search.
Copy and delete functionality.
Basic privacy controls.
Emoji picker.
Symbol picker.
Hyprland integration.
Niri integration.

GIF support, image history and synchronization can be developed later.

 1. Project Development Principles

Every development decision should follow these principles.

Performance First

No feature should introduce unnecessary background resource consumption.

Privacy First

Clipboard data belongs to the user.

Distribution Independent

The application should not depend on Arch-specific functionality.

Compositor Independent

Platform-specific functionality must remain isolated behind adapters.

Modular

Every major feature should be maintainable independently.

Offline First

Core functionality should work without an internet connection.

Keyboard First

Important operations should be accessible without a mouse.

Beautiful but Efficient

The UI should look polished without relying on expensive visual effects.

Minimal Dependencies

Every dependency must justify its cost.

 1. Immediate Development Checklist

Before building the full application:

Create Nexus Git repository.
Install Rust and Cargo.
Set up minimal GPUI application.
Test GPUI on Hyprland.
Test GPUI on Niri.
Verify popup and focus behavior.
Test Wayland clipboard protocols.
Measure baseline RAM.
Establish workspace architecture.
Implement clipboard prototype.
Connect core to UI.
Implement first keyboard shortcut.
Create initial visual design. 18. Final Product Vision

Nexus Panel should become a universal, lightweight Linux input hub.

A user should be able to install Nexus on their preferred Linux distribution, configure their shortcuts and immediately get a unified clipboard and emoji experience.

They should not need to install several unrelated utilities.

They should not need to learn multiple interfaces.

They should not need to sacrifice performance.

One panel. One engine. One beautiful experience. Every Linux desktop.

Nexus is not intended to replace Linux desktop environments.

It is intended to make everyday Linux desktop usage feel more complete.

Project Identity

Name: Nexus Panel

Tagline: Everything you copy, find and insert — in one place.

Language: Rust

UI: GPUI

Core: Modular Rust workspace

Database: SQLite

Initial Platform: Wayland

Initial Compositors: Hyprland and Niri

Long-Term Goal: Universal Linux desktop support.
