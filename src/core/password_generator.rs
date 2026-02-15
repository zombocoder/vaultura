use rand::Rng;

#[derive(Debug, Clone)]
pub struct PasswordConfig {
    pub length: usize,
    pub uppercase: bool,
    pub lowercase: bool,
    pub digits: bool,
    pub symbols: bool,
    pub exclude_ambiguous: bool,
}

impl Default for PasswordConfig {
    fn default() -> Self {
        Self {
            length: 20,
            uppercase: true,
            lowercase: true,
            digits: true,
            symbols: true,
            exclude_ambiguous: false,
        }
    }
}

const UPPERCASE: &str = "ABCDEFGHIJKLMNOPQRSTUVWXYZ";
const LOWERCASE: &str = "abcdefghijklmnopqrstuvwxyz";
const DIGITS: &str = "0123456789";
const SYMBOLS: &str = "!@#$%^&*()-_=+[]{}|;:,.<>?";
const AMBIGUOUS: &str = "0O1lI";

pub fn generate_password(config: &PasswordConfig) -> String {
    let mut charset = String::new();

    if config.uppercase {
        charset.push_str(UPPERCASE);
    }
    if config.lowercase {
        charset.push_str(LOWERCASE);
    }
    if config.digits {
        charset.push_str(DIGITS);
    }
    if config.symbols {
        charset.push_str(SYMBOLS);
    }

    if charset.is_empty() {
        charset.push_str(LOWERCASE);
    }

    if config.exclude_ambiguous {
        charset = charset
            .chars()
            .filter(|c| !AMBIGUOUS.contains(*c))
            .collect();
    }

    let chars: Vec<char> = charset.chars().collect();
    let mut rng = rand::thread_rng();

    loop {
        let password: String = (0..config.length)
            .map(|_| chars[rng.gen_range(0..chars.len())])
            .collect();

        if meets_requirements(&password, config) {
            return password;
        }
    }
}

fn meets_requirements(password: &str, config: &PasswordConfig) -> bool {
    if config.uppercase && !password.chars().any(|c| c.is_ascii_uppercase()) {
        return false;
    }
    if config.lowercase && !password.chars().any(|c| c.is_ascii_lowercase()) {
        return false;
    }
    if config.digits && !password.chars().any(|c| c.is_ascii_digit()) {
        return false;
    }
    if config.symbols && !password.chars().any(|c| SYMBOLS.contains(c)) {
        return false;
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config_generates_valid_password() {
        let config = PasswordConfig::default();
        let password = generate_password(&config);
        assert_eq!(password.len(), 20);
        assert!(password.chars().any(|c| c.is_ascii_uppercase()));
        assert!(password.chars().any(|c| c.is_ascii_lowercase()));
        assert!(password.chars().any(|c| c.is_ascii_digit()));
        assert!(password.chars().any(|c| SYMBOLS.contains(c)));
    }

    #[test]
    fn test_custom_length() {
        let config = PasswordConfig {
            length: 8,
            ..Default::default()
        };
        let password = generate_password(&config);
        assert_eq!(password.len(), 8);
    }

    #[test]
    fn test_only_lowercase() {
        let config = PasswordConfig {
            length: 30,
            uppercase: false,
            lowercase: true,
            digits: false,
            symbols: false,
            exclude_ambiguous: false,
        };
        let password = generate_password(&config);
        assert!(password.chars().all(|c| c.is_ascii_lowercase()));
    }

    #[test]
    fn test_only_digits() {
        let config = PasswordConfig {
            length: 30,
            uppercase: false,
            lowercase: false,
            digits: true,
            symbols: false,
            exclude_ambiguous: false,
        };
        let password = generate_password(&config);
        assert!(password.chars().all(|c| c.is_ascii_digit()));
    }

    #[test]
    fn test_exclude_ambiguous() {
        let config = PasswordConfig {
            length: 100,
            uppercase: true,
            lowercase: true,
            digits: true,
            symbols: false,
            exclude_ambiguous: true,
        };
        let password = generate_password(&config);
        assert!(!password.contains('0'));
        assert!(!password.contains('O'));
        assert!(!password.contains('1'));
        assert!(!password.contains('l'));
        assert!(!password.contains('I'));
    }

    #[test]
    fn test_uniqueness() {
        let config = PasswordConfig::default();
        let p1 = generate_password(&config);
        let p2 = generate_password(&config);
        assert_ne!(p1, p2);
    }
}
