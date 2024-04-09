use hashbrown::HashMap;
use once_cell::sync::Lazy;

pub static CONVERT_MAP: Lazy<HashMap<char, char>> = Lazy::new(|| {
    HashMap::from([
        ('＂', '"'),
        ('＃', '#'),
        ('＄', '$'),
        ('％', '%'),
        ('＆', '&'),
        ('＇', '\''),
        ('＊', '*'),
        ('＋', '+'),
        ('．', '.'),
        ('／', '/'),
        ('０', '0'),
        ('１', '1'),
        ('２', '2'),
        ('３', '3'),
        ('４', '4'),
        ('５', '5'),
        ('６', '6'),
        ('７', '7'),
        ('８', '8'),
        ('９', '9'),
        ('＜', '<'),
        ('＝', '='),
        ('＞', '>'),
        ('＠', '@'),
        ('Ａ', 'A'),
        ('Ｂ', 'B'),
        ('Ｃ', 'C'),
        ('Ｄ', 'D'),
        ('Ｅ', 'E'),
        ('Ｆ', 'F'),
        ('Ｇ', 'G'),
        ('Ｈ', 'H'),
        ('Ｉ', 'I'),
        ('Ｊ', 'J'),
        ('Ｋ', 'K'),
        ('Ｌ', 'L'),
        ('Ｍ', 'M'),
        ('Ｎ', 'N'),
        ('Ｏ', 'O'),
        ('Ｐ', 'P'),
        ('Ｑ', 'Q'),
        ('Ｒ', 'R'),
        ('Ｓ', 'S'),
        ('Ｔ', 'T'),
        ('Ｕ', 'U'),
        ('Ｖ', 'V'),
        ('Ｗ', 'W'),
        ('Ｘ', 'X'),
        ('Ｙ', 'Y'),
        ('Ｚ', 'Z'),
        ('＼', '\\'),
        ('＾', '^'),
        ('｀', '`'),
        ('ａ', 'a'),
        ('ｂ', 'b'),
        ('ｃ', 'c'),
        ('ｄ', 'd'),
        ('ｅ', 'e'),
        ('ｆ', 'f'),
        ('ｇ', 'g'),
        ('ｈ', 'h'),
        ('ｉ', 'i'),
        ('ｊ', 'j'),
        ('ｋ', 'k'),
        ('ｌ', 'l'),
        ('ｍ', 'm'),
        ('ｎ', 'n'),
        ('ｏ', 'o'),
        ('ｐ', 'p'),
        ('ｑ', 'q'),
        ('ｒ', 'r'),
        ('ｓ', 's'),
        ('ｔ', 't'),
        ('ｕ', 'u'),
        ('ｖ', 'v'),
        ('ｗ', 'w'),
        ('ｘ', 'x'),
        ('ｙ', 'y'),
        ('ｚ', 'z'),
        ('｛', '{'),
        ('｜', '|'),
        ('｝', '}'),
        ('｡', '。'),
        ('｢', '「'),
        ('｣', '」'),
        ('､', '、'),
        ('･', '·'),
        ('•', '·'),
        ('─', '—'),
        ('―', '—'),
        ('∶', '：'),
        ('‧', '·'),
        ('・', '·'),
        ('﹑', '、'),
        ('〜', '～'),
        ('︰', '：'),
        ('?', '？'),
        ('!', '！'),
        (',', '，'),
        (';', '；'),
        ('(', '（'),
        (')', '）'),
    ])
});

// https://zh.wiktionary.org/wiki/
pub static CONVERT_T2S_MAP: Lazy<HashMap<char, char>> = Lazy::new(|| {
    HashMap::from([
        ('妳', '你'),
        ('姊', '姐'),
        ('擡', '抬'),
        ('牠', '它'),
        ('緖', '绪'),
        ('揹', '背'),
    ])
});

// https://zh.wikipedia.org/wiki/%E4%B8%AD%E6%97%A5%E9%9F%93%E7%B5%B1%E4%B8%80%E8%A1%A8%E6%84%8F%E6%96%87%E5%AD%97
#[must_use]
#[inline]
pub const fn is_cjk(c: char) -> bool {
    c == '\u{3007}'
        || range(c, '\u{3400}', '\u{4DBF}')
        || range(c, '\u{4E00}', '\u{9FFF}')
        || range(c, '\u{FA0E}', '\u{FA0F}')
        || c == '\u{FA11}'
        || range(c, '\u{FA13}', '\u{FA14}')
        || c == '\u{FA1F}'
        || c == '\u{FA21}'
        || range(c, '\u{FA23}', '\u{FA24}')
        || range(c, '\u{FA27}', '\u{FA29}')
        || range(c, '\u{20000}', '\u{2A6DF}')
        || range(c, '\u{2A700}', '\u{2B739}')
        || range(c, '\u{2B740}', '\u{2B81D}')
        || range(c, '\u{2B820}', '\u{2CEA1}')
        || range(c, '\u{2CEB0}', '\u{2EBE0}')
        || range(c, '\u{2EBF0}', '\u{2EE5F}')
        || range(c, '\u{30000}', '\u{3134A}')
        || range(c, '\u{31350}', '\u{323AF}')
}

#[must_use]
#[inline]
const fn range(c: char, min: char, max: char) -> bool {
    c >= min && c <= max
}

// https://zh.wikipedia.org/wiki/%E6%A0%87%E7%82%B9%E7%AC%A6%E5%8F%B7
#[must_use]
#[inline]
pub const fn is_chinese_punctuation(c: char) -> bool {
    c =='。' || c =='？' || c =='！' ||
     c =='，' || c =='、' || c =='；' ||
     c =='：' || c =='“' || c =='”' ||
     c =='『' || c =='』' || c =='‘' ||
     c =='’' || c =='「' || c =='」' ||
     c =='（' || c =='）' || c =='［' ||
     c =='］' || c =='〔' || c =='〕' ||
     c =='【' || c =='】' ||
     // ——
     c =='—' ||
     // ……
     c =='…' || c =='－' || c =='-' ||
     c =='～' || c =='·' || c =='《' ||
     c =='》' || c =='〈' || c =='〉' ||
     // ﹏﹏
     c =='﹏' ||
     // ＿＿
     c =='＿' || c =='.'
}

// https://zh.wikipedia.org/wiki/%E6%A0%87%E7%82%B9%E7%AC%A6%E5%8F%B7
#[must_use]
#[inline]
pub const fn is_english_punctuation(c: char) -> bool {
    c == '.'
        || c == '?'
        || c == '!'
        || c == ','
        || c == ':'
        || c == '…'
        || c == ';'
        || c == '-'
        || c == '–'
        || c == '—'
        || c == '('
        || c == ')'
        || c == '['
        || c == ']'
        || c == '{'
        || c == '}'
        || c == '"'
        || c == '\''
        || c == '/'
}

#[must_use]
#[inline]
pub const fn is_punctuation(c: char) -> bool {
    is_chinese_punctuation(c) || is_english_punctuation(c)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_is_cjk() {
        assert!(is_cjk('你'));
        assert!(is_cjk('〇'));
        assert!(is_cjk('䀹'));
        assert!(is_cjk('鿃'));
        assert!(is_cjk('\u{9FEB}'));
        assert!(is_cjk('﨧'));
        assert!(is_cjk('𱞈'));

        assert!(!is_cjk('a'));
        assert!(!is_cjk('🍌'));
    }
}
