use once_cell::sync::Lazy;
use std::collections::HashMap;

pub static BOOK_ALIASES: Lazy<HashMap<&'static str, &'static str>> = Lazy::new(|| {
    HashMap::from([
        ("gen", "Genesis"), ("ge", "Genesis"), ("gn", "Genesis"),
        ("ex", "Exodus"), ("exo", "Exodus"), ("exod", "Exodus"),
        ("lev", "Leviticus"), ("lv", "Leviticus"), ("levi", "Leviticus"),
        ("num", "Numbers"), ("nu", "Numbers"), ("nb", "Numbers"), ("numb", "Numbers"),
        ("deut", "Deuteronomy"), ("dt", "Deuteronomy"), ("deu", "Deuteronomy"),
        ("josh", "Joshua"), ("jos", "Joshua"), ("jsh", "Joshua"),
        ("judg", "Judges"), ("jdg", "Judges"), ("jud", "Judges"), ("jgs", "Judges"),
        ("ru", "Ruth"), ("rut", "Ruth"),
        ("1sam", "1 Samuel"), ("1 sam", "1 Samuel"), ("1sa", "1 Samuel"), ("1st sam", "1 Samuel"), ("isam", "1 Samuel"),
        ("2sam", "2 Samuel"), ("2 sam", "2 Samuel"), ("2sa", "2 Samuel"), ("2nd sam", "2 Samuel"), ("iisam", "2 Samuel"),
        ("1kgs", "1 Kings"), ("1ki", "1 Kings"), ("1 kgs", "1 Kings"), ("1st kgs", "1 Kings"), ("ikgs", "1 Kings"),
        ("2kgs", "2 Kings"), ("2ki", "2 Kings"), ("2 kgs", "2 Kings"), ("2nd kgs", "2 Kings"), ("iikgs", "2 Kings"),
        ("1chr", "1 Chronicles"), ("1ch", "1 Chronicles"), ("1 chron", "1 Chronicles"), ("1chron", "1 Chronicles"),
        ("2chr", "2 Chronicles"), ("2ch", "2 Chronicles"), ("2 chron", "2 Chronicles"), ("2chron", "2 Chronicles"),
        ("ezr", "Ezra"),
        ("neh", "Nehemiah"), ("ne", "Nehemiah"),
        ("est", "Esther"), ("esth", "Esther"),
        ("job", "Job"),
        ("ps", "Psalms"), ("psa", "Psalms"), ("psalm", "Psalms"), ("pss", "Psalms"), ("psl", "Psalms"),
        ("prov", "Proverbs"), ("pr", "Proverbs"), ("prv", "Proverbs"), ("pro", "Proverbs"),
        ("eccl", "Ecclesiastes"), ("ecc", "Ecclesiastes"), ("qoh", "Ecclesiastes"), ("eccles", "Ecclesiastes"),
        ("song", "Song of Songs"), ("sos", "Song of Songs"), ("sng", "Song of Songs"), ("canticles", "Song of Songs"), ("cant", "Song of Songs"),
        ("isa", "Isaiah"), ("is", "Isaiah"), ("isai", "Isaiah"),
        ("jer", "Jeremiah"), ("je", "Jeremiah"), ("jere", "Jeremiah"),
        ("lam", "Lamentations"), ("la", "Lamentations"),
        ("ezek", "Ezekiel"), ("eze", "Ezekiel"), ("ezk", "Ezekiel"),
        ("dan", "Daniel"), ("da", "Daniel"), ("dn", "Daniel"),
        ("hos", "Hosea"), ("ho", "Hosea"),
        ("joel", "Joel"), ("jl", "Joel"),
        ("amos", "Amos"), ("am", "Amos"),
        ("obad", "Obadiah"), ("ob", "Obadiah"),
        ("jon", "Jonah"), ("jnh", "Jonah"),
        ("mic", "Micah"), ("mi", "Micah"),
        ("nah", "Nahum"), ("na", "Nahum"),
        ("hab", "Habakkuk"), ("hb", "Habakkuk"),
        ("zeph", "Zephaniah"), ("zep", "Zephaniah"), ("zp", "Zephaniah"),
        ("hag", "Haggai"), ("hg", "Haggai"),
        ("zech", "Zechariah"), ("zec", "Zechariah"), ("zc", "Zechariah"),
        ("mal", "Malachi"), ("ml", "Malachi"),

        ("mt", "Matthew"), ("matt", "Matthew"), ("mat", "Matthew"),
        ("mk", "Mark"), ("mrk", "Mark"), ("mr", "Mark"),
        ("lk", "Luke"), ("luk", "Luke"), ("lu", "Luke"),
        ("jn", "John"), ("jhn", "John"), ("joh", "John"),
        ("acts", "Acts"), ("ac", "Acts"), ("act", "Acts"),
        ("rom", "Romans"), ("ro", "Romans"), ("rm", "Romans"),
        ("1cor", "1 Corinthians"), ("1co", "1 Corinthians"), ("1 cor", "1 Corinthians"), ("icor", "1 Corinthians"),
        ("2cor", "2 Corinthians"), ("2co", "2 Corinthians"), ("2 cor", "2 Corinthians"), ("iicor", "2 Corinthians"),
        ("gal", "Galatians"), ("ga", "Galatians"),
        ("eph", "Ephesians"), ("ephes", "Ephesians"),
        ("phil", "Philippians"), ("php", "Philippians"), ("philip", "Philippians"),
        ("col", "Colossians"), ("colo", "Colossians"),
        ("1thess", "1 Thessalonians"), ("1th", "1 Thessalonians"), ("1thes", "1 Thessalonians"), ("1thss", "1 Thessalonians"), ("1 th", "1 Thessalonians"),
        ("2thess", "2 Thessalonians"), ("2th", "2 Thessalonians"), ("2thes", "2 Thessalonians"), ("2thss", "2 Thessalonians"), ("2 th", "2 Thessalonians"),
        ("1tim", "1 Timothy"), ("1ti", "1 Timothy"), ("1 tim", "1 Timothy"),
        ("2tim", "2 Timothy"), ("2ti", "2 Timothy"), ("2 tim", "2 Timothy"),
        ("tit", "Titus"), ("ti", "Titus"),
        ("phlm", "Philemon"), ("phm", "Philemon"), ("philem", "Philemon"),
        ("heb", "Hebrews"), ("hebr", "Hebrews"),
        ("jas", "James"), ("jm", "James"), ("ja", "James"),
        ("1pet", "1 Peter"), ("1pe", "1 Peter"), ("1pt", "1 Peter"), ("1 pet", "1 Peter"),
        ("2pet", "2 Peter"), ("2pe", "2 Peter"), ("2pt", "2 Peter"), ("2 pet", "2 Peter"),
        ("1jn", "1 John"), ("1 jn", "1 John"), ("1jo", "1 John"), ("1john", "1 John"),
        ("2jn", "2 John"), ("2 jn", "2 John"), ("2jo", "2 John"), ("2john", "2 John"),
        ("3jn", "3 John"), ("3 jn", "3 John"), ("3jo", "3 John"), ("3john", "3 John"),
        ("jude", "Jude"), ("jud", "Jude"), ("jd", "Jude"),
        ("rev", "Revelation"), ("re", "Revelation"), ("rv", "Revelation"), ("apoc", "Revelation"),
    ])
});

#[derive(Debug)]
pub enum ParsedQuery {
    Reference { book: String, chapter: i64, verse: Option<String> },
    Keyword { term: String, book: Option<String> },
}

pub fn parse_query(input: &str) -> ParsedQuery {
    let trimmed = input.trim();

    // "wisdom understanding; prov" -> keyword search scoped to Proverbs
    if let Some((kw, book_part)) = trimmed.split_once(';') {
        let term = kw.trim().to_string();
        let book_raw = book_part.trim();
        let book = if book_raw.is_empty() { None } else { Some(resolve_book(book_raw)) };
        return ParsedQuery::Keyword { term, book };
    }

    let re = regex::Regex::new(
        r"(?i)^(\d?\s?[a-zA-Z]+)\.?\s*(\d+)(?:[:\.\s](\d+(?:-\d+)?))?$"
    ).unwrap();

    if let Some(caps) = re.captures(trimmed) {
        let raw_book = caps.get(1).unwrap().as_str().trim();
        let chapter: i64 = caps.get(2).unwrap().as_str().parse().unwrap_or(1);
        let verse = caps.get(3).map(|m| m.as_str().to_string());

        return ParsedQuery::Reference {
            book: resolve_book(raw_book),
            chapter,
            verse,
        };
    }

    ParsedQuery::Keyword { term: trimmed.to_string(), book: None }
}

pub fn resolve_book(raw: &str) -> String {
    let key = raw.to_lowercase();
    BOOK_ALIASES
        .get(key.as_str())
        .map(|s| s.to_string())
        .unwrap_or_else(|| raw.to_string()) // fall through: assume it's already a canonical/full name
}
