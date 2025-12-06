// Kana conversion functionality
// Handles conversion between hiragana, katakana, and detection

/// Check if a character is a kana character (hiragana or katakana)
pub fn is_kana(ch: char) -> bool {
    is_hiragana(ch) || is_katakana(ch)
}

/// Check if a character is hiragana
pub fn is_hiragana(ch: char) -> bool {
    '\u{3040}' <= ch && ch <= '\u{309F}'
}

/// Check if a character is katakana
pub fn is_katakana(ch: char) -> bool {
    '\u{30A0}' <= ch && ch <= '\u{30FF}'
}

/// Check if a character is kanji
pub fn is_kanji(ch: char) -> bool {
    '\u{4E00}' <= ch && ch <= '\u{9FFF}'
}

/// Convert katakana to hiragana
/// 
/// This function converts katakana characters to their hiragana equivalents.
/// Non-katakana characters are left unchanged.
pub fn katakana_to_hiragana(text: &str) -> String {
    text.chars()
        .map(|ch| {
            if is_katakana(ch) && ch != 'ー' {
                // Katakana to hiragana: subtract 0x60
                // ァ (U+30A1) -> ぁ (U+3041)
                // ン (U+30F3) -> ん (U+3093)
                if ch >= 'ァ' && ch <= 'ン' {
                    char::from_u32(ch as u32 - 0x60).unwrap_or(ch)
                } else {
                    ch
                }
            } else {
                ch
            }
        })
        .collect()
}

/// Convert hiragana to katakana
/// 
/// This function converts hiragana characters to their katakana equivalents.
/// Non-hiragana characters are left unchanged.
pub fn hiragana_to_katakana(text: &str) -> String {
    text.chars()
        .map(|ch| {
            if is_hiragana(ch) {
                // Hiragana to katakana: add 0x60
                // ぁ (U+3041) -> ァ (U+30A1)
                // ん (U+3093) -> ン (U+30F3)
                if ch >= 'ぁ' && ch <= 'ん' {
                    char::from_u32(ch as u32 + 0x60).unwrap_or(ch)
                } else {
                    ch
                }
            } else {
                ch
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_hiragana() {
        assert!(is_hiragana('あ'));
        assert!(is_hiragana('ん'));
        assert!(!is_hiragana('ア'));
        assert!(!is_hiragana('a'));
    }

    #[test]
    fn test_is_katakana() {
        assert!(is_katakana('ア'));
        assert!(is_katakana('ン'));
        assert!(!is_katakana('あ'));
        assert!(!is_katakana('a'));
    }

    #[test]
    fn test_is_kana() {
        assert!(is_kana('あ'));
        assert!(is_kana('ア'));
        assert!(!is_kana('a'));
        assert!(!is_kana('漢'));
    }

    #[test]
    fn test_is_kanji() {
        assert!(is_kanji('漢'));
        assert!(is_kanji('字'));
        assert!(!is_kanji('あ'));
        assert!(!is_kanji('a'));
    }

    #[test]
    fn test_katakana_to_hiragana() {
        assert_eq!(katakana_to_hiragana("カタカナ"), "かたかな");
        assert_eq!(katakana_to_hiragana("コンピューター"), "こんぴゅーたー");
        assert_eq!(katakana_to_hiragana("abc"), "abc");
        assert_eq!(katakana_to_hiragana("カタカナabc"), "かたかなabc");
    }

    #[test]
    fn test_hiragana_to_katakana() {
        assert_eq!(hiragana_to_katakana("ひらがな"), "ヒラガナ");
        assert_eq!(hiragana_to_katakana("こんにちは"), "コンニチハ");
        assert_eq!(hiragana_to_katakana("abc"), "abc");
        assert_eq!(hiragana_to_katakana("ひらがなabc"), "ヒラガナabc");
    }

    #[test]
    fn test_round_trip() {
        let original = "カタカナ";
        let hiragana = katakana_to_hiragana(original);
        let back_to_katakana = hiragana_to_katakana(&hiragana);
        assert_eq!(original, back_to_katakana);
    }
}
