use arboard::Clipboard;
use enigo::{
    Direction::{Click, Press, Release},
    Enigo, Key, Keyboard, Settings,
};
use parking_lot::Mutex;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Duration;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TypeMethod {
    Keystroke,
    Clipboard,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContextHeuristic {
    None,
    CodeBlock,
    PasswordField,
}

impl Default for ContextHeuristic {
    fn default() -> Self {
        Self::None
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TypeOptions {
    pub method: TypeMethod,
    pub throttle_ms: u64,
    pub newline_append: bool,
    pub clipboard_fallback: bool,
    pub detect_code_context: bool,
    pub detect_password_fields: bool,
}

impl Default for TypeOptions {
    fn default() -> Self {
        Self {
            method: TypeMethod::Keystroke,
            throttle_ms: 0,
            newline_append: false,
            clipboard_fallback: true,
            detect_code_context: true,
            detect_password_fields: true,
        }
    }
}

pub struct Typer {
    enigo: Arc<Mutex<Enigo>>,
    options: TypeOptions,
    modifiers_held: AtomicU64,
    last_type_time: AtomicU64,
    newline_pending: AtomicBool,
}

impl Typer {
    pub fn new(options: TypeOptions) -> Result<Self, enigo::NewConError> {
        let enigo = Enigo::new(&Settings::default())?;
        Ok(Self {
            enigo: Arc::new(Mutex::new(enigo)),
            options,
            modifiers_held: AtomicU64::new(0),
            last_type_time: AtomicU64::new(0),
            newline_pending: AtomicBool::new(false),
        })
    }

    pub fn with_defaults() -> Result<Self, enigo::NewConError> {
        Self::new(TypeOptions::default())
    }

    fn release_all_modifiers(&self) {
        let mut enigo = self.enigo.lock();
        if self.modifiers_held.load(Ordering::SeqCst) & 0x01 != 0 {
            let _ = enigo.key(Key::Shift, Release);
        }
        if self.modifiers_held.load(Ordering::SeqCst) & 0x02 != 0 {
            let _ = enigo.key(Key::Control, Release);
        }
        if self.modifiers_held.load(Ordering::SeqCst) & 0x04 != 0 {
            let _ = enigo.key(Key::Alt, Release);
        }
        if self.modifiers_held.load(Ordering::SeqCst) & 0x08 != 0 {
            let _ = enigo.key(Key::Meta, Release);
        }
        self.modifiers_held.store(0, Ordering::SeqCst);
    }

    fn type_via_keystroke(&self, text: &str) -> Result<(), String> {
        self.release_all_modifiers();

        let mut enigo = self.enigo.lock();

        for c in text.chars() {
            if self.options.throttle_ms > 0 {
                let now = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_millis() as u64;
                let last = self.last_type_time.load(Ordering::SeqCst);
                if now.saturating_sub(last) < self.options.throttle_ms {
                    std::thread::sleep(Duration::from_millis(
                        self.options
                            .throttle_ms
                            .saturating_sub(now.saturating_sub(last)),
                    ));
                }
                self.last_type_time.store(
                    std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap()
                        .as_millis() as u64,
                    Ordering::SeqCst,
                );
            }

            match c {
                'a'..='z' | 'A'..='Z' | '0'..='9' | ' ' => {
                    if c.is_uppercase() {
                        let _ = enigo.key(Key::Shift, Press);
                        self.modifiers_held.fetch_or(0x01, Ordering::SeqCst);
                    }
                    let _ = enigo.key(Key::Unicode(c.to_ascii_lowercase()), Click);
                    if c.is_uppercase() {
                        let _ = enigo.key(Key::Shift, Release);
                        self.modifiers_held.fetch_and(!0x01, Ordering::SeqCst);
                    }
                }
                '\n' => {
                    let _ = enigo.key(Key::Return, Click);
                }
                '\t' => {
                    let _ = enigo.key(Key::Tab, Click);
                }
                '.' | ',' | '!' | '?' | ':' | ';' | '\'' | '"' | '-' | '(' | ')' => {
                    self.type_punctuation(&mut enigo, c)?;
                }
                _ => {
                    let _ = enigo.key(Key::Unicode(c), Click);
                }
            }
        }

        self.release_all_modifiers();
        Ok(())
    }

    fn type_punctuation(&self, enigo: &mut Enigo, c: char) -> Result<(), String> {
        match c {
            '!' | '?' | ':' | '"' | '(' | ')' => {
                let _ = enigo.key(Key::Shift, Press);
                self.modifiers_held.fetch_or(0x01, Ordering::SeqCst);
            }
            _ => {}
        }

        let key = match c {
            '.' => Key::Unicode('.'),
            ',' => Key::Unicode(','),
            '!' => Key::Unicode('1'),
            '?' => Key::Unicode('?'),
            ':' => Key::Unicode(':'),
            ';' => Key::Unicode(';'),
            '\'' => Key::Unicode('\''),
            '"' => Key::Unicode('"'),
            '-' => Key::Unicode('-'),
            '(' => Key::Unicode('('),
            ')' => Key::Unicode(')'),
            _ => Key::Unicode(c),
        };

        let _ = enigo.key(key, Click);

        if "!?:\"()".contains(c) {
            let _ = enigo.key(Key::Shift, Release);
            self.modifiers_held.fetch_and(!0x01, Ordering::SeqCst);
        }

        Ok(())
    }

    fn type_via_clipboard(&self, text: &str) -> Result<(), String> {
        let mut clipboard = Clipboard::new().map_err(|e| e.to_string())?;

        let previous = clipboard.get_text().ok();

        clipboard.set_text(text).map_err(|e| e.to_string())?;

        std::thread::sleep(Duration::from_millis(50));

        let mut enigo = self.enigo.lock();
        let _ = enigo.key(Key::Control, Press);
        self.modifiers_held.fetch_or(0x02, Ordering::SeqCst);
        let _ = enigo.key(Key::Unicode('v'), Click);
        std::thread::sleep(Duration::from_millis(50));
        let _ = enigo.key(Key::Control, Release);
        self.modifiers_held.fetch_and(!0x02, Ordering::SeqCst);

        std::thread::sleep(Duration::from_millis(50));

        drop(enigo);

        if let Some(prev) = previous {
            let _ = clipboard.set_text(prev);
        }

        Ok(())
    }

    pub fn type_text(&self, text: &str) -> Result<(), String> {
        let context = self.detect_context(text);
        if context == ContextHeuristic::PasswordField {
            return Err("Skipping typing in password field".to_string());
        }

        let mut text = text.to_string();

        if self.options.newline_append && !text.ends_with('\n') {
            text.push('\n');
        }

        if self.options.method == TypeMethod::Clipboard {
            return self.type_via_clipboard(&text);
        }

        match self.type_via_keystroke(&text) {
            Ok(()) => Ok(()),
            Err(e) => {
                if self.options.clipboard_fallback {
                    self.type_via_clipboard(&text)
                } else {
                    Err(e)
                }
            }
        }
    }

    pub fn set_options(&mut self, options: TypeOptions) {
        self.options = options;
    }

    pub fn get_options(&self) -> TypeOptions {
        self.options
    }

    fn detect_context(&self, text: &str) -> ContextHeuristic {
        if !self.options.detect_code_context && !self.options.detect_password_fields {
            return ContextHeuristic::None;
        }

        let code_indicators = [
            "```",
            "``",
            "`",
            "fn ",
            "func ",
            "def ",
            "class ",
            "const ",
            "let ",
            "var ",
            "import ",
            "export ",
            "return ",
            "if ",
            "else ",
            "for ",
            "while ",
            "match ",
            "pub ",
            "struct ",
            "enum ",
            "impl ",
            "trait ",
            "type ",
            "interface ",
            "=>",
            "->",
            "::",
            "&&",
            "||",
            "==",
            "!=",
            "===",
            "!==",
        ];

        let first_line = text.lines().next().unwrap_or("").trim();
        let code_score: usize = code_indicators
            .iter()
            .filter(|&ind| text.contains(ind) || first_line.starts_with(ind))
            .count();

        let has_braces = text.contains('{') && text.contains('}');
        let has_brackets = text.contains('[') && text.contains(']');
        let has_parens = text.contains('(') && text.contains(')');
        let has_semicolons = text.contains(';');
        let indented_lines = text.lines().filter(|l| l.starts_with("    ")).count();
        let total_lines = text.lines().count();

        let is_code = code_score >= 2
            || (has_braces && has_semicolons)
            || (indented_lines > 2 && total_lines > 3)
            || (has_brackets && has_parens && text.contains(','));

        if is_code && self.options.detect_code_context {
            return ContextHeuristic::CodeBlock;
        }

        if self.options.detect_password_fields {
            let lower = text.to_lowercase();
            if lower.contains("password") || lower.contains("passwd") || lower.contains("secret") {
                return ContextHeuristic::PasswordField;
            }

            let masked_chars = text
                .chars()
                .filter(|&c| c == '•' || c == '*' || c == '•')
                .count();
            if masked_chars > 3 {
                return ContextHeuristic::PasswordField;
            }
        }

        ContextHeuristic::None
    }

    pub fn detect_context_for_options(options: &TypeOptions, text: &str) -> ContextHeuristic {
        if !options.detect_code_context && !options.detect_password_fields {
            return ContextHeuristic::None;
        }

        let code_indicators = [
            "```",
            "``",
            "`",
            "fn ",
            "func ",
            "def ",
            "class ",
            "const ",
            "let ",
            "var ",
            "import ",
            "export ",
            "return ",
            "if ",
            "else ",
            "for ",
            "while ",
            "match ",
            "pub ",
            "struct ",
            "enum ",
            "impl ",
            "trait ",
            "type ",
            "interface ",
            "=>",
            "->",
            "::",
            "&&",
            "||",
            "==",
            "!=",
            "===",
            "!==",
        ];

        let first_line = text.lines().next().unwrap_or("").trim();
        let code_score: usize = code_indicators
            .iter()
            .filter(|&ind| text.contains(ind) || first_line.starts_with(ind))
            .count();

        let has_braces = text.contains('{') && text.contains('}');
        let has_brackets = text.contains('[') && text.contains(']');
        let has_parens = text.contains('(') && text.contains(')');
        let has_semicolons = text.contains(';');
        let indented_lines = text.lines().filter(|l| l.starts_with("    ")).count();
        let total_lines = text.lines().count();

        let is_code = code_score >= 2
            || (has_braces && has_semicolons)
            || (indented_lines > 2 && total_lines > 3)
            || (has_brackets && has_parens && text.contains(','));

        if is_code && options.detect_code_context {
            return ContextHeuristic::CodeBlock;
        }

        if options.detect_password_fields {
            let lower = text.to_lowercase();
            if lower.contains("password") || lower.contains("passwd") || lower.contains("secret") {
                return ContextHeuristic::PasswordField;
            }

            let masked_chars = text
                .chars()
                .filter(|&c| c == '•' || c == '*' || c == '•')
                .count();
            if masked_chars > 3 {
                return ContextHeuristic::PasswordField;
            }
        }

        ContextHeuristic::None
    }

    pub fn should_type_for_options(options: &TypeOptions, text: &str) -> bool {
        match Self::detect_context_for_options(options, text) {
            ContextHeuristic::PasswordField => false,
            _ => true,
        }
    }

    pub fn should_type(&self, text: &str) -> bool {
        Self::should_type_for_options(&self.options, text)
    }

    pub fn get_context(&self, text: &str) -> ContextHeuristic {
        Self::detect_context_for_options(&self.options, text)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_type_options_default() {
        let options = TypeOptions::default();
        assert_eq!(options.method, TypeMethod::Keystroke);
        assert_eq!(options.throttle_ms, 0);
        assert!(!options.newline_append);
        assert!(options.clipboard_fallback);
        assert!(options.detect_code_context);
        assert!(options.detect_password_fields);
    }

    #[test]
    fn test_type_options_custom() {
        let options = TypeOptions {
            method: TypeMethod::Clipboard,
            throttle_ms: 10,
            newline_append: true,
            clipboard_fallback: false,
            detect_code_context: false,
            detect_password_fields: false,
        };
        assert_eq!(options.method, TypeMethod::Clipboard);
        assert_eq!(options.throttle_ms, 10);
        assert!(options.newline_append);
        assert!(!options.clipboard_fallback);
    }

    #[test]
    fn test_type_method_keystroke() {
        let options = TypeOptions {
            method: TypeMethod::Keystroke,
            ..Default::default()
        };
        assert_eq!(options.method, TypeMethod::Keystroke);
    }

    #[test]
    fn test_type_method_clipboard() {
        let options = TypeOptions {
            method: TypeMethod::Clipboard,
            ..Default::default()
        };
        assert_eq!(options.method, TypeMethod::Clipboard);
    }

    #[test]
    fn test_newline_append_option() {
        let options_with_newline = TypeOptions {
            newline_append: true,
            ..Default::default()
        };
        assert!(options_with_newline.newline_append);

        let options_without_newline = TypeOptions::default();
        assert!(!options_without_newline.newline_append);
    }

    #[test]
    fn test_clipboard_fallback_option() {
        let options_with_fallback = TypeOptions::default();
        assert!(options_with_fallback.clipboard_fallback);

        let options_without_fallback = TypeOptions {
            clipboard_fallback: false,
            ..Default::default()
        };
        assert!(!options_without_fallback.clipboard_fallback);
    }

    #[test]
    fn test_throttle_ms_option() {
        let options = TypeOptions {
            throttle_ms: 100,
            ..Default::default()
        };
        assert_eq!(options.throttle_ms, 100);

        let options_default = TypeOptions::default();
        assert_eq!(options_default.throttle_ms, 0);
    }

    #[test]
    fn test_type_options_all_keystroke() {
        let options = TypeOptions {
            method: TypeMethod::Keystroke,
            throttle_ms: 50,
            newline_append: false,
            clipboard_fallback: true,
            detect_code_context: true,
            detect_password_fields: true,
        };
        assert_eq!(options.method, TypeMethod::Keystroke);
        assert_eq!(options.throttle_ms, 50);
        assert!(!options.newline_append);
        assert!(options.clipboard_fallback);
    }

    #[test]
    fn test_type_options_all_clipboard() {
        let options = TypeOptions {
            method: TypeMethod::Clipboard,
            throttle_ms: 0,
            newline_append: true,
            clipboard_fallback: false,
            detect_code_context: false,
            detect_password_fields: false,
        };
        assert_eq!(options.method, TypeMethod::Clipboard);
        assert_eq!(options.throttle_ms, 0);
        assert!(options.newline_append);
        assert!(!options.clipboard_fallback);
    }

    #[test]
    fn test_context_heuristic_default() {
        let ctx = ContextHeuristic::default();
        assert_eq!(ctx, ContextHeuristic::None);
    }

    #[test]
    fn test_context_heuristic_variants() {
        assert_eq!(ContextHeuristic::None, ContextHeuristic::None);
        assert_eq!(ContextHeuristic::CodeBlock, ContextHeuristic::CodeBlock);
        assert_eq!(
            ContextHeuristic::PasswordField,
            ContextHeuristic::PasswordField
        );
    }

    #[test]
    fn test_detect_code_block_triple_backticks() {
        let options = TypeOptions::default();
        let ctx = Typer::detect_context_for_options(&options, "```javascript\nconst x = 1;\n```");
        assert_eq!(ctx, ContextHeuristic::CodeBlock);
    }

    #[test]
    fn test_detect_code_block_with_function() {
        let options = TypeOptions::default();
        let ctx =
            Typer::detect_context_for_options(&options, "function hello() {\n  return 'world';\n}");
        assert_eq!(ctx, ContextHeuristic::CodeBlock);
    }

    #[test]
    fn test_detect_password_field() {
        let options = TypeOptions::default();
        let ctx = Typer::detect_context_for_options(&options, "My password is secret");
        assert_eq!(ctx, ContextHeuristic::PasswordField);
    }

    #[test]
    fn test_detect_password_field_passwd() {
        let options = TypeOptions::default();
        let ctx = Typer::detect_context_for_options(&options, "Enter your passwd here");
        assert_eq!(ctx, ContextHeuristic::PasswordField);
    }

    #[test]
    fn test_normal_text_returns_none() {
        let options = TypeOptions::default();
        let ctx =
            Typer::detect_context_for_options(&options, "Hello world this is some normal text");
        assert_eq!(ctx, ContextHeuristic::None);
    }

    #[test]
    fn test_should_type_password_field() {
        let options = TypeOptions::default();
        assert!(!Typer::should_type_for_options(&options, "password123"));
    }

    #[test]
    fn test_should_type_normal_text() {
        let options = TypeOptions::default();
        assert!(Typer::should_type_for_options(&options, "Hello world"));
    }

    #[test]
    fn test_context_detection_disabled() {
        let options = TypeOptions {
            detect_code_context: false,
            detect_password_fields: false,
            ..Default::default()
        };
        let ctx = Typer::detect_context_for_options(&options, "function test() { return 1; }");
        assert_eq!(ctx, ContextHeuristic::None);
    }
}
