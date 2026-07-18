use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Translation {
    pub translationid: i64,
    pub abbreviation: String,
    pub name: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Book {
    pub bookid: i64,
    pub testamentid: i64,
    pub name: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Segment {
    Text { text: String, wj: bool },
    FootnoteMarker { index: i64 },
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Footnote {
    pub reference: Option<String>,
    pub text: Option<String>,
    #[serde(rename = "alternateReading")]
    pub alternate_reading: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Verse {
    pub verseid: i64,
    pub translationid: i64,
    pub bookid: i64,
    pub book_name: String,
    pub chapternumber: i64,
    pub versenumber: String, // kept as string: "16" or "16-17"
    pub versetext: String,
    pub haswj: bool,
    pub hasfootnotes: bool,
    pub segments: Option<Vec<Segment>>,
    pub footnotes: Option<Vec<Footnote>>,
}

/// First integer in a versenumber, used for sorting/navigation on ranges like "14-15"
impl Verse {
    pub fn verse_sort_key(&self) -> i64 {
        self.versenumber
            .split(|c: char| !c.is_ascii_digit())
            .find(|s| !s.is_empty())
            .and_then(|s| s.parse().ok())
            .unwrap_or(0)
    }
}
