// Romaji conversion functionality
// Converts katakana to Hepburn romanization

use std::collections::HashMap;

/// Convert katakana to Hepburn romanization
///
/// This function converts katakana text to Hepburn-style romaji.
/// When `use_macron` is true, long vowels are represented with macrons (ā, ī, ū, ē, ō).
/// When false, long vowels are represented with repeated vowels (aa, ii, uu, ee, oo/ou).
pub fn katakana_to_hepburn(kata: &str, use_macron: bool) -> String {
    // Base kana to romaji mappings
    let base = get_base_mappings();
    let digraphs = get_digraph_mappings();
    let small_kana: Vec<char> = vec![
        'ャ', 'ュ', 'ョ', 'ァ', 'ィ', 'ゥ', 'ェ', 'ォ', 'ヮ', 'ヵ', 'ヶ', 'ッ',
    ];

    let mut result = String::new();
    let chars: Vec<char> = kata.trim().chars().collect();
    let length = chars.len();
    let mut i = 0;

    while i < length {
        let ch = chars[i];

        // Handle sokuon (っ/ッ) - geminate consonant
        if ch == 'ッ' {
            if i + 1 < length {
                // Look ahead to get the next consonant
                let next_romaji = if i + 2 < length {
                    // Check for digraph first
                    digraphs.get(&(chars[i + 1], chars[i + 2]))
                } else {
                    None
                };

                let next_romaji = next_romaji.or_else(|| base.get(&chars[i + 1]));

                if let Some(rom) = next_romaji {
                    let consonant = get_initial_consonant(rom);
                    if !consonant.is_empty() {
                        result.push(consonant.chars().next().unwrap());
                    }
                }
            }
            i += 1;
            continue;
        }

        // Handle chōonpu (ー) - long vowel mark
        if ch == 'ー' {
            result.push('-');
            i += 1;
            continue;
        }

        // Check for digraphs (two-character combinations)
        if i + 1 < length {
            if let Some(rom) = digraphs.get(&(ch, chars[i + 1])) {
                result.push_str(rom);
                i += 2;
                continue;
            }
        }

        // Handle small kana appearing independently
        if small_kana.contains(&ch) && ch != 'ッ' {
            if let Some(rom) = base.get(&ch) {
                result.push_str(rom);
            }
            i += 1;
            continue;
        }

        // Regular katakana
        if let Some(rom) = base.get(&ch) {
            result.push_str(rom);
            i += 1;
            continue;
        }

        // Non-katakana characters (pass through)
        result.push(ch);
        i += 1;
    }

    let mut raw = result;

    // Handle 'n' before b/p/m -> 'm' (Hepburn romanization rule)
    // This needs to be done carefully to avoid replacing in the middle of other patterns
    let mut chars: Vec<char> = raw.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        if i + 1 < chars.len() && chars[i] == 'n' {
            let next = chars[i + 1];
            if next == 'b' || next == 'p' || next == 'm' {
                chars[i] = 'm';
            }
        }
        i += 1;
    }
    raw = chars.iter().collect();

    // Process long vowel marks (-)
    while let Some(idx) = raw.find('-') {
        if idx == 0 {
            // Remove leading dash
            raw = raw[1..].to_string();
            continue;
        }

        let prev_char = raw.chars().nth(idx - 1).unwrap();
        if "aiueo".contains(prev_char) {
            // Replace dash with the previous vowel
            let mut chars: Vec<char> = raw.chars().collect();
            chars[idx] = prev_char;
            raw = chars.iter().collect();
        } else {
            // Remove dash if previous char is not a vowel
            raw = raw[..idx].to_string() + &raw[idx + 1..];
        }
    }

    // Apply macron conversion if requested
    if use_macron {
        let macron_map = get_macron_mappings();
        for (pattern, replacement) in macron_map.iter() {
            raw = raw.replace(pattern, replacement);
        }
    }

    // Convert to lowercase (Hepburn standard)
    raw.to_lowercase()
}

/// Get the initial consonant(s) from a romaji string
fn get_initial_consonant(rom: &str) -> String {
    let vowels = ['a', 'e', 'i', 'o', 'u'];
    for (i, ch) in rom.chars().enumerate() {
        if vowels.contains(&ch) {
            return rom[..i].to_string();
        }
    }
    rom.to_string()
}

/// Get base katakana to romaji mappings
fn get_base_mappings() -> HashMap<char, &'static str> {
    let mut map = HashMap::new();
    
    // Basic vowels
    map.insert('ア', "a");
    map.insert('イ', "i");
    map.insert('ウ', "u");
    map.insert('エ', "e");
    map.insert('オ', "o");
    
    // K-row
    map.insert('カ', "ka");
    map.insert('キ', "ki");
    map.insert('ク', "ku");
    map.insert('ケ', "ke");
    map.insert('コ', "ko");
    
    // S-row
    map.insert('サ', "sa");
    map.insert('シ', "shi");
    map.insert('ス', "su");
    map.insert('セ', "se");
    map.insert('ソ', "so");
    
    // T-row
    map.insert('タ', "ta");
    map.insert('チ', "chi");
    map.insert('ツ', "tsu");
    map.insert('テ', "te");
    map.insert('ト', "to");
    
    // N-row
    map.insert('ナ', "na");
    map.insert('ニ', "ni");
    map.insert('ヌ', "nu");
    map.insert('ネ', "ne");
    map.insert('ノ', "no");
    
    // H-row
    map.insert('ハ', "ha");
    map.insert('ヒ', "hi");
    map.insert('フ', "fu");
    map.insert('ヘ', "he");
    map.insert('ホ', "ho");
    
    // M-row
    map.insert('マ', "ma");
    map.insert('ミ', "mi");
    map.insert('ム', "mu");
    map.insert('メ', "me");
    map.insert('モ', "mo");
    
    // Y-row
    map.insert('ヤ', "ya");
    map.insert('ユ', "yu");
    map.insert('ヨ', "yo");
    
    // R-row
    map.insert('ラ', "ra");
    map.insert('リ', "ri");
    map.insert('ル', "ru");
    map.insert('レ', "re");
    map.insert('ロ', "ro");
    
    // W-row
    map.insert('ワ', "wa");
    map.insert('ヲ', "wo");
    map.insert('ン', "n");
    
    // Voiced consonants (G-row)
    map.insert('ガ', "ga");
    map.insert('ギ', "gi");
    map.insert('グ', "gu");
    map.insert('ゲ', "ge");
    map.insert('ゴ', "go");
    
    // Z-row
    map.insert('ザ', "za");
    map.insert('ジ', "ji");
    map.insert('ズ', "zu");
    map.insert('ゼ', "ze");
    map.insert('ゾ', "zo");
    
    // D-row
    map.insert('ダ', "da");
    map.insert('ヂ', "ji");
    map.insert('ヅ', "zu");
    map.insert('デ', "de");
    map.insert('ド', "do");
    
    // B-row
    map.insert('バ', "ba");
    map.insert('ビ', "bi");
    map.insert('ブ', "bu");
    map.insert('ベ', "be");
    map.insert('ボ', "bo");
    
    // P-row
    map.insert('パ', "pa");
    map.insert('ピ', "pi");
    map.insert('プ', "pu");
    map.insert('ペ', "pe");
    map.insert('ポ', "po");
    
    // Small kana
    map.insert('ァ', "a");
    map.insert('ィ', "i");
    map.insert('ゥ', "u");
    map.insert('ェ', "e");
    map.insert('ォ', "o");
    map.insert('ャ', "ya");
    map.insert('ュ', "yu");
    map.insert('ョ', "yo");
    map.insert('ッ', "xtsu");
    map.insert('ー', "-");
    
    // Special characters
    map.insert('ヴ', "vu");
    
    map
}

/// Get digraph (two-character combination) mappings
fn get_digraph_mappings() -> HashMap<(char, char), &'static str> {
    let mut map = HashMap::new();
    
    // K-row combinations
    map.insert(('キ', 'ャ'), "kya");
    map.insert(('キ', 'ュ'), "kyu");
    map.insert(('キ', 'ョ'), "kyo");
    map.insert(('キ', 'ェ'), "kye");
    
    // G-row combinations
    map.insert(('ギ', 'ャ'), "gya");
    map.insert(('ギ', 'ュ'), "gyu");
    map.insert(('ギ', 'ョ'), "gyo");
    map.insert(('ギ', 'ェ'), "gye");
    
    // S-row combinations
    map.insert(('シ', 'ャ'), "sha");
    map.insert(('シ', 'ュ'), "shu");
    map.insert(('シ', 'ョ'), "sho");
    map.insert(('シ', 'ェ'), "she");
    
    // J-row combinations
    map.insert(('ジ', 'ャ'), "ja");
    map.insert(('ジ', 'ュ'), "ju");
    map.insert(('ジ', 'ョ'), "jo");
    
    // Ch-row combinations
    map.insert(('チ', 'ャ'), "cha");
    map.insert(('チ', 'ュ'), "chu");
    map.insert(('チ', 'ョ'), "cho");
    map.insert(('チ', 'ェ'), "che");
    
    // N-row combinations
    map.insert(('ニ', 'ャ'), "nya");
    map.insert(('ニ', 'ュ'), "nyu");
    map.insert(('ニ', 'ョ'), "nyo");
    
    // H-row combinations
    map.insert(('ヒ', 'ャ'), "hya");
    map.insert(('ヒ', 'ュ'), "hyu");
    map.insert(('ヒ', 'ョ'), "hyo");
    
    // B-row combinations
    map.insert(('ビ', 'ャ'), "bya");
    map.insert(('ビ', 'ュ'), "byu");
    map.insert(('ビ', 'ョ'), "byo");
    
    // P-row combinations
    map.insert(('ピ', 'ャ'), "pya");
    map.insert(('ピ', 'ュ'), "pyu");
    map.insert(('ピ', 'ョ'), "pyo");
    
    // M-row combinations
    map.insert(('ミ', 'ャ'), "mya");
    map.insert(('ミ', 'ュ'), "myu");
    map.insert(('ミ', 'ョ'), "myo");
    
    // R-row combinations
    map.insert(('リ', 'ャ'), "rya");
    map.insert(('リ', 'ュ'), "ryu");
    map.insert(('リ', 'ョ'), "ryo");
    
    // F-row combinations (foreign sounds)
    map.insert(('フ', 'ャ'), "fya");
    map.insert(('フ', 'ュ'), "fyu");
    map.insert(('フ', 'ョ'), "fyo");
    map.insert(('フ', 'ァ'), "fa");
    map.insert(('フ', 'ィ'), "fi");
    map.insert(('フ', 'ェ'), "fe");
    map.insert(('フ', 'ォ'), "fo");
    
    // T/D combinations
    map.insert(('ト', 'ゥ'), "tu");
    map.insert(('ド', 'ゥ'), "du");
    map.insert(('テ', 'ィ'), "ti");
    
    // W combinations
    map.insert(('ウ', 'ァ'), "wa");
    map.insert(('ウ', 'ィ'), "wi");
    map.insert(('ウ', 'ェ'), "we");
    map.insert(('ウ', 'ォ'), "wo");
    
    // Other foreign sound combinations
    map.insert(('ス', 'ィ'), "si");
    map.insert(('ズ', 'ィ'), "zi");
    map.insert(('ツ', 'ァ'), "tsa");
    map.insert(('ツ', 'ィ'), "tsi");
    map.insert(('ツ', 'ェ'), "tse");
    map.insert(('ツ', 'ォ'), "tso");
    
    // V combinations
    map.insert(('ヴ', 'ァ'), "va");
    map.insert(('ヴ', 'ィ'), "vi");
    map.insert(('ヴ', 'ェ'), "ve");
    map.insert(('ヴ', 'ォ'), "vo");
    map.insert(('ヴ', 'ュ'), "vyu");
    
    map
}

/// Get macron conversion mappings for long vowels
fn get_macron_mappings() -> Vec<(&'static str, &'static str)> {
    vec![
        ("ou", "ō"),
        ("oo", "ō"),
        ("aa", "ā"),
        ("ii", "ī"),
        ("uu", "ū"),
        ("ee", "ē"),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_katakana() {
        assert_eq!(katakana_to_hepburn("カタカナ", false), "katakana");
        assert_eq!(katakana_to_hepburn("ホテル", false), "hoteru");
    }

    #[test]
    fn test_long_vowels_without_macron() {
        // Note: "ン" before "p" becomes "m" in Hepburn romanization
        assert_eq!(katakana_to_hepburn("コンピューター", false), "kompyuutaa");
        assert_eq!(katakana_to_hepburn("スーパー", false), "suupaa");
    }

    #[test]
    fn test_long_vowels_with_macron() {
        // Note: "ン" before "p" becomes "m" in Hepburn romanization
        assert_eq!(katakana_to_hepburn("コンピューター", true), "kompyūtā");
        assert_eq!(katakana_to_hepburn("スーパー", true), "sūpā");
        assert_eq!(katakana_to_hepburn("トウキョウ", true), "tōkyō");
    }

    #[test]
    fn test_sokuon() {
        // Note: "ッ" doubles the following consonant
        assert_eq!(katakana_to_hepburn("マッチャ", false), "maccha");
        assert_eq!(katakana_to_hepburn("キャッチ", false), "kyacchi");
    }

    #[test]
    fn test_digraphs() {
        assert_eq!(katakana_to_hepburn("キャ", false), "kya");
        assert_eq!(katakana_to_hepburn("シャ", false), "sha");
        assert_eq!(katakana_to_hepburn("チュ", false), "chu");
    }

    #[test]
    fn test_n_before_bpm() {
        assert_eq!(katakana_to_hepburn("サンバ", false), "samba");
        assert_eq!(katakana_to_hepburn("サンポ", false), "sampo");
    }

    #[test]
    fn test_foreign_sounds() {
        assert_eq!(katakana_to_hepburn("ファイル", false), "fairu");
        assert_eq!(katakana_to_hepburn("ヴァイオリン", false), "vaiorin");
    }

    #[test]
    fn test_mixed_content() {
        assert_eq!(katakana_to_hepburn("カタカナabc", false), "katakanaabc");
    }

    #[test]
    fn test_empty_string() {
        assert_eq!(katakana_to_hepburn("", false), "");
    }
}
