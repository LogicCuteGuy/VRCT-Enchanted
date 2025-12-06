// Language definitions for transcription
// Maps display language and country to engine-specific language codes

use std::collections::HashMap;
use serde::{Deserialize, Serialize};

/// Language mapping for a specific country
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LanguageMapping {
    pub google: String,
    pub whisper: String,
}

/// Get the complete transcription language table
/// Returns a nested HashMap: Language -> Country -> LanguageMapping
pub fn get_transcription_languages() -> HashMap<String, HashMap<String, LanguageMapping>> {
    let mut languages = HashMap::new();
    
    // Afrikaans
    let mut afrikaans = HashMap::new();
    afrikaans.insert("South Africa".to_string(), LanguageMapping {
        google: "af-ZA".to_string(),
        whisper: "af".to_string(),
    });
    languages.insert("Afrikaans".to_string(), afrikaans);
    
    // Albanian
    let mut albanian = HashMap::new();
    albanian.insert("Albania".to_string(), LanguageMapping {
        google: "sq-AL".to_string(),
        whisper: "sq".to_string(),
    });
    languages.insert("Albanian".to_string(), albanian);
    
    // Amharic
    let mut amharic = HashMap::new();
    amharic.insert("Ethiopia".to_string(), LanguageMapping {
        google: "am-ET".to_string(),
        whisper: "am".to_string(),
    });
    languages.insert("Amharic".to_string(), amharic);
    
    // Arabic - multiple countries
    let mut arabic = HashMap::new();
    let arabic_countries = vec![
        ("Algeria", "ar-DZ"),
        ("Bahrain", "ar-BH"),
        ("Egypt", "ar-EG"),
        ("Israel", "ar-IL"),
        ("Iraq", "ar-IQ"),
        ("Jordan", "ar-JO"),
        ("Kuwait", "ar-KW"),
        ("Lebanon", "ar-LB"),
        ("Mauritania", "ar-MR"),
        ("Morocco", "ar-MA"),
        ("Oman", "ar-OM"),
        ("Qatar", "ar-QA"),
        ("Saudi Arabia", "ar-SA"),
        ("Palestine", "ar-PS"),
        ("Syria", "ar-SY"),
        ("Tunisia", "ar-TN"),
        ("United Arab Emirates", "ar-AE"),
        ("Yemen", "ar-YE"),
    ];
    for (country, google_code) in arabic_countries {
        arabic.insert(country.to_string(), LanguageMapping {
            google: google_code.to_string(),
            whisper: "ar".to_string(),
        });
    }
    languages.insert("Arabic".to_string(), arabic);
    
    // Armenian
    let mut armenian = HashMap::new();
    armenian.insert("Armenia".to_string(), LanguageMapping {
        google: "hy-AM".to_string(),
        whisper: "hy".to_string(),
    });
    languages.insert("Armenian".to_string(), armenian);
    
    // Azerbaijani
    let mut azerbaijani = HashMap::new();
    azerbaijani.insert("Azerbaijan".to_string(), LanguageMapping {
        google: "az-AZ".to_string(),
        whisper: "az".to_string(),
    });
    languages.insert("Azerbaijani".to_string(), azerbaijani);
    
    // Basque
    let mut basque = HashMap::new();
    basque.insert("Spain".to_string(), LanguageMapping {
        google: "eu-ES".to_string(),
        whisper: "eu".to_string(),
    });
    languages.insert("Basque".to_string(), basque);
    
    // Bengali
    let mut bengali = HashMap::new();
    bengali.insert("Bangladesh".to_string(), LanguageMapping {
        google: "bn-BD".to_string(),
        whisper: "bn".to_string(),
    });
    bengali.insert("India".to_string(), LanguageMapping {
        google: "bn-IN".to_string(),
        whisper: "bn".to_string(),
    });
    languages.insert("Bengali".to_string(), bengali);
    
    // Bosnian
    let mut bosnian = HashMap::new();
    bosnian.insert("Bosnia and Herzegovina".to_string(), LanguageMapping {
        google: "bs-BA".to_string(),
        whisper: "bs".to_string(),
    });
    languages.insert("Bosnian".to_string(), bosnian);
    
    // Bulgarian
    let mut bulgarian = HashMap::new();
    bulgarian.insert("Bulgaria".to_string(), LanguageMapping {
        google: "bg-BG".to_string(),
        whisper: "bg".to_string(),
    });
    languages.insert("Bulgarian".to_string(), bulgarian);
    
    // Burmese
    let mut burmese = HashMap::new();
    burmese.insert("Myanmar".to_string(), LanguageMapping {
        google: "my-MM".to_string(),
        whisper: "my".to_string(),
    });
    languages.insert("Burmese".to_string(), burmese);
    
    // Catalan
    let mut catalan = HashMap::new();
    catalan.insert("Spain".to_string(), LanguageMapping {
        google: "ca-ES".to_string(),
        whisper: "ca".to_string(),
    });
    languages.insert("Catalan".to_string(), catalan);
    
    // Chinese Simplified
    let mut chinese_simplified = HashMap::new();
    chinese_simplified.insert("China".to_string(), LanguageMapping {
        google: "cmn-Hans-CN".to_string(),
        whisper: "zh".to_string(),
    });
    chinese_simplified.insert("Hong Kong".to_string(), LanguageMapping {
        google: "cmn-Hans-HK".to_string(),
        whisper: "zh".to_string(),
    });
    languages.insert("Chinese Simplified".to_string(), chinese_simplified);
    
    // Chinese Traditional
    let mut chinese_traditional = HashMap::new();
    chinese_traditional.insert("Taiwan".to_string(), LanguageMapping {
        google: "cmn-Hant-TW".to_string(),
        whisper: "zh".to_string(),
    });
    chinese_traditional.insert("Hong Kong".to_string(), LanguageMapping {
        google: "yue-Hant-HK".to_string(),
        whisper: "yue".to_string(),
    });
    languages.insert("Chinese Traditional".to_string(), chinese_traditional);
    
    // Croatian
    let mut croatian = HashMap::new();
    croatian.insert("Croatia".to_string(), LanguageMapping {
        google: "hr-HR".to_string(),
        whisper: "hr".to_string(),
    });
    languages.insert("Croatian".to_string(), croatian);
    
    // Czech
    let mut czech = HashMap::new();
    czech.insert("Czech Republic".to_string(), LanguageMapping {
        google: "cs-CZ".to_string(),
        whisper: "cs".to_string(),
    });
    languages.insert("Czech".to_string(), czech);
    
    // Danish
    let mut danish = HashMap::new();
    danish.insert("Denmark".to_string(), LanguageMapping {
        google: "da-DK".to_string(),
        whisper: "da".to_string(),
    });
    languages.insert("Danish".to_string(), danish);
    
    // Dutch
    let mut dutch = HashMap::new();
    dutch.insert("Belgium".to_string(), LanguageMapping {
        google: "nl-BE".to_string(),
        whisper: "nl".to_string(),
    });
    dutch.insert("Netherlands".to_string(), LanguageMapping {
        google: "nl-NL".to_string(),
        whisper: "nl".to_string(),
    });
    languages.insert("Dutch".to_string(), dutch);
    
    // English - multiple countries
    let mut english = HashMap::new();
    let english_countries = vec![
        ("Australia", "en-AU"),
        ("Canada", "en-CA"),
        ("Ghana", "en-GH"),
        ("Hong Kong", "en-HK"),
        ("India", "en-IN"),
        ("Ireland", "en-IE"),
        ("Kenya", "en-KE"),
        ("New Zealand", "en-NZ"),
        ("Nigeria", "en-NG"),
        ("Philippines", "en-PH"),
        ("Singapore", "en-SG"),
        ("South Africa", "en-ZA"),
        ("Tanzania", "en-TZ"),
        ("United Kingdom", "en-GB"),
        ("United States", "en-US"),
    ];
    for (country, google_code) in english_countries {
        english.insert(country.to_string(), LanguageMapping {
            google: google_code.to_string(),
            whisper: "en".to_string(),
        });
    }
    languages.insert("English".to_string(), english);
    
    // Estonian
    let mut estonian = HashMap::new();
    estonian.insert("Estonia".to_string(), LanguageMapping {
        google: "et-EE".to_string(),
        whisper: "et".to_string(),
    });
    languages.insert("Estonian".to_string(), estonian);
    
    // Filipino
    let mut filipino = HashMap::new();
    filipino.insert("Philippines".to_string(), LanguageMapping {
        google: "fil-PH".to_string(),
        whisper: "tl".to_string(),
    });
    languages.insert("Filipino".to_string(), filipino);
    
    // Finnish
    let mut finnish = HashMap::new();
    finnish.insert("Finland".to_string(), LanguageMapping {
        google: "fi-FI".to_string(),
        whisper: "fi".to_string(),
    });
    languages.insert("Finnish".to_string(), finnish);
    
    // French
    let mut french = HashMap::new();
    french.insert("Belgium".to_string(), LanguageMapping {
        google: "fr-BE".to_string(),
        whisper: "fr".to_string(),
    });
    french.insert("Canada".to_string(), LanguageMapping {
        google: "fr-CA".to_string(),
        whisper: "fr".to_string(),
    });
    french.insert("France".to_string(), LanguageMapping {
        google: "fr-FR".to_string(),
        whisper: "fr".to_string(),
    });
    french.insert("Switzerland".to_string(), LanguageMapping {
        google: "fr-CH".to_string(),
        whisper: "fr".to_string(),
    });
    languages.insert("French".to_string(), french);
    
    // Galician
    let mut galician = HashMap::new();
    galician.insert("Spain".to_string(), LanguageMapping {
        google: "gl-ES".to_string(),
        whisper: "gl".to_string(),
    });
    languages.insert("Galician".to_string(), galician);
    
    // Georgian
    let mut georgian = HashMap::new();
    georgian.insert("Georgia".to_string(), LanguageMapping {
        google: "ka-GE".to_string(),
        whisper: "ka".to_string(),
    });
    languages.insert("Georgian".to_string(), georgian);
    
    // German
    let mut german = HashMap::new();
    german.insert("Austria".to_string(), LanguageMapping {
        google: "de-AT".to_string(),
        whisper: "de".to_string(),
    });
    german.insert("Germany".to_string(), LanguageMapping {
        google: "de-DE".to_string(),
        whisper: "de".to_string(),
    });
    german.insert("Switzerland".to_string(), LanguageMapping {
        google: "de-CH".to_string(),
        whisper: "de".to_string(),
    });
    languages.insert("German".to_string(), german);
    
    // Greek
    let mut greek = HashMap::new();
    greek.insert("Greece".to_string(), LanguageMapping {
        google: "el-GR".to_string(),
        whisper: "el".to_string(),
    });
    languages.insert("Greek".to_string(), greek);
    
    // Gujarati
    let mut gujarati = HashMap::new();
    gujarati.insert("India".to_string(), LanguageMapping {
        google: "gu-IN".to_string(),
        whisper: "gu".to_string(),
    });
    languages.insert("Gujarati".to_string(), gujarati);
    
    // Hebrew
    let mut hebrew = HashMap::new();
    hebrew.insert("Israel".to_string(), LanguageMapping {
        google: "iw-IL".to_string(),
        whisper: "he".to_string(),
    });
    languages.insert("Hebrew".to_string(), hebrew);
    
    // Hindi
    let mut hindi = HashMap::new();
    hindi.insert("India".to_string(), LanguageMapping {
        google: "hi-IN".to_string(),
        whisper: "hi".to_string(),
    });
    languages.insert("Hindi".to_string(), hindi);
    
    // Hungarian
    let mut hungarian = HashMap::new();
    hungarian.insert("Hungary".to_string(), LanguageMapping {
        google: "hu-HU".to_string(),
        whisper: "hu".to_string(),
    });
    languages.insert("Hungarian".to_string(), hungarian);
    
    // Icelandic
    let mut icelandic = HashMap::new();
    icelandic.insert("Iceland".to_string(), LanguageMapping {
        google: "is-IS".to_string(),
        whisper: "is".to_string(),
    });
    languages.insert("Icelandic".to_string(), icelandic);
    
    // Indonesian
    let mut indonesian = HashMap::new();
    indonesian.insert("Indonesia".to_string(), LanguageMapping {
        google: "id-ID".to_string(),
        whisper: "id".to_string(),
    });
    languages.insert("Indonesian".to_string(), indonesian);
    
    // Italian
    let mut italian = HashMap::new();
    italian.insert("Italy".to_string(), LanguageMapping {
        google: "it-IT".to_string(),
        whisper: "it".to_string(),
    });
    italian.insert("Switzerland".to_string(), LanguageMapping {
        google: "it-CH".to_string(),
        whisper: "it".to_string(),
    });
    languages.insert("Italian".to_string(), italian);
    
    // Japanese
    let mut japanese = HashMap::new();
    japanese.insert("Japan".to_string(), LanguageMapping {
        google: "ja-JP".to_string(),
        whisper: "ja".to_string(),
    });
    languages.insert("Japanese".to_string(), japanese);
    
    // Kannada
    let mut kannada = HashMap::new();
    kannada.insert("India".to_string(), LanguageMapping {
        google: "kn-IN".to_string(),
        whisper: "kn".to_string(),
    });
    languages.insert("Kannada".to_string(), kannada);
    
    // Kazakh
    let mut kazakh = HashMap::new();
    kazakh.insert("Kazakhstan".to_string(), LanguageMapping {
        google: "kk-KZ".to_string(),
        whisper: "kk".to_string(),
    });
    languages.insert("Kazakh".to_string(), kazakh);
    
    // Khmer
    let mut khmer = HashMap::new();
    khmer.insert("Cambodia".to_string(), LanguageMapping {
        google: "km-KH".to_string(),
        whisper: "km".to_string(),
    });
    languages.insert("Khmer".to_string(), khmer);
    
    // Korean
    let mut korean = HashMap::new();
    korean.insert("South Korea".to_string(), LanguageMapping {
        google: "ko-KR".to_string(),
        whisper: "ko".to_string(),
    });
    languages.insert("Korean".to_string(), korean);
    
    // Lao
    let mut lao = HashMap::new();
    lao.insert("Laos".to_string(), LanguageMapping {
        google: "lo-LA".to_string(),
        whisper: "lo".to_string(),
    });
    languages.insert("Lao".to_string(), lao);
    
    // Latvian
    let mut latvian = HashMap::new();
    latvian.insert("Latvia".to_string(), LanguageMapping {
        google: "lv-LV".to_string(),
        whisper: "lv".to_string(),
    });
    languages.insert("Latvian".to_string(), latvian);
    
    // Lithuanian
    let mut lithuanian = HashMap::new();
    lithuanian.insert("Lithuania".to_string(), LanguageMapping {
        google: "lt-LT".to_string(),
        whisper: "lt".to_string(),
    });
    languages.insert("Lithuanian".to_string(), lithuanian);
    
    // Macedonian
    let mut macedonian = HashMap::new();
    macedonian.insert("North Macedonia".to_string(), LanguageMapping {
        google: "mk-MK".to_string(),
        whisper: "mk".to_string(),
    });
    languages.insert("Macedonian".to_string(), macedonian);
    
    // Malay
    let mut malay = HashMap::new();
    malay.insert("Malaysia".to_string(), LanguageMapping {
        google: "ms-MY".to_string(),
        whisper: "ms".to_string(),
    });
    languages.insert("Malay".to_string(), malay);
    
    // Malayalam
    let mut malayalam = HashMap::new();
    malayalam.insert("India".to_string(), LanguageMapping {
        google: "ml-IN".to_string(),
        whisper: "ml".to_string(),
    });
    languages.insert("Malayalam".to_string(), malayalam);
    
    // Mongolian
    let mut mongolian = HashMap::new();
    mongolian.insert("Mongolia".to_string(), LanguageMapping {
        google: "mn-MN".to_string(),
        whisper: "mn".to_string(),
    });
    languages.insert("Mongolian".to_string(), mongolian);
    
    // Nepali
    let mut nepali = HashMap::new();
    nepali.insert("Nepal".to_string(), LanguageMapping {
        google: "ne-NP".to_string(),
        whisper: "ne".to_string(),
    });
    languages.insert("Nepali".to_string(), nepali);
    
    // Norwegian
    let mut norwegian = HashMap::new();
    norwegian.insert("Norway".to_string(), LanguageMapping {
        google: "no-NO".to_string(),
        whisper: "no".to_string(),
    });
    languages.insert("Norwegian".to_string(), norwegian);
    
    // Persian
    let mut persian = HashMap::new();
    persian.insert("Iran".to_string(), LanguageMapping {
        google: "fa-IR".to_string(),
        whisper: "fa".to_string(),
    });
    languages.insert("Persian".to_string(), persian);
    
    // Polish
    let mut polish = HashMap::new();
    polish.insert("Poland".to_string(), LanguageMapping {
        google: "pl-PL".to_string(),
        whisper: "pl".to_string(),
    });
    languages.insert("Polish".to_string(), polish);
    
    // Portuguese
    let mut portuguese = HashMap::new();
    portuguese.insert("Brazil".to_string(), LanguageMapping {
        google: "pt-BR".to_string(),
        whisper: "pt".to_string(),
    });
    portuguese.insert("Portugal".to_string(), LanguageMapping {
        google: "pt-PT".to_string(),
        whisper: "pt".to_string(),
    });
    languages.insert("Portuguese".to_string(), portuguese);
    
    // Romanian
    let mut romanian = HashMap::new();
    romanian.insert("Romania".to_string(), LanguageMapping {
        google: "ro-RO".to_string(),
        whisper: "ro".to_string(),
    });
    languages.insert("Romanian".to_string(), romanian);
    
    // Russian
    let mut russian = HashMap::new();
    russian.insert("Russia".to_string(), LanguageMapping {
        google: "ru-RU".to_string(),
        whisper: "ru".to_string(),
    });
    languages.insert("Russian".to_string(), russian);
    
    // Serbian
    let mut serbian = HashMap::new();
    serbian.insert("Serbia".to_string(), LanguageMapping {
        google: "sr-RS".to_string(),
        whisper: "sr".to_string(),
    });
    languages.insert("Serbian".to_string(), serbian);
    
    // Sinhala
    let mut sinhala = HashMap::new();
    sinhala.insert("Sri Lanka".to_string(), LanguageMapping {
        google: "si-LK".to_string(),
        whisper: "si".to_string(),
    });
    languages.insert("Sinhala".to_string(), sinhala);
    
    // Slovak
    let mut slovak = HashMap::new();
    slovak.insert("Slovakia".to_string(), LanguageMapping {
        google: "sk-SK".to_string(),
        whisper: "sk".to_string(),
    });
    languages.insert("Slovak".to_string(), slovak);
    
    // Slovenian
    let mut slovenian = HashMap::new();
    slovenian.insert("Slovenia".to_string(), LanguageMapping {
        google: "sl-SI".to_string(),
        whisper: "sl".to_string(),
    });
    languages.insert("Slovenian".to_string(), slovenian);
    
    // Spanish - multiple countries
    let mut spanish = HashMap::new();
    let spanish_countries = vec![
        ("Argentina", "es-AR"),
        ("Bolivia", "es-BO"),
        ("Chile", "es-CL"),
        ("Colombia", "es-CO"),
        ("Costa Rica", "es-CR"),
        ("Dominican Republic", "es-DO"),
        ("Ecuador", "es-EC"),
        ("El Salvador", "es-SV"),
        ("Guatemala", "es-GT"),
        ("Honduras", "es-HN"),
        ("Mexico", "es-MX"),
        ("Nicaragua", "es-NI"),
        ("Panama", "es-PA"),
        ("Paraguay", "es-PY"),
        ("Peru", "es-PE"),
        ("Puerto Rico", "es-PR"),
        ("Spain", "es-ES"),
        ("United States", "es-US"),
        ("Uruguay", "es-UY"),
        ("Venezuela", "es-VE"),
    ];
    for (country, google_code) in spanish_countries {
        spanish.insert(country.to_string(), LanguageMapping {
            google: google_code.to_string(),
            whisper: "es".to_string(),
        });
    }
    languages.insert("Spanish".to_string(), spanish);
    
    // Sundanese
    let mut sundanese = HashMap::new();
    sundanese.insert("Indonesia".to_string(), LanguageMapping {
        google: "su-ID".to_string(),
        whisper: "su".to_string(),
    });
    languages.insert("Sundanese".to_string(), sundanese);
    
    // Swahili
    let mut swahili = HashMap::new();
    swahili.insert("Kenya".to_string(), LanguageMapping {
        google: "sw-KE".to_string(),
        whisper: "sw".to_string(),
    });
    swahili.insert("Tanzania".to_string(), LanguageMapping {
        google: "sw-TZ".to_string(),
        whisper: "sw".to_string(),
    });
    languages.insert("Swahili".to_string(), swahili);
    
    // Swedish
    let mut swedish = HashMap::new();
    swedish.insert("Sweden".to_string(), LanguageMapping {
        google: "sv-SE".to_string(),
        whisper: "sv".to_string(),
    });
    languages.insert("Swedish".to_string(), swedish);
    
    // Tamil
    let mut tamil = HashMap::new();
    tamil.insert("India".to_string(), LanguageMapping {
        google: "ta-IN".to_string(),
        whisper: "ta".to_string(),
    });
    tamil.insert("malaysia".to_string(), LanguageMapping {
        google: "ta-MY".to_string(),
        whisper: "ta".to_string(),
    });
    tamil.insert("Singapore".to_string(), LanguageMapping {
        google: "ta-SG".to_string(),
        whisper: "ta".to_string(),
    });
    tamil.insert("Sri Lanka".to_string(), LanguageMapping {
        google: "ta-LK".to_string(),
        whisper: "ta".to_string(),
    });
    languages.insert("Tamil".to_string(), tamil);
    
    // Telugu
    let mut telugu = HashMap::new();
    telugu.insert("India".to_string(), LanguageMapping {
        google: "te-IN".to_string(),
        whisper: "te".to_string(),
    });
    languages.insert("Telugu".to_string(), telugu);
    
    // Thai
    let mut thai = HashMap::new();
    thai.insert("Thailand".to_string(), LanguageMapping {
        google: "th-TH".to_string(),
        whisper: "th".to_string(),
    });
    languages.insert("Thai".to_string(), thai);
    
    // Turkish
    let mut turkish = HashMap::new();
    turkish.insert("Turkey".to_string(), LanguageMapping {
        google: "tr-TR".to_string(),
        whisper: "tr".to_string(),
    });
    languages.insert("Turkish".to_string(), turkish);
    
    // Ukrainian
    let mut ukrainian = HashMap::new();
    ukrainian.insert("Ukraine".to_string(), LanguageMapping {
        google: "uk-UA".to_string(),
        whisper: "uk".to_string(),
    });
    languages.insert("Ukrainian".to_string(), ukrainian);
    
    // Urdu
    let mut urdu = HashMap::new();
    urdu.insert("India".to_string(), LanguageMapping {
        google: "ur-IN".to_string(),
        whisper: "ur".to_string(),
    });
    urdu.insert("Pakistan".to_string(), LanguageMapping {
        google: "ur-PK".to_string(),
        whisper: "ur".to_string(),
    });
    languages.insert("Urdu".to_string(), urdu);
    
    // Uzbek
    let mut uzbek = HashMap::new();
    uzbek.insert("Uzbekistan".to_string(), LanguageMapping {
        google: "uz-UZ".to_string(),
        whisper: "uz".to_string(),
    });
    languages.insert("Uzbek".to_string(), uzbek);
    
    // Vietnamese
    let mut vietnamese = HashMap::new();
    vietnamese.insert("Vietnam".to_string(), LanguageMapping {
        google: "vi-VN".to_string(),
        whisper: "vi".to_string(),
    });
    languages.insert("Vietnamese".to_string(), vietnamese);
    
    languages
}

/// Get Whisper language code for a given language and country
pub fn get_whisper_code(language: &str, country: &str) -> Option<String> {
    let languages = get_transcription_languages();
    languages
        .get(language)
        .and_then(|countries| countries.get(country))
        .map(|mapping| mapping.whisper.clone())
}

/// Get Google language code for a given language and country
pub fn get_google_code(language: &str, country: &str) -> Option<String> {
    let languages = get_transcription_languages();
    languages
        .get(language)
        .and_then(|countries| countries.get(country))
        .map(|mapping| mapping.google.clone())
}
