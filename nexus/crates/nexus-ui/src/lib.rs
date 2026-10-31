use nexus_storage::{ClipboardEntry, ContentType, SqliteStorage};
use serde::{Deserialize, Serialize};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{SystemTime, UNIX_EPOCH};
use tracing::info;

/// Navigation view modes corresponding to the Windows 11 Input & Clipboard flyout tabs
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ViewMode {
    /// Favorites / Recent tab (Heart icon)
    Favorites,
    /// Emoji picker (Smiley icon)
    Emoji,
    /// GIF picker (GIF icon)
    Gif,
    /// Kaomoji picker ( ;-) icon )
    Kaomoji,
    /// Symbols picker (% π Ω icon)
    Symbols,
    /// Clipboard history (Clipboard icon) - default active in Windows 11
    Clipboard,
}

/// Theme mode (Light matches Windows 11 Fluent light, Dark matches Fluent dark)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ThemeMode {
    Light,
    Dark,
}

/// Detect Linux system theme preference (Dark or Light) via gsettings, dconf, and environment
pub fn detect_system_theme() -> ThemeMode {
    // 1. Try GNOME / desktop interface color-scheme
    if let Ok(output) = Command::new("gsettings")
        .args(["get", "org.gnome.desktop.interface", "color-scheme"])
        .output()
    {
        if output.status.success() {
            let s = String::from_utf8_lossy(&output.stdout).to_lowercase();
            if s.contains("dark") {
                return ThemeMode::Dark;
            } else if s.contains("light") {
                return ThemeMode::Light;
            }
        }
    }

    // 2. Try GNOME / desktop interface gtk-theme
    if let Ok(output) = Command::new("gsettings")
        .args(["get", "org.gnome.desktop.interface", "gtk-theme"])
        .output()
    {
        if output.status.success() {
            let s = String::from_utf8_lossy(&output.stdout).to_lowercase();
            if s.contains("dark") {
                return ThemeMode::Dark;
            }
        }
    }

    // 3. Try dconf read directly
    if let Ok(output) = Command::new("dconf")
        .args(["read", "/org/gnome/desktop/interface/color-scheme"])
        .output()
    {
        if output.status.success() {
            let s = String::from_utf8_lossy(&output.stdout).to_lowercase();
            if s.contains("dark") {
                return ThemeMode::Dark;
            }
        }
    }

    // 4. Try GTK_THEME environment variable
    if let Ok(theme) = std::env::var("GTK_THEME") {
        if theme.to_lowercase().contains("dark") {
            return ThemeMode::Dark;
        }
    }

    ThemeMode::Light
}

/// Kaomoji item
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KaomojiItem {
    pub text: &'static str,
    pub category: &'static str,
    pub name: &'static str,
}

/// Curated Kaomojis grouped into Windows 11 categories
pub fn get_kaomoji_list() -> Vec<KaomojiItem> {
    vec![
        // Classic
        KaomojiItem { text: "¯\\_(ツ)_/¯", category: "Classic", name: "Shrug" },
        KaomojiItem { text: "(╯°□°)╯︵ ┻━┻", category: "Classic", name: "Table flip" },
        KaomojiItem { text: "┬─┬ノ( º _ ºノ)", category: "Classic", name: "Put table back" },
        KaomojiItem { text: "( ͡° ͜ʖ ͡°)", category: "Classic", name: "Lenny face" },
        KaomojiItem { text: "ʕ•ᴥ•ʔ", category: "Classic", name: "Bear" },
        KaomojiItem { text: "(ง'̀-'́)ง", category: "Classic", name: "Ready to fight" },
        KaomojiItem { text: "ಠ_ಠ", category: "Classic", name: "Look of disapproval" },
        KaomojiItem { text: "(•‿•)", category: "Classic", name: "Smile" },

        // Happy
        KaomojiItem { text: "(｡♥‿♥｡)", category: "Happy", name: "In love" },
        KaomojiItem { text: "(✿◠‿◠)", category: "Happy", name: "Cute flower" },
        KaomojiItem { text: "(◕‿◕✿)", category: "Happy", name: "Gentle smile" },
        KaomojiItem { text: "(ﾉ◕ヮ◕)ﾉ*:･ﾟ✧", category: "Happy", name: "Sparkles" },
        KaomojiItem { text: "(≧▽≦)", category: "Happy", name: "Super happy" },
        KaomojiItem { text: "╰(▔∀▔)╯", category: "Happy", name: "Hooray" },
        KaomojiItem { text: "(o^▽^o)", category: "Happy", name: "Delighted" },
        KaomojiItem { text: "(*^‿^*)", category: "Happy", name: "Sweet grin" },

        // Greeting
        KaomojiItem { text: "( ´ ▽ ` )ﾉ", category: "Greeting", name: "Wave hello" },
        KaomojiItem { text: "(・∀・)ノ", category: "Greeting", name: "Hey there" },
        KaomojiItem { text: "(￣▽￣)/", category: "Greeting", name: "Friendly wave" },
        KaomojiItem { text: "ヾ(＾∇＾)", category: "Greeting", name: "Energetic wave" },
        KaomojiItem { text: "(・_・)ノ", category: "Greeting", name: "Casual greeting" },
        KaomojiItem { text: "＼(＾▽＾)／", category: "Greeting", name: "Welcome" },

        // Cute
        KaomojiItem { text: "(=^･ω･^=)", category: "Cute", name: "Kitty cat" },
        KaomojiItem { text: "(=^･ｪ･^=)", category: "Cute", name: "Puppy dog" },
        KaomojiItem { text: "(づ｡◕‿‿◕｡)づ", category: "Cute", name: "Big warm hug" },
        KaomojiItem { text: "(｡•́‿•̀｡)", category: "Cute", name: "Soft face" },
        KaomojiItem { text: "(｡･ω･｡)", category: "Cute", name: "Curious puppy" },
        KaomojiItem { text: "(='.'=)", category: "Cute", name: "Bunny rabbit" },

        // Sad
        KaomojiItem { text: "(╥﹏╥)", category: "Sad", name: "Crying tears" },
        KaomojiItem { text: "(ಥ﹏ಥ)", category: "Sad", name: "Sobbing" },
        KaomojiItem { text: "(T_T)", category: "Sad", name: "Tears" },
        KaomojiItem { text: "(ノ_<。)", category: "Sad", name: "Hiding grief" },
        KaomojiItem { text: "( ; ω ; )", category: "Sad", name: "Emotional" },
        KaomojiItem { text: "(っ˘̩╭╮˘̩)っ", category: "Sad", name: "Needs hug" },

        // Angry
        KaomojiItem { text: "(ノಠ益ಠ)ノ彡┻━┻", category: "Angry", name: "Rage flip" },
        KaomojiItem { text: "(╬▔皿▔)╯", category: "Angry", name: "Gritting teeth" },
        KaomojiItem { text: "凸(｀0´)凸", category: "Angry", name: "Defiant" },
        KaomojiItem { text: "(`皿´＃)", category: "Angry", name: "Furious" },
        KaomojiItem { text: "(◣_◢)", category: "Angry", name: "Glare" },
    ]
}

/// GIF item representation
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GifItem {
    pub title: &'static str,
    pub preview_emoji: &'static str,
    pub tag: &'static str,
    pub url: &'static str,
}

/// Curated reaction GIF list
pub fn get_gif_list() -> Vec<GifItem> {
    vec![
        GifItem { title: "Mind Blown", preview_emoji: "🤯", tag: "Reaction", url: "https://media.giphy.com/media/26ufdipQqU2lhNA4g/giphy.gif" },
        GifItem { title: "Applause", preview_emoji: "👏", tag: "Applaud", url: "https://media.giphy.com/media/ytTYwIlWYnm6Y/giphy.gif" },
        GifItem { title: "Confused", preview_emoji: "🤔", tag: "Thinking", url: "https://media.giphy.com/media/3o7btPCcdNniyf0ArS/giphy.gif" },
        GifItem { title: "Dance Party", preview_emoji: "🕺", tag: "Celebration", url: "https://media.giphy.com/media/l0MYt5jPR6QX5pnqM/giphy.gif" },
        GifItem { title: "Yes / Agree", preview_emoji: "👍", tag: "Approval", url: "https://media.giphy.com/media/nXxOjZrbnbRxS/giphy.gif" },
        GifItem { title: "Facepalm", preview_emoji: "🤦", tag: "Reaction", url: "https://media.giphy.com/media/3xz2BLBOt13X9AgjEA/giphy.gif" },
        GifItem { title: "Popcorn Time", preview_emoji: "🍿", tag: "Waiting", url: "https://media.giphy.com/media/gl0mkIZOW6Nwc/giphy.gif" },
        GifItem { title: "Cat Vibing", preview_emoji: "🐱", tag: "Chill", url: "https://media.giphy.com/media/jpbnoe3UIa8TU8LM13/giphy.gif" },
        GifItem { title: "Victory Celebration", preview_emoji: "🎉", tag: "Celebration", url: "https://media.giphy.com/media/artj92V8o75VPL7AeQ/giphy.gif" },
        GifItem { title: "Cheers", preview_emoji: "🥂", tag: "Cheers", url: "https://media.giphy.com/media/BPJmthQ3YRwD6QqcVD/giphy.gif" },
    ]
}

/// Extended symbol item representation
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SymbolDisplayItem {
    pub value: &'static str,
    pub name: &'static str,
    pub category: &'static str,
}

/// Comprehensive Unicode symbols list grouped like Windows 11 Symbols panel
pub fn get_symbol_list() -> Vec<SymbolDisplayItem> {
    vec![
        // General Punctuation & Typography
        SymbolDisplayItem { value: "©", name: "Copyright", category: "Punctuation" },
        SymbolDisplayItem { value: "®", name: "Registered", category: "Punctuation" },
        SymbolDisplayItem { value: "™", name: "Trademark", category: "Punctuation" },
        SymbolDisplayItem { value: "•", name: "Bullet", category: "Punctuation" },
        SymbolDisplayItem { value: "·", name: "Middle Dot", category: "Punctuation" },
        SymbolDisplayItem { value: "…", name: "Ellipsis", category: "Punctuation" },
        SymbolDisplayItem { value: "§", name: "Section", category: "Punctuation" },
        SymbolDisplayItem { value: "¶", name: "Pilcrow", category: "Punctuation" },
        SymbolDisplayItem { value: "†", name: "Dagger", category: "Punctuation" },
        SymbolDisplayItem { value: "‡", name: "Double Dagger", category: "Punctuation" },
        SymbolDisplayItem { value: "“", name: "Left Double Quote", category: "Punctuation" },
        SymbolDisplayItem { value: "”", name: "Right Double Quote", category: "Punctuation" },
        SymbolDisplayItem { value: "‘", name: "Left Single Quote", category: "Punctuation" },
        SymbolDisplayItem { value: "’", name: "Right Single Quote", category: "Punctuation" },
        SymbolDisplayItem { value: "«", name: "Left Guillemet", category: "Punctuation" },
        SymbolDisplayItem { value: "»", name: "Right Guillemet", category: "Punctuation" },
        SymbolDisplayItem { value: "–", name: "En Dash", category: "Punctuation" },
        SymbolDisplayItem { value: "—", name: "Em Dash", category: "Punctuation" },

        // Currency
        SymbolDisplayItem { value: "$", name: "Dollar", category: "Currency" },
        SymbolDisplayItem { value: "€", name: "Euro", category: "Currency" },
        SymbolDisplayItem { value: "£", name: "Pound", category: "Currency" },
        SymbolDisplayItem { value: "¥", name: "Yen", category: "Currency" },
        SymbolDisplayItem { value: "₹", name: "Rupee", category: "Currency" },
        SymbolDisplayItem { value: "₽", name: "Ruble", category: "Currency" },
        SymbolDisplayItem { value: "₩", name: "Won", category: "Currency" },
        SymbolDisplayItem { value: "¢", name: "Cent", category: "Currency" },
        SymbolDisplayItem { value: "₿", name: "Bitcoin", category: "Currency" },
        SymbolDisplayItem { value: "₺", name: "Turkish Lira", category: "Currency" },
        SymbolDisplayItem { value: "₴", name: "Hryvnia", category: "Currency" },
        SymbolDisplayItem { value: "₫", name: "Dong", category: "Currency" },

        // Mathematics
        SymbolDisplayItem { value: "∞", name: "Infinity", category: "Math" },
        SymbolDisplayItem { value: "≠", name: "Not Equal", category: "Math" },
        SymbolDisplayItem { value: "≤", name: "Less or Equal", category: "Math" },
        SymbolDisplayItem { value: "≥", name: "Greater or Equal", category: "Math" },
        SymbolDisplayItem { value: "±", name: "Plus Minus", category: "Math" },
        SymbolDisplayItem { value: "×", name: "Multiply", category: "Math" },
        SymbolDisplayItem { value: "÷", name: "Divide", category: "Math" },
        SymbolDisplayItem { value: "∑", name: "Summation", category: "Math" },
        SymbolDisplayItem { value: "√", name: "Square Root", category: "Math" },
        SymbolDisplayItem { value: "π", name: "Pi", category: "Math" },
        SymbolDisplayItem { value: "≈", name: "Almost Equal", category: "Math" },
        SymbolDisplayItem { value: "≡", name: "Identical", category: "Math" },
        SymbolDisplayItem { value: "∫", name: "Integral", category: "Math" },
        SymbolDisplayItem { value: "∂", name: "Partial Differential", category: "Math" },
        SymbolDisplayItem { value: "∆", name: "Delta Increment", category: "Math" },
        SymbolDisplayItem { value: "∇", name: "Nabla", category: "Math" },
        SymbolDisplayItem { value: "∩", name: "Intersection", category: "Math" },
        SymbolDisplayItem { value: "∪", name: "Union", category: "Math" },
        SymbolDisplayItem { value: "°", name: "Degree", category: "Math" },
        SymbolDisplayItem { value: "‰", name: "Per Mille", category: "Math" },

        // Arrows
        SymbolDisplayItem { value: "→", name: "Right Arrow", category: "Arrows" },
        SymbolDisplayItem { value: "←", name: "Left Arrow", category: "Arrows" },
        SymbolDisplayItem { value: "↑", name: "Up Arrow", category: "Arrows" },
        SymbolDisplayItem { value: "↓", name: "Down Arrow", category: "Arrows" },
        SymbolDisplayItem { value: "↔", name: "Left Right Arrow", category: "Arrows" },
        SymbolDisplayItem { value: "↕", name: "Up Down Arrow", category: "Arrows" },
        SymbolDisplayItem { value: "↗", name: "North East Arrow", category: "Arrows" },
        SymbolDisplayItem { value: "↘", name: "South East Arrow", category: "Arrows" },
        SymbolDisplayItem { value: "↙", name: "South West Arrow", category: "Arrows" },
        SymbolDisplayItem { value: "↖", name: "North West Arrow", category: "Arrows" },
        SymbolDisplayItem { value: "⇒", name: "Double Right Arrow", category: "Arrows" },
        SymbolDisplayItem { value: "⇔", name: "Double Left Right Arrow", category: "Arrows" },

        // Greek Alphabet
        SymbolDisplayItem { value: "α", name: "Alpha", category: "Greek" },
        SymbolDisplayItem { value: "β", name: "Beta", category: "Greek" },
        SymbolDisplayItem { value: "γ", name: "Gamma", category: "Greek" },
        SymbolDisplayItem { value: "δ", name: "Delta", category: "Greek" },
        SymbolDisplayItem { value: "ε", name: "Epsilon", category: "Greek" },
        SymbolDisplayItem { value: "θ", name: "Theta", category: "Greek" },
        SymbolDisplayItem { value: "λ", name: "Lambda", category: "Greek" },
        SymbolDisplayItem { value: "μ", name: "Micro / Mu", category: "Greek" },
        SymbolDisplayItem { value: "σ", name: "Sigma", category: "Greek" },
        SymbolDisplayItem { value: "φ", name: "Phi", category: "Greek" },
        SymbolDisplayItem { value: "ω", name: "Omega", category: "Greek" },
        SymbolDisplayItem { value: "Ω", name: "Capital Omega", category: "Greek" },
        SymbolDisplayItem { value: "Δ", name: "Capital Delta", category: "Greek" },
        SymbolDisplayItem { value: "Σ", name: "Capital Sigma", category: "Greek" },

        // Special & Geometric
        SymbolDisplayItem { value: "★", name: "Black Star", category: "Special" },
        SymbolDisplayItem { value: "☆", name: "White Star", category: "Special" },
        SymbolDisplayItem { value: "♥", name: "Black Heart", category: "Special" },
        SymbolDisplayItem { value: "♦", name: "Diamond", category: "Special" },
        SymbolDisplayItem { value: "♠", name: "Spade", category: "Special" },
        SymbolDisplayItem { value: "♣", name: "Club", category: "Special" },
        SymbolDisplayItem { value: "♪", name: "Eighth Note", category: "Special" },
        SymbolDisplayItem { value: "♫", name: "Beamed Notes", category: "Special" },
        SymbolDisplayItem { value: "✓", name: "Check Mark", category: "Special" },
        SymbolDisplayItem { value: "✔", name: "Heavy Check", category: "Special" },
        SymbolDisplayItem { value: "✕", name: "Multiplication X", category: "Special" },
        SymbolDisplayItem { value: "✖", name: "Heavy X", category: "Special" },
        SymbolDisplayItem { value: "⚡", name: "High Voltage", category: "Special" },
        SymbolDisplayItem { value: "☕", name: "Hot Beverage", category: "Special" },
    ]
}

/// Rich emoji item representation
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EmojiItem {
    pub char: &'static str,
    pub name: &'static str,
    pub category: &'static str,
    pub keywords: &'static [&'static str],
}

/// Extensive emoji catalog covering Unicode 16 groups powered by emojic
pub fn get_emoji_list() -> Vec<EmojiItem> {
    vec![
        // === SMILEYS & EMOTION ===
        EmojiItem { char: "😀", name: "Grinning face", category: "Smileys", keywords: &["happy", "smile", "grin"] },
        EmojiItem { char: "😃", name: "Grinning face with big eyes", category: "Smileys", keywords: &["happy", "joy"] },
        EmojiItem { char: "😄", name: "Grinning face with smiling eyes", category: "Smileys", keywords: &["happy", "laugh"] },
        EmojiItem { char: "😁", name: "Beaming face", category: "Smileys", keywords: &["happy", "teeth"] },
        EmojiItem { char: "😆", name: "Grinning squinting face", category: "Smileys", keywords: &["laugh", "haha"] },
        EmojiItem { char: "😅", name: "Sweat smile", category: "Smileys", keywords: &["nervous", "relief"] },
        EmojiItem { char: "🤣", name: "Rolling on the floor laughing", category: "Smileys", keywords: &["rofl", "lol"] },
        EmojiItem { char: "😂", name: "Face with tears of joy", category: "Smileys", keywords: &["cry", "laugh", "tears"] },
        EmojiItem { char: "🙂", name: "Slightly smiling face", category: "Smileys", keywords: &["smile", "mild"] },
        EmojiItem { char: "🙃", name: "Upside-down face", category: "Smileys", keywords: &["silly", "sarcasm"] },
        EmojiItem { char: "😉", name: "Winking face", category: "Smileys", keywords: &["wink", "flirt"] },
        EmojiItem { char: "😊", name: "Smiling face with smiling eyes", category: "Smileys", keywords: &["blush", "warm"] },
        EmojiItem { char: "😇", name: "Smiling face with halo", category: "Smileys", keywords: &["angel", "innocent"] },
        EmojiItem { char: "🥰", name: "Smiling face with hearts", category: "Smileys", keywords: &["love", "adore"] },
        EmojiItem { char: "😍", name: "Heart eyes", category: "Smileys", keywords: &["love", "crush"] },
        EmojiItem { char: "🤩", name: "Star-struck", category: "Smileys", keywords: &["excited", "eyes"] },
        EmojiItem { char: "😘", name: "Face blowing a kiss", category: "Smileys", keywords: &["kiss", "flirt"] },
        EmojiItem { char: "😗", name: "Kissing face", category: "Smileys", keywords: &["kiss"] },
        EmojiItem { char: "😚", name: "Kissing face with closed eyes", category: "Smileys", keywords: &["kiss", "love"] },
        EmojiItem { char: "😙", name: "Kissing face with smiling eyes", category: "Smileys", keywords: &["kiss", "smile"] },
        EmojiItem { char: "😋", name: "Face savoring food", category: "Smileys", keywords: &["yum", "delicious"] },
        EmojiItem { char: "😛", name: "Face with tongue", category: "Smileys", keywords: &["playful", "silly"] },
        EmojiItem { char: "😜", name: "Winking face with tongue", category: "Smileys", keywords: &["crazy", "party"] },
        EmojiItem { char: "🤪", name: "Zany face", category: "Smileys", keywords: &["goofy", "wild"] },
        EmojiItem { char: "😝", name: "Squinting face with tongue", category: "Smileys", keywords: &["tongue", "haha"] },
        EmojiItem { char: "🤑", name: "Money-mouth face", category: "Smileys", keywords: &["rich", "dollar"] },
        EmojiItem { char: "🤗", name: "Smiling face with open hands", category: "Smileys", keywords: &["hug", "warm"] },
        EmojiItem { char: "🤭", name: "Face with hand over mouth", category: "Smileys", keywords: &["oops", "giggle"] },
        EmojiItem { char: "🤫", name: "Shushing face", category: "Smileys", keywords: &["quiet", "secret"] },
        EmojiItem { char: "🤔", name: "Thinking face", category: "Smileys", keywords: &["think", "hmm", "curious"] },
        EmojiItem { char: "🤐", name: "Zipper-mouth face", category: "Smileys", keywords: &["mute", "secret"] },
        EmojiItem { char: "🤨", name: "Face with raised eyebrow", category: "Smileys", keywords: &["skeptic", "doubt"] },
        EmojiItem { char: "😐", name: "Neutral face", category: "Smileys", keywords: &["blank", "straight"] },
        EmojiItem { char: "😑", name: "Expressionless face", category: "Smileys", keywords: &["flat", "bored"] },
        EmojiItem { char: "😶", name: "Face without mouth", category: "Smileys", keywords: &["silent", "quiet"] },
        EmojiItem { char: "😏", name: "Smirking face", category: "Smileys", keywords: &["smirk", "flirt"] },
        EmojiItem { char: "😒", name: "Unamused face", category: "Smileys", keywords: &["annoyed", "unimpressed"] },
        EmojiItem { char: "🙄", name: "Face with rolling eyes", category: "Smileys", keywords: &["eyeroll", "whatever"] },
        EmojiItem { char: "😬", name: "Grimacing face", category: "Smileys", keywords: &["awkward", "nervous"] },
        EmojiItem { char: "🤥", name: "Lying face", category: "Smileys", keywords: &["pinocchio", "lie"] },
        EmojiItem { char: "😌", name: "Relieved face", category: "Smileys", keywords: &["calm", "peace"] },
        EmojiItem { char: "😔", name: "Pensive face", category: "Smileys", keywords: &["sad", "thoughtful"] },
        EmojiItem { char: "😪", name: "Sleepy face", category: "Smileys", keywords: &["tired", "drowsy"] },
        EmojiItem { char: "🤤", name: "Drooling face", category: "Smileys", keywords: &["tasty", "crave"] },
        EmojiItem { char: "😴", name: "Sleeping face", category: "Smileys", keywords: &["sleep", "tired", "zzz"] },
        EmojiItem { char: "😷", name: "Face with medical mask", category: "Smileys", keywords: &["sick", "mask"] },
        EmojiItem { char: "🤒", name: "Face with thermometer", category: "Smileys", keywords: &["fever", "ill"] },
        EmojiItem { char: "🤕", name: "Face with head-bandage", category: "Smileys", keywords: &["hurt", "injured"] },
        EmojiItem { char: "🤢", name: "Nauseated face", category: "Smileys", keywords: &["gross", "sick"] },
        EmojiItem { char: "🤮", name: "Face vomiting", category: "Smileys", keywords: &["puke", "disgust"] },
        EmojiItem { char: "🤧", name: "Sneezing face", category: "Smileys", keywords: &["sneeze", "cold"] },
        EmojiItem { char: "🥵", name: "Hot face", category: "Smileys", keywords: &["heat", "sweat"] },
        EmojiItem { char: "🥶", name: "Cold face", category: "Smileys", keywords: &["freezing", "ice"] },
        EmojiItem { char: "🥴", name: "Woozy face", category: "Smileys", keywords: &["drunk", "dizzy"] },
        EmojiItem { char: "😵", name: "Face with crossed-out eyes", category: "Smileys", keywords: &["knocked out", "dead"] },
        EmojiItem { char: "🤯", name: "Exploding head", category: "Smileys", keywords: &["mindblown", "shock"] },
        EmojiItem { char: "🤠", name: "Cowboy hat face", category: "Smileys", keywords: &["sheriff", "wild west"] },
        EmojiItem { char: "🥳", name: "Partying face", category: "Smileys", keywords: &["birthday", "celebrate"] },
        EmojiItem { char: "😎", name: "Smiling face with sunglasses", category: "Smileys", keywords: &["cool", "sunglasses"] },
        EmojiItem { char: "🤓", name: "Nerd face", category: "Smileys", keywords: &["geek", "glasses"] },
        EmojiItem { char: "🧐", name: "Face with monocle", category: "Smileys", keywords: &["investigate", "classy"] },
        EmojiItem { char: "😕", name: "Confused face", category: "Smileys", keywords: &["puzzled"] },
        EmojiItem { char: "😟", name: "Worried face", category: "Smileys", keywords: &["concern", "nervous"] },
        EmojiItem { char: "🙁", name: "Slightly frowning face", category: "Smileys", keywords: &["sad"] },
        EmojiItem { char: "😮", name: "Face with open mouth", category: "Smileys", keywords: &["surprise", "wow"] },
        EmojiItem { char: "😯", name: "Hushed face", category: "Smileys", keywords: &["speechless"] },
        EmojiItem { char: "😲", name: "Astonished face", category: "Smileys", keywords: &["gasp", "amazed"] },
        EmojiItem { char: "😳", name: "Flushed face", category: "Smileys", keywords: &["blush", "embarrassed"] },
        EmojiItem { char: "🥺", name: "Pleading face", category: "Smileys", keywords: &["beg", "puppy eyes"] },
        EmojiItem { char: "😦", name: "Frowning face with open mouth", category: "Smileys", keywords: &["fear"] },
        EmojiItem { char: "😧", name: "Anguished face", category: "Smileys", keywords: &["shock", "distress"] },
        EmojiItem { char: "😨", name: "Fearful face", category: "Smileys", keywords: &["scared", "dread"] },
        EmojiItem { char: "😰", name: "Anxious face with sweat", category: "Smileys", keywords: &["panic", "stress"] },
        EmojiItem { char: "😥", name: "Sad but relieved face", category: "Smileys", keywords: &["whew", "close"] },
        EmojiItem { char: "😢", name: "Crying face", category: "Smileys", keywords: &["tear", "sad"] },
        EmojiItem { char: "😭", name: "Loudly crying face", category: "Smileys", keywords: &["bawling", "sobbing"] },
        EmojiItem { char: "😱", name: "Face screaming in fear", category: "Smileys", keywords: &["scared", "munch"] },
        EmojiItem { char: "😖", name: "Confounded face", category: "Smileys", keywords: &["struggle"] },
        EmojiItem { char: "😣", name: "Persevering face", category: "Smileys", keywords: &["pushing", "strain"] },
        EmojiItem { char: "😞", name: "Disappointed face", category: "Smileys", keywords: &["upset"] },
        EmojiItem { char: "😓", name: "Downcast face with sweat", category: "Smileys", keywords: &["tired"] },
        EmojiItem { char: "😩", name: "Weary face", category: "Smileys", keywords: &["exhausted"] },
        EmojiItem { char: "😫", name: "Tired face", category: "Smileys", keywords: &["sleepy", "done"] },
        EmojiItem { char: "🥱", name: "Yawning face", category: "Smileys", keywords: &["bored", "sleepy"] },
        EmojiItem { char: "😤", name: "Face with steam from nose", category: "Smileys", keywords: &["triumph", "frustrated"] },
        EmojiItem { char: "😡", name: "Enraged face", category: "Smileys", keywords: &["pout", "mad", "red"] },
        EmojiItem { char: "😠", name: "Angry face", category: "Smileys", keywords: &["mad", "grr"] },
        EmojiItem { char: "🤬", name: "Face with symbols on mouth", category: "Smileys", keywords: &["swearing", "curse"] },
        EmojiItem { char: "😈", name: "Smiling face with horns", category: "Smileys", keywords: &["devil", "evil"] },
        EmojiItem { char: "👿", name: "Angry face with horns", category: "Smileys", keywords: &["imp", "demon"] },
        EmojiItem { char: "💀", name: "Skull", category: "Smileys", keywords: &["dead", "skeleton", "death"] },
        EmojiItem { char: "☠️", name: "Skull and crossbones", category: "Smileys", keywords: &["poison", "danger"] },
        EmojiItem { char: "💩", name: "Pile of poo", category: "Smileys", keywords: &["poop", "funny"] },
        EmojiItem { char: "🤡", name: "Clown face", category: "Smileys", keywords: &["circus", "silly"] },
        EmojiItem { char: "👻", name: "Ghost", category: "Smileys", keywords: &["boo", "halloween"] },
        EmojiItem { char: "👽", name: "Alien", category: "Smileys", keywords: &["ufo", "et"] },
        EmojiItem { char: "🤖", name: "Robot", category: "Smileys", keywords: &["bot", "ai"] },
        EmojiItem { char: "😺", name: "Grinning cat", category: "Smileys", keywords: &["cat", "meow"] },
        EmojiItem { char: "😸", name: "Grinning cat with smiling eyes", category: "Smileys", keywords: &["happy cat"] },
        EmojiItem { char: "😹", name: "Cat with tears of joy", category: "Smileys", keywords: &["laughing cat"] },
        EmojiItem { char: "😻", name: "Smiling cat with heart-eyes", category: "Smileys", keywords: &["love cat"] },
        EmojiItem { char: "🔥", name: "Fire", category: "Smileys", keywords: &["hot", "lit", "flame"] },
        EmojiItem { char: "✨", name: "Sparkles", category: "Smileys", keywords: &["magic", "stars", "clean"] },
        EmojiItem { char: "💯", name: "Hundred points", category: "Smileys", keywords: &["100", "score", "perfect"] },
        EmojiItem { char: "❤️", name: "Red heart", category: "Smileys", keywords: &["love", "heart"] },
        EmojiItem { char: "🧡", name: "Orange heart", category: "Smileys", keywords: &["love", "warmth"] },
        EmojiItem { char: "💛", name: "Yellow heart", category: "Smileys", keywords: &["friendship", "gold"] },
        EmojiItem { char: "💚", name: "Green heart", category: "Smileys", keywords: &["nature", "health"] },
        EmojiItem { char: "💙", name: "Blue heart", category: "Smileys", keywords: &["peace", "loyalty"] },
        EmojiItem { char: "💜", name: "Purple heart", category: "Smileys", keywords: &["kindness", "royalty"] },
        EmojiItem { char: "🖤", name: "Black heart", category: "Smileys", keywords: &["dark", "emo"] },
        EmojiItem { char: "🤍", name: "White heart", category: "Smileys", keywords: &["pure", "peace"] },
        EmojiItem { char: "💔", name: "Broken heart", category: "Smileys", keywords: &["breakup", "heartbreak"] },

        // === PEOPLE & BODY ===
        EmojiItem { char: "👋", name: "Waving hand", category: "People", keywords: &["hello", "goodbye", "wave"] },
        EmojiItem { char: "🤚", name: "Raised back of hand", category: "People", keywords: &["stop", "halt"] },
        EmojiItem { char: "🖐️", name: "Hand with fingers splayed", category: "People", keywords: &["highfive", "five"] },
        EmojiItem { char: "✋", name: "Raised hand", category: "People", keywords: &["stop", "paper"] },
        EmojiItem { char: "🖖", name: "Vulcan salute", category: "People", keywords: &["spock", "star trek"] },
        EmojiItem { char: "👌", name: "OK hand", category: "People", keywords: &["okay", "perfect", "deal"] },
        EmojiItem { char: "🤌", name: "Pinched fingers", category: "People", keywords: &["italian", "chef kiss"] },
        EmojiItem { char: "🤏", name: "Pinching hand", category: "People", keywords: &["tiny", "small", "little"] },
        EmojiItem { char: "✌️", name: "Victory hand", category: "People", keywords: &["peace", "two"] },
        EmojiItem { char: "🤞", name: "Crossed fingers", category: "People", keywords: &["luck", "hope"] },
        EmojiItem { char: "🫰", name: "Hand with index and thumb crossed", category: "People", keywords: &["kpop", "heart", "finger"] },
        EmojiItem { char: "🤟", name: "Love-you gesture", category: "People", keywords: &["ily", "sign language"] },
        EmojiItem { char: "🤘", name: "Sign of the horns", category: "People", keywords: &["rock on", "metal"] },
        EmojiItem { char: "🤙", name: "Call me hand", category: "People", keywords: &["shaka", "hang loose"] },
        EmojiItem { char: "👈", name: "Backhand index pointing left", category: "People", keywords: &["point", "left"] },
        EmojiItem { char: "👉", name: "Backhand index pointing right", category: "People", keywords: &["point", "right"] },
        EmojiItem { char: "👆", name: "Backhand index pointing up", category: "People", keywords: &["point", "up"] },
        EmojiItem { char: "👇", name: "Backhand index pointing down", category: "People", keywords: &["point", "down"] },
        EmojiItem { char: "☝️", name: "Index pointing up", category: "People", keywords: &["first", "one"] },
        EmojiItem { char: "👍", name: "Thumbs up", category: "People", keywords: &["approve", "yes", "good", "+1"] },
        EmojiItem { char: "👎", name: "Thumbs down", category: "People", keywords: &["disapprove", "no", "bad", "-1"] },
        EmojiItem { char: "✊", name: "Raised fist", category: "People", keywords: &["power", "solidarity"] },
        EmojiItem { char: "👊", name: "Oncoming fist", category: "People", keywords: &["brofist", "punch"] },
        EmojiItem { char: "👏", name: "Clapping hands", category: "People", keywords: &["applause", "bravo"] },
        EmojiItem { char: "🙌", name: "Raising hands", category: "People", keywords: &["praise", "celebration"] },
        EmojiItem { char: "👐", name: "Open hands", category: "People", keywords: &["hug", "open"] },
        EmojiItem { char: "🤲", name: "Palms up together", category: "People", keywords: &["prayer", "offering"] },
        EmojiItem { char: "🤝", name: "Handshake", category: "People", keywords: &["deal", "partner", "agree"] },
        EmojiItem { char: "🙏", name: "Folded hands", category: "People", keywords: &["please", "thank you", "pray", "namaste"] },
        EmojiItem { char: "✍️", name: "Writing hand", category: "People", keywords: &["write", "pen", "note"] },
        EmojiItem { char: "💅", name: "Nail polish", category: "People", keywords: &["glam", "slay", "care"] },
        EmojiItem { char: "🤳", name: "Selfie", category: "People", keywords: &["camera", "phone"] },
        EmojiItem { char: "💪", name: "Flexed biceps", category: "People", keywords: &["muscle", "strong", "gym"] },
        EmojiItem { char: "🧠", name: "Brain", category: "People", keywords: &["smart", "mind", "think"] },
        EmojiItem { char: "👀", name: "Eyes", category: "People", keywords: &["look", "see", "watch"] },
        EmojiItem { char: "👁️", name: "Eye", category: "People", keywords: &["vision", "stare"] },
        EmojiItem { char: "👶", name: "Baby", category: "People", keywords: &["infant", "child"] },
        EmojiItem { char: "🧒", name: "Child", category: "People", keywords: &["kid", "young"] },
        EmojiItem { char: "👦", name: "Boy", category: "People", keywords: &["male", "kid"] },
        EmojiItem { char: "👧", name: "Girl", category: "People", keywords: &["female", "kid"] },
        EmojiItem { char: "🧑", name: "Person", category: "People", keywords: &["human", "adult"] },
        EmojiItem { char: "👨", name: "Man", category: "People", keywords: &["guy", "adult"] },
        EmojiItem { char: "👩", name: "Woman", category: "People", keywords: &["female", "adult"] },

        // === ANIMALS & NATURE ===
        EmojiItem { char: "🐶", name: "Dog face", category: "Animals", keywords: &["puppy", "pet", "dog"] },
        EmojiItem { char: "🐱", name: "Cat face", category: "Animals", keywords: &["kitty", "meow", "cat"] },
        EmojiItem { char: "🐭", name: "Mouse face", category: "Animals", keywords: &["rodent", "cheese"] },
        EmojiItem { char: "🐹", name: "Hamster face", category: "Animals", keywords: &["pet", "cute"] },
        EmojiItem { char: "🐰", name: "Rabbit face", category: "Animals", keywords: &["bunny", "easter"] },
        EmojiItem { char: "🦊", name: "Fox face", category: "Animals", keywords: &["clever", "wild"] },
        EmojiItem { char: "🐻", name: "Bear face", category: "Animals", keywords: &["grizzly", "teddy"] },
        EmojiItem { char: "🐼", name: "Panda face", category: "Animals", keywords: &["bamboo", "china"] },
        EmojiItem { char: "🐨", name: "Koala", category: "Animals", keywords: &["australia", "cute"] },
        EmojiItem { char: "🐯", name: "Tiger face", category: "Animals", keywords: &["wild", "stripes", "cat"] },
        EmojiItem { char: "🦁", name: "Lion", category: "Animals", keywords: &["king", "mane", "safari"] },
        EmojiItem { char: "🐮", name: "Cow face", category: "Animals", keywords: &["moo", "milk", "farm"] },
        EmojiItem { char: "🐷", name: "Pig face", category: "Animals", keywords: &["oink", "bacon"] },
        EmojiItem { char: "🐸", name: "Frog", category: "Animals", keywords: &["toad", "ribbit", "green"] },
        EmojiItem { char: "🐵", name: "Monkey face", category: "Animals", keywords: &["chimp", "jungle"] },
        EmojiItem { char: "🐧", name: "Penguin", category: "Animals", keywords: &["bird", "ice", "antarctica"] },
        EmojiItem { char: "🦅", name: "Eagle", category: "Animals", keywords: &["bird", "soar", "usa"] },
        EmojiItem { char: "🦉", name: "Owl", category: "Animals", keywords: &["wise", "night", "bird"] },
        EmojiItem { char: "🦇", name: "Bat", category: "Animals", keywords: &["vampire", "cave", "night"] },
        EmojiItem { char: "🐺", name: "Wolf", category: "Animals", keywords: &["howl", "pack"] },
        EmojiItem { char: "🐗", name: "Boar", category: "Animals", keywords: &["wild", "pig"] },
        EmojiItem { char: "🐴", name: "Horse face", category: "Animals", keywords: &["steed", "equine"] },
        EmojiItem { char: "🦄", name: "Unicorn", category: "Animals", keywords: &["magic", "fantasy"] },
        EmojiItem { char: "🐝", name: "Honeybee", category: "Animals", keywords: &["honey", "buzz", "sting"] },
        EmojiItem { char: "🦋", name: "Butterfly", category: "Animals", keywords: &["wings", "caterpillar", "pretty"] },
        EmojiItem { char: "🐞", name: "Lady beetle", category: "Animals", keywords: &["ladybug", "red"] },
        EmojiItem { char: "🐢", name: "Turtle", category: "Animals", keywords: &["slow", "shell", "reptile"] },
        EmojiItem { char: "🐍", name: "Snake", category: "Animals", keywords: &["slither", "reptile"] },
        EmojiItem { char: "🐙", name: "Octopus", category: "Animals", keywords: &["tentacles", "sea"] },
        EmojiItem { char: "🐬", name: "Dolphin", category: "Animals", keywords: &["ocean", "marine"] },
        EmojiItem { char: "🐳", name: "Spouting whale", category: "Animals", keywords: &["sea", "huge"] },
        EmojiItem { char: "🦈", name: "Shark", category: "Animals", keywords: &["teeth", "ocean", "predator"] },
        EmojiItem { char: "🌸", name: "Cherry blossom", category: "Animals", keywords: &["flower", "sakura", "pink"] },
        EmojiItem { char: "🌹", name: "Rose", category: "Animals", keywords: &["flower", "romantic", "red"] },
        EmojiItem { char: "🌻", name: "Sunflower", category: "Animals", keywords: &["summer", "yellow"] },
        EmojiItem { char: "🌲", name: "Evergreen tree", category: "Animals", keywords: &["pine", "forest", "nature"] },
        EmojiItem { char: "🍀", name: "Four leaf clover", category: "Animals", keywords: &["lucky", "irish"] },

        // === FOOD & DRINK ===
        EmojiItem { char: "🍎", name: "Red apple", category: "Food", keywords: &["fruit", "healthy"] },
        EmojiItem { char: "🍌", name: "Banana", category: "Food", keywords: &["fruit", "yellow"] },
        EmojiItem { char: "🍉", name: "Watermelon", category: "Food", keywords: &["fruit", "summer"] },
        EmojiItem { char: "🍇", name: "Grapes", category: "Food", keywords: &["wine", "fruit"] },
        EmojiItem { char: "🍓", name: "Strawberry", category: "Food", keywords: &["berry", "sweet"] },
        EmojiItem { char: "🥑", name: "Avocado", category: "Food", keywords: &["guacamole", "healthy"] },
        EmojiItem { char: "🍕", name: "Pizza", category: "Food", keywords: &["cheese", "slice", "italian"] },
        EmojiItem { char: "🍔", name: "Hamburger", category: "Food", keywords: &["burger", "fast food"] },
        EmojiItem { char: "🍟", name: "French fries", category: "Food", keywords: &["fries", "snack"] },
        EmojiItem { char: "🌭", name: "Hot dog", category: "Food", keywords: &["frankfurter", "snack"] },
        EmojiItem { char: "🌮", name: "Taco", category: "Food", keywords: &["mexican", "shell"] },
        EmojiItem { char: "🌯", name: "Burrito", category: "Food", keywords: &["wrap", "mexican"] },
        EmojiItem { char: "🍣", name: "Sushi", category: "Food", keywords: &["japanese", "fish", "roll"] },
        EmojiItem { char: "🍜", name: "Steaming bowl", category: "Food", keywords: &["ramen", "noodles", "soup"] },
        EmojiItem { char: "🍝", name: "Spaghetti", category: "Food", keywords: &["pasta", "italian"] },
        EmojiItem { char: "🍞", name: "Bread", category: "Food", keywords: &["toast", "bakery"] },
        EmojiItem { char: "🧀", name: "Cheese wedge", category: "Food", keywords: &["dairy", "swiss"] },
        EmojiItem { char: "🍦", name: "Soft ice cream", category: "Food", keywords: &["dessert", "cone"] },
        EmojiItem { char: "🎂", name: "Birthday cake", category: "Food", keywords: &["cake", "celebrate"] },
        EmojiItem { char: "🍫", name: "Chocolate bar", category: "Food", keywords: &["candy", "sweet"] },
        EmojiItem { char: "🍿", name: "Popcorn", category: "Food", keywords: &["movie", "snack"] },
        EmojiItem { char: "☕", name: "Hot beverage", category: "Food", keywords: &["coffee", "tea", "espresso"] },
        EmojiItem { char: "🍵", name: "Teacup without handle", category: "Food", keywords: &["green tea", "matcha"] },
        EmojiItem { char: "🧋", name: "Bubble tea", category: "Food", keywords: &["boba", "milk tea"] },
        EmojiItem { char: "🍺", name: "Beer mug", category: "Food", keywords: &["drink", "alcohol", "pub"] },
        EmojiItem { char: "🍻", name: "Clinking beer mugs", category: "Food", keywords: &["cheers", "toast"] },
        EmojiItem { char: "🥂", name: "Clinking glasses", category: "Food", keywords: &["champagne", "celebrate"] },
        EmojiItem { char: "🍷", name: "Wine glass", category: "Food", keywords: &["red wine", "drink"] },

        // === TRAVEL & PLACES ===
        EmojiItem { char: "🚗", name: "Automobile", category: "Travel", keywords: &["car", "drive"] },
        EmojiItem { char: "🏎️", name: "Racing car", category: "Travel", keywords: &["f1", "speed", "fast"] },
        EmojiItem { char: "🚲", name: "Bicycle", category: "Travel", keywords: &["bike", "cycle"] },
        EmojiItem { char: "🛵", name: "Motor scooter", category: "Travel", keywords: &["vespa", "moped"] },
        EmojiItem { char: "🏍️", name: "Motorcycle", category: "Travel", keywords: &["motorbike", "ride"] },
        EmojiItem { char: "🚀", name: "Rocket", category: "Travel", keywords: &["launch", "space", "fast"] },
        EmojiItem { char: "✈️", name: "Airplane", category: "Travel", keywords: &["flight", "trip", "vacation"] },
        EmojiItem { char: "🚁", name: "Helicopter", category: "Travel", keywords: &["chopper", "fly"] },
        EmojiItem { char: "🚆", name: "Train", category: "Travel", keywords: &["railway", "commute"] },
        EmojiItem { char: "🚇", name: "Metro", category: "Travel", keywords: &["subway", "underground"] },
        EmojiItem { char: "🚢", name: "Ship", category: "Travel", keywords: &["cruise", "boat", "sea"] },
        EmojiItem { char: "⛵", name: "Sailboat", category: "Travel", keywords: &["sailing", "wind"] },
        EmojiItem { char: "🏠", name: "House", category: "Travel", keywords: &["home", "building"] },
        EmojiItem { char: "🏢", name: "Office building", category: "Travel", keywords: &["work", "city"] },
        EmojiItem { char: "🏖️", name: "Beach with umbrella", category: "Travel", keywords: &["sand", "sea", "summer"] },
        EmojiItem { char: "⛰️", name: "Mountain", category: "Travel", keywords: &["hike", "nature"] },
        EmojiItem { char: "🌋", name: "Volcano", category: "Travel", keywords: &["lava", "eruption"] },
        EmojiItem { char: "🏕️", name: "Camping", category: "Travel", keywords: &["tent", "outdoor"] },

        // === ACTIVITIES & SPORTS ===
        EmojiItem { char: "⚽", name: "Soccer ball", category: "Activities", keywords: &["football", "sports"] },
        EmojiItem { char: "🏀", name: "Basketball", category: "Activities", keywords: &["nba", "hoop", "dunk"] },
        EmojiItem { char: "🏈", name: "American football", category: "Activities", keywords: &["nfl", "superbowl"] },
        EmojiItem { char: "⚾", name: "Baseball", category: "Activities", keywords: &["mlb", "strike"] },
        EmojiItem { char: "🎾", name: "Tennis", category: "Activities", keywords: &["racket", "match"] },
        EmojiItem { char: "🏐", name: "Volleyball", category: "Activities", keywords: &["beach", "spike"] },
        EmojiItem { char: "🏓", name: "Ping pong", category: "Activities", keywords: &["table tennis"] },
        EmojiItem { char: "🏸", name: "Badminton", category: "Activities", keywords: &["shuttlecock"] },
        EmojiItem { char: "🥊", name: "Boxing glove", category: "Activities", keywords: &["fight", "punch"] },
        EmojiItem { char: "🥋", name: "Martial arts uniform", category: "Activities", keywords: &["karate", "judo"] },
        EmojiItem { char: "🎮", name: "Video game controller", category: "Activities", keywords: &["gaming", "console", "play"] },
        EmojiItem { char: "🎯", name: "Bullseye", category: "Activities", keywords: &["target", "darts", "aim"] },
        EmojiItem { char: "🎲", name: "Game die", category: "Activities", keywords: &["dice", "board game", "luck"] },
        EmojiItem { char: "♟️", name: "Chess pawn", category: "Activities", keywords: &["strategy", "chess"] },
        EmojiItem { char: "🎨", name: "Artist palette", category: "Activities", keywords: &["paint", "art", "draw"] },
        EmojiItem { char: "🎬", name: "Clapper board", category: "Activities", keywords: &["movie", "film", "cinema"] },
        EmojiItem { char: "🎤", name: "Microphone", category: "Activities", keywords: &["sing", "karaoke"] },
        EmojiItem { char: "🎧", name: "Headphone", category: "Activities", keywords: &["music", "audio", "listen"] },
        EmojiItem { char: "🎸", name: "Guitar", category: "Activities", keywords: &["rock", "instrument"] },
        EmojiItem { char: "🎹", name: "Musical keyboard", category: "Activities", keywords: &["piano", "music"] },
        EmojiItem { char: "🏆", name: "Trophy", category: "Activities", keywords: &["winner", "champion", "first"] },
        EmojiItem { char: "🥇", name: "1st place medal", category: "Activities", keywords: &["gold", "first", "winner"] },
        EmojiItem { char: "🥈", name: "2nd place medal", category: "Activities", keywords: &["silver", "second"] },
        EmojiItem { char: "🥉", name: "3rd place medal", category: "Activities", keywords: &["bronze", "third"] },

        // === OBJECTS & TECH ===
        EmojiItem { char: "💻", name: "Laptop", category: "Objects", keywords: &["computer", "tech", "macbook", "pc"] },
        EmojiItem { char: "🖥️", name: "Desktop computer", category: "Objects", keywords: &["monitor", "screen"] },
        EmojiItem { char: "⌨️", name: "Keyboard", category: "Objects", keywords: &["type", "typing"] },
        EmojiItem { char: "🖱️", name: "Computer mouse", category: "Objects", keywords: &["click", "cursor"] },
        EmojiItem { char: "📱", name: "Mobile phone", category: "Objects", keywords: &["iphone", "android", "cell"] },
        EmojiItem { char: "💡", name: "Light bulb", category: "Objects", keywords: &["idea", "bright", "smart"] },
        EmojiItem { char: "🔦", name: "Flashlight", category: "Objects", keywords: &["torch", "light"] },
        EmojiItem { char: "🔋", name: "Battery", category: "Objects", keywords: &["power", "charge"] },
        EmojiItem { char: "🔌", name: "Electric plug", category: "Objects", keywords: &["power", "outlet"] },
        EmojiItem { char: "🔑", name: "Key", category: "Objects", keywords: &["lock", "password", "security"] },
        EmojiItem { char: "🔒", name: "Locked", category: "Objects", keywords: &["secure", "padlock", "private"] },
        EmojiItem { char: "🔓", name: "Unlocked", category: "Objects", keywords: &["open", "insecure"] },
        EmojiItem { char: "🧰", name: "Toolbox", category: "Objects", keywords: &["tools", "fix"] },
        EmojiItem { char: "🔧", name: "Wrench", category: "Objects", keywords: &["tool", "fix", "settings"] },
        EmojiItem { char: "🔨", name: "Hammer", category: "Objects", keywords: &["build", "tool"] },
        EmojiItem { char: "⚙️", name: "Gear", category: "Objects", keywords: &["settings", "cog", "preferences"] },
        EmojiItem { char: "📷", name: "Camera", category: "Objects", keywords: &["photo", "picture"] },
        EmojiItem { char: "📸", name: "Camera with flash", category: "Objects", keywords: &["snapshot", "screenshot"] },
        EmojiItem { char: "📦", name: "Package", category: "Objects", keywords: &["box", "parcel", "delivery"] },
        EmojiItem { char: "✉️", name: "Envelope", category: "Objects", keywords: &["mail", "letter", "email"] },
        EmojiItem { char: "📝", name: "Memo", category: "Objects", keywords: &["note", "write", "document"] },
        EmojiItem { char: "📌", name: "Pushpin", category: "Objects", keywords: &["pin", "pinned", "location"] },
        EmojiItem { char: "📎", name: "Paperclip", category: "Objects", keywords: &["attach", "attachment"] },
        EmojiItem { char: "💎", name: "Gem stone", category: "Objects", keywords: &["diamond", "jewel", "luxury"] },
        EmojiItem { char: "💰", name: "Money bag", category: "Objects", keywords: &["cash", "dollar", "wealth"] },

        // === SYMBOLS ===
        EmojiItem { char: "⚠️", name: "Warning", category: "Symbols", keywords: &["alert", "caution", "danger"] },
        EmojiItem { char: "⛔", name: "No entry", category: "Symbols", keywords: &["stop", "denied", "forbidden"] },
        EmojiItem { char: "🚫", name: "Prohibited", category: "Symbols", keywords: &["ban", "no"] },
        EmojiItem { char: "✅", name: "Check mark button", category: "Symbols", keywords: &["done", "ok", "success", "yes"] },
        EmojiItem { char: "❌", name: "Cross mark", category: "Symbols", keywords: &["error", "cancel", "wrong", "no"] },
        EmojiItem { char: "❓", name: "Question mark", category: "Symbols", keywords: &["help", "what", "query"] },
        EmojiItem { char: "❗", name: "Exclamation mark", category: "Symbols", keywords: &["important", "attention"] },
        EmojiItem { char: "➕", name: "Plus sign", category: "Symbols", keywords: &["add", "more"] },
        EmojiItem { char: "➖", name: "Minus sign", category: "Symbols", keywords: &["subtract", "less"] },
        EmojiItem { char: "ℹ️", name: "Information", category: "Symbols", keywords: &["info", "help"] },
        EmojiItem { char: "♾️", name: "Infinity", category: "Symbols", keywords: &["forever", "loop"] },
        EmojiItem { char: "🔁", name: "Repeat button", category: "Symbols", keywords: &["loop", "again"] },
        EmojiItem { char: "🔀", name: "Shuffle tracks button", category: "Symbols", keywords: &["random"] },
        EmojiItem { char: "⚡", name: "High voltage", category: "Symbols", keywords: &["lightning", "fast", "power", "bolt"] },

        // === FLAGS ===
        EmojiItem { char: "🏁", name: "Chequered flag", category: "Flags", keywords: &["race", "finish"] },
        EmojiItem { char: "🚩", name: "Triangular flag", category: "Flags", keywords: &["red flag", "warning"] },
        EmojiItem { char: "🏳️‍🌈", name: "Rainbow flag", category: "Flags", keywords: &["pride", "lgbt"] },
        EmojiItem { char: "🏳️‍⚧️", name: "Transgender flag", category: "Flags", keywords: &["pride", "trans"] },
        EmojiItem { char: "🏴‍☠️", name: "Pirate flag", category: "Flags", keywords: &["jolly roger", "skull"] },
        EmojiItem { char: "🇺🇸", name: "Flag: United States", category: "Flags", keywords: &["usa", "america"] },
        EmojiItem { char: "🇬🇧", name: "Flag: United Kingdom", category: "Flags", keywords: &["uk", "britain"] },
        EmojiItem { char: "🇨🇦", name: "Flag: Canada", category: "Flags", keywords: &["canada"] },
        EmojiItem { char: "🇦🇺", name: "Flag: Australia", category: "Flags", keywords: &["australia"] },
        EmojiItem { char: "🇩🇪", name: "Flag: Germany", category: "Flags", keywords: &["germany"] },
        EmojiItem { char: "🇫🇷", name: "Flag: France", category: "Flags", keywords: &["france"] },
        EmojiItem { char: "🇯🇵", name: "Flag: Japan", category: "Flags", keywords: &["japan"] },
        EmojiItem { char: "🇰🇷", name: "Flag: South Korea", category: "Flags", keywords: &["korea"] },
        EmojiItem { char: "🇮🇳", name: "Flag: India", category: "Flags", keywords: &["india"] },
        EmojiItem { char: "🇧🇷", name: "Flag: Brazil", category: "Flags", keywords: &["brazil"] },
        EmojiItem { char: "🇮🇹", name: "Flag: Italy", category: "Flags", keywords: &["italy"] },
        EmojiItem { char: "🇪🇸", name: "Flag: Spain", category: "Flags", keywords: &["spain"] },
        EmojiItem { char: "🇪🇺", name: "Flag: European Union", category: "Flags", keywords: &["europe", "eu"] },
    ]
}

/// Helper to resolve shortcode or emoji using emojic
pub fn resolve_emojic_shortcode(query: &str) -> Option<&'static str> {
    let clean = query.trim().trim_matches(':');
    if clean.is_empty() {
        return None;
    }
    let formatted = format!(":{}:", clean);
    emojic::parse_alias(&formatted).map(|e| e.grapheme)
}

/// Directory for storing clipboard screenshot images
pub fn get_images_cache_dir() -> PathBuf {
    let dir = dirs::cache_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("nexus")
        .join("images");
    let _ = std::fs::create_dir_all(&dir);
    dir
}

/// The main application state holding everything required for the Windows 11-style UI
pub struct NexusState {
    /// Active navigation tab
    pub view_mode: ViewMode,
    /// Active theme (defaults to system theme)
    pub theme: ThemeMode,
    /// Search query in search bar
    pub search_query: String,
    /// Clipboard entries loaded from storage
    pub clipboard_entries: Vec<ClipboardEntry>,
    /// Active category filter for emoji/kaomoji/symbols
    pub selected_category: String,
    /// Toast message shown temporarily
    pub toast_message: Option<String>,
    /// Last copied text content for duplicate prevention
    pub last_clipboard_text: Option<String>,
    /// Last copied image hash for duplicate prevention
    pub last_clipboard_image_hash: Option<String>,
    /// List of recently used emojis (pinned in dedicated top box)
    pub recent_emojis: Vec<String>,
    /// SQLite storage connection
    pub storage: Option<SqliteStorage>,
    /// Last seen database entry count for instant change detection
    pub last_db_count: usize,
    /// Last seen max created_at timestamp for instant change detection
    pub last_db_max_created_at: i64,
}

impl Default for NexusState {
    fn default() -> Self {
        Self {
            view_mode: ViewMode::Clipboard,
            theme: detect_system_theme(),
            search_query: String::new(),
            clipboard_entries: vec![],
            selected_category: "All".to_string(),
            toast_message: None,
            last_clipboard_text: None,
            last_clipboard_image_hash: None,
            recent_emojis: vec![
                "👍".to_string(),
                "❤️".to_string(),
                "😂".to_string(),
                "✨".to_string(),
                "🔥".to_string(),
                "🎉".to_string(),
                "😊".to_string(),
                "🙏".to_string(),
            ],
            storage: None,
            last_db_count: 0,
            last_db_max_created_at: 0,
        }
    }
}

impl NexusState {
    /// Initialize with storage and sample/initial entries
    pub fn new() -> Self {
        let mut state = Self::default();
        state.init_storage_and_seed();
        state
    }

    /// Set up SQLite storage and seed initial entries if empty
    pub fn init_storage_and_seed(&mut self) {
        let db_dir = dirs::data_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("nexus");
        let _ = std::fs::create_dir_all(&db_dir);
        let db_path = db_dir.join("nexus.db");

        if let Ok(mut storage) = SqliteStorage::open(&db_path) {
            let entries = storage.get_entries().unwrap_or_default();
            if entries.is_empty() {
                let now = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs() as i64;

                // Ensure an initial sample image exists
                let sample_img_path = get_images_cache_dir().join("sample_screenshot.png");
                if !sample_img_path.exists() {
                    // Try copying existing test/screenshot or create a small fallback PNG
                    if Path::new("/home/sahilcodex/Documents/nexus/user_attached_image.png").exists() {
                        let _ = std::fs::copy(
                            "/home/sahilcodex/Documents/nexus/user_attached_image.png",
                            &sample_img_path,
                        );
                    }
                }

                let mut samples: Vec<(String, ContentType, bool, i64)> = vec![
                    ("Screenshots & copied images show up right here in your history".to_string(), ContentType::Text, false, now - 60),
                    ("Tip: Press Super + V to open Nexus Panel anytime".to_string(), ContentType::Text, true, now - 180),
                    ("https://github.com/zed-industries/gpui".to_string(), ContentType::Url, false, now - 360),
                ];

                if sample_img_path.exists() {
                    samples.insert(
                        0,
                        (
                            sample_img_path.to_string_lossy().to_string(),
                            ContentType::Image,
                            false,
                            now - 30,
                        ),
                    );
                }

                for (idx, (content, ctype, pinned, ts)) in samples.into_iter().enumerate() {
                    let entry = ClipboardEntry {
                        id: (idx + 1) as u64,
                        content_type: ctype,
                        content: content.clone(),
                        content_hash: format!("{:x}", idx),
                        created_at: ts,
                        last_used_at: None,
                        is_pinned: pinned,
                    };
                    let _ = storage.save_entry(&entry);
                }
            }
            if let Ok(entries) = storage.get_entries() {
                self.clipboard_entries = entries;
                if let Ok((count, max_created)) = storage.get_stats() {
                    self.last_db_count = count;
                    self.last_db_max_created_at = max_created;
                }
            }
            if let Ok(Some(json_str)) = storage.get_setting("recent_emojis") {
                if let Ok(saved) = serde_json::from_str::<Vec<String>>(&json_str) {
                    if !saved.is_empty() {
                        self.recent_emojis = saved;
                    }
                }
            }
            self.storage = Some(storage);
        }
    }

    /// Refresh entries from storage
    pub fn refresh_entries(&mut self) {
        if let Some(storage) = &self.storage {
            if let Ok(entries) = storage.get_entries() {
                self.clipboard_entries = entries;
                if let Ok((count, max_created)) = storage.get_stats() {
                    self.last_db_count = count;
                    self.last_db_max_created_at = max_created;
                }
            }
        }
    }

    /// Check if database has new or modified entries and refresh if needed
    pub fn check_for_database_updates(&mut self) -> bool {
        if let Some(storage) = &self.storage {
            if let Ok((count, max_created)) = storage.get_stats() {
                if count != self.last_db_count || max_created != self.last_db_max_created_at {
                    self.last_db_count = count;
                    self.last_db_max_created_at = max_created;
                    if let Ok(entries) = storage.get_entries() {
                        self.clipboard_entries = entries;
                        return true;
                    }
                }
            }
        }
        false
    }

    /// Set navigation tab
    pub fn set_view_mode(&mut self, mode: ViewMode) {
        self.view_mode = mode;
        self.search_query.clear();
        self.selected_category = "All".to_string();
    }

    /// Toggle pin status of an entry
    pub fn toggle_pin(&mut self, id: u64) {
        if let Some(storage) = &mut self.storage {
            let _ = storage.toggle_pin(id);
            self.refresh_entries();
        }
    }

    /// Delete a single entry
    pub fn delete_entry(&mut self, id: u64) {
        if let Some(storage) = &mut self.storage {
            let _ = storage.delete_entry(id);
            self.refresh_entries();
        }
    }

    /// Clear unpinned entries
    pub fn clear_unpinned(&mut self) {
        if let Some(storage) = &mut self.storage {
            let _ = storage.clear_unpinned();
            self.refresh_entries();
        }
    }

    /// Toggle light / dark theme
    pub fn toggle_theme(&mut self) {
        self.theme = match self.theme {
            ThemeMode::Light => ThemeMode::Dark,
            ThemeMode::Dark => ThemeMode::Light,
        };
    }

    /// Copy text to system clipboard (Wayland or X11) and show toast
    pub fn copy_text(&mut self, text: &str) {
        copy_to_system_clipboard(text);
        self.toast_message = Some("Copied to clipboard ✓".to_string());

        // If in clipboard mode, update or add to storage
        if let Some(storage) = &mut self.storage {
            let now = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs() as i64;
            let id = (self.clipboard_entries.len() as u64) + 1;
            let ctype = if text.starts_with("http://") || text.starts_with("https://") {
                ContentType::Url
            } else {
                ContentType::Text
            };

            // Avoid duplicate of identical recent text
            if !self.clipboard_entries.iter().any(|e| e.content == text) {
                let entry = ClipboardEntry {
                    id,
                    content_type: ctype,
                    content: text.to_string(),
                    content_hash: format!("{:x?}", text),
                    created_at: now,
                    last_used_at: Some(now),
                    is_pinned: false,
                };
                let _ = storage.save_entry(&entry);
                self.refresh_entries();
            }
        }
    }

    /// Use an emoji: copy to system clipboard and prepend to recent_emojis top box
    pub fn use_emoji(&mut self, emoji: &str) {
        self.copy_text(emoji);

        // Update recent emojis
        self.recent_emojis.retain(|e| e != emoji);
        self.recent_emojis.insert(0, emoji.to_string());
        self.recent_emojis.truncate(14);

        // Save to sqlite storage
        if let Some(storage) = &mut self.storage {
            if let Ok(json_str) = serde_json::to_string(&self.recent_emojis) {
                let _ = storage.save_setting("recent_emojis", &json_str);
            }
        }
    }

    /// Copy image file to system clipboard (Wayland or X11) and show toast
    pub fn copy_image(&mut self, path_str: &str) {
        let path = Path::new(path_str);
        if path.exists() {
            copy_image_to_system_clipboard(path);
            self.toast_message = Some("Copied image to clipboard ✓".to_string());
        }
    }

    /// Check if system clipboard has new text or image and import it
    pub fn check_system_clipboard_sync(&mut self) {
        // 1. First check if DB had updates from background daemon
        self.check_for_database_updates();

        // 2. Direct Wayland inspection
        if std::env::var_os("WAYLAND_DISPLAY").is_some() {
            if let Ok(output) = Command::new("wl-paste").arg("--list-types").output() {
                if output.status.success() {
                    let types = String::from_utf8_lossy(&output.stdout);
                    if types.contains("image/") {
                        if let Some(img_bytes) = get_system_clipboard_image() {
                            let len = img_bytes.len();
                            let hash = format!("img_{:x}", len);
                            if self.last_clipboard_image_hash.as_deref() != Some(&hash) {
                                self.last_clipboard_image_hash = Some(hash.clone());
                                let cache_dir = get_images_cache_dir();
                                let now = SystemTime::now()
                                    .duration_since(UNIX_EPOCH)
                                    .unwrap_or_default()
                                    .as_secs() as i64;
                                let file_path = cache_dir.join(format!("clip_{}_{}.png", now, len));
                                if std::fs::write(&file_path, &img_bytes).is_ok() {
                                    let path_str = file_path.to_string_lossy().to_string();
                                    if let Some(storage) = &mut self.storage {
                                        let entry = ClipboardEntry {
                                            id: 0,
                                            content_type: ContentType::Image,
                                            content: path_str,
                                            content_hash: hash,
                                            created_at: now,
                                            last_used_at: None,
                                            is_pinned: false,
                                        };
                                        let _ = storage.save_entry(&entry);
                                        self.refresh_entries();
                                    }
                                }
                            }
                        }
                        return;
                    } else if types.contains("text/") || types.contains("STRING") {
                        if let Some(text) = get_system_clipboard_text() {
                            if !text.is_empty() && self.last_clipboard_text.as_deref() != Some(&text) {
                                self.last_clipboard_text = Some(text.clone());
                                if let Some(storage) = &mut self.storage {
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
                                    self.refresh_entries();
                                }
                            }
                        }
                        return;
                    }
                }
            }
        }

        // 3. Fallback for X11 environments
        if let Some(text) = get_system_clipboard_text() {
            if !text.is_empty() && self.last_clipboard_text.as_deref() != Some(&text) {
                self.last_clipboard_text = Some(text.clone());
                if let Some(storage) = &mut self.storage {
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
                    self.refresh_entries();
                }
            }
        }
    }
}

/// Copy text to Linux system clipboard via wl-copy (Wayland) with xclip fallback
pub fn copy_to_system_clipboard(text: &str) {
    if let Ok(mut child) = Command::new("wl-copy")
        .stdin(Stdio::piped())
        .spawn()
    {
        if let Some(mut stdin) = child.stdin.take() {
            let _ = stdin.write_all(text.as_bytes());
        }
        let _ = child.wait();
        return;
    }

    // Fallback to xclip
    if let Ok(mut child) = Command::new("xclip")
        .arg("-selection")
        .arg("clipboard")
        .stdin(Stdio::piped())
        .spawn()
    {
        if let Some(mut stdin) = child.stdin.take() {
            let _ = stdin.write_all(text.as_bytes());
        }
        let _ = child.wait();
    }
}

/// Copy image to Linux system clipboard via wl-copy (Wayland) with xclip fallback
pub fn copy_image_to_system_clipboard(image_path: &Path) {
    if !image_path.exists() {
        return;
    }

    if let Ok(file) = std::fs::File::open(image_path) {
        if let Ok(mut child) = Command::new("wl-copy")
            .arg("--type")
            .arg("image/png")
            .stdin(Stdio::from(file))
            .spawn()
        {
            let _ = child.wait();
            return;
        }
    }

    if let Ok(file) = std::fs::File::open(image_path) {
        if let Ok(mut child) = Command::new("xclip")
            .args(["-selection", "clipboard", "-t", "image/png", "-i"])
            .stdin(Stdio::from(file))
            .spawn()
        {
            let _ = child.wait();
        }
    }
}

/// Read text from Linux system clipboard via wl-paste with xclip fallback
pub fn get_system_clipboard_text() -> Option<String> {
    if let Ok(output) = Command::new("wl-paste")
        .arg("--type")
        .arg("text/plain")
        .arg("--no-newline")
        .output()
    {
        if output.status.success() {
            if let Ok(s) = String::from_utf8(output.stdout) {
                if !s.trim().is_empty() {
                    return Some(s);
                }
            }
        }
    }

    if let Ok(output) = Command::new("xclip")
        .arg("-selection")
        .arg("clipboard")
        .arg("-o")
        .output()
    {
        if output.status.success() {
            if let Ok(s) = String::from_utf8(output.stdout) {
                if !s.trim().is_empty() {
                    return Some(s);
                }
            }
        }
    }

    None
}

/// Read image raw bytes from Linux system clipboard via wl-paste with xclip fallback
pub fn get_system_clipboard_image() -> Option<Vec<u8>> {
    // 1. Wayland check
    if let Ok(types_output) = Command::new("wl-paste")
        .arg("--list-types")
        .output()
    {
        if types_output.status.success() {
            let types_str = String::from_utf8_lossy(&types_output.stdout);
            for target in ["image/png", "image/jpeg", "image/webp", "image/bmp"] {
                if types_str.contains(target) {
                    if let Ok(img_output) = Command::new("wl-paste")
                        .arg("--type")
                        .arg(target)
                        .output()
                    {
                        if img_output.status.success() && !img_output.stdout.is_empty() {
                            return Some(img_output.stdout);
                        }
                    }
                }
            }
        }
    }

    // 2. X11 check
    if let Ok(types_output) = Command::new("xclip")
        .args(["-selection", "clipboard", "-t", "TARGETS", "-o"])
        .output()
    {
        if types_output.status.success() {
            let types_str = String::from_utf8_lossy(&types_output.stdout);
            for target in ["image/png", "image/jpeg", "image/bmp"] {
                if types_str.contains(target) {
                    if let Ok(img_output) = Command::new("xclip")
                        .args(["-selection", "clipboard", "-t", target, "-o"])
                        .output()
                    {
                        if img_output.status.success() && !img_output.stdout.is_empty() {
                            return Some(img_output.stdout);
                        }
                    }
                }
            }
        }
    }

    None
}

/// Convert unix epoch timestamp to friendly relative time string
pub fn relative_time_string(created_at: i64) -> String {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64;
    let diff = now - created_at;

    if diff < 10 {
        "Just now".to_string()
    } else if diff < 60 {
        format!("{}s ago", diff)
    } else if diff < 3600 {
        format!("{}m ago", diff / 60)
    } else if diff < 86400 {
        format!("{}h ago", diff / 3600)
    } else {
        format!("{}d ago", diff / 86400)
    }
}

/// Initialize the application
pub fn init_app() -> Result<(), Box<dyn std::error::Error>> {
    info!("Initializing Nexus Panel...");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_recent_emojis_use_and_ordering() {
        let mut state = NexusState::default();
        assert!(!state.recent_emojis.is_empty());

        // Use a new emoji not initially first
        state.use_emoji("🚀");
        assert_eq!(state.recent_emojis[0], "🚀");

        // Use an emoji already in list
        state.use_emoji("🔥");
        assert_eq!(state.recent_emojis[0], "🔥");
        // Ensure deduplication
        let fire_count = state.recent_emojis.iter().filter(|&e| e == "🔥").count();
        assert_eq!(fire_count, 1);

        // Ensure max length constraint
        assert!(state.recent_emojis.len() <= 14);
    }
}
