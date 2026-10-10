//! The command palette's search, without the interface: text folding
//! (case and accents), the fuzzy matcher, the ranking of results and the
//! rows shown for an empty query.

use std::cmp::Reverse;

use crate::commands::CommandId;

/// At most this many recent commands are remembered.
pub const RECENT_MAX: usize = 5;

/// Text lowercased and without accents, with, for each of its chars, the
/// index of the char of the source text it comes from (`ß` gives two `s`
/// coming from one char).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Folded {
    pub chars: Vec<char>,
    pub origin: Vec<usize>,
}

/// The letters of French, German and Spanish without their accents.
fn unaccent(c: char) -> Option<&'static str> {
    Some(match c {
        'à' | 'â' | 'ä' | 'á' | 'ã' | 'å' => "a",
        'ç' => "c",
        'é' | 'è' | 'ê' | 'ë' => "e",
        'í' | 'ì' | 'î' | 'ï' => "i",
        'ñ' => "n",
        'ó' | 'ò' | 'ô' | 'ö' | 'õ' => "o",
        'ú' | 'ù' | 'û' | 'ü' => "u",
        'ý' | 'ÿ' => "y",
        'ß' => "ss",
        'æ' => "ae",
        'œ' => "oe",
        _ => return None,
    })
}

/// Folds `text` for matching: lowercase, accents removed.
pub fn fold(text: &str) -> Folded {
    let mut chars = Vec::with_capacity(text.len());
    let mut origin = Vec::with_capacity(text.len());
    for (i, c) in text.chars().enumerate() {
        for lower in c.to_lowercase() {
            match unaccent(lower) {
                Some(plain) => {
                    for p in plain.chars() {
                        chars.push(p);
                        origin.push(i);
                    }
                }
                None => {
                    chars.push(lower);
                    origin.push(i);
                }
            }
        }
    }
    Folded { chars, origin }
}

/// How well an entry matches a query.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Match {
    /// Higher is better.
    pub score: i32,
    /// The chars of the label that match (indices of chars, in order).
    pub label_hits: Vec<usize>,
}

/// Score of a matched letter that starts a word.
const WORD_START: i32 = 8;
/// Score of a matched letter right after the previous one.
const CONTIGUOUS: i32 = 4;
/// Most a gap between two matched letters costs.
const GAP_MAX: i32 = 6;

/// The best alignment of `word` as a subsequence of `text`: its score and
/// the indices of the matched chars. Every occurrence of its first letter is
/// tried as a start, the rest matched left to right.
fn match_word(word: &[char], text: &[char]) -> Option<(i32, Vec<usize>)> {
    let first = *word.first()?;
    let mut best: Option<(i32, Vec<usize>)> = None;
    for start in (0..text.len()).filter(|&i| text[i] == first) {
        let mut hits = vec![start];
        let mut pos = start;
        let mut found = true;
        for &c in &word[1..] {
            match text[pos + 1..].iter().position(|&t| t == c) {
                Some(offset) => {
                    pos += 1 + offset;
                    hits.push(pos);
                }
                None => {
                    found = false;
                    break;
                }
            }
        }
        if !found {
            // Later starts can only find fewer letters after them.
            break;
        }
        let mut score = 0;
        for (k, &j) in hits.iter().enumerate() {
            score += 1;
            if j == 0 || !text[j - 1].is_alphanumeric() {
                score += WORD_START;
            }
            if k > 0 {
                let gap = (j - hits[k - 1] - 1) as i32;
                score += if gap == 0 {
                    CONTIGUOUS
                } else {
                    -gap.min(GAP_MAX)
                };
            }
        }
        if best.as_ref().is_none_or(|(b, _)| score > *b) {
            best = Some((score, hits));
        }
    }
    best
}

/// The query's words, folded.
pub fn words(query: &str) -> Vec<Vec<char>> {
    query
        .split_whitespace()
        .map(|w| fold(w).chars)
        .filter(|w| !w.is_empty())
        .collect()
}

/// Matches `words` against an entry: each word must be found, letters in
/// order, in its label, else in its context. A word found in the label
/// counts double; a shorter label wins over a longer one with the same
/// letters matched. `None` when a word is found in neither.
pub fn score(words: &[Vec<char>], label: &str, context: &str) -> Option<Match> {
    let label_folded = fold(label);
    let context_folded = fold(context);
    let mut total = 0;
    let mut hits = Vec::new();
    for word in words {
        if let Some((s, h)) = match_word(word, &label_folded.chars) {
            total += 2 * s;
            hits.extend(h.into_iter().map(|i| label_folded.origin[i]));
        } else {
            let (s, _) = match_word(word, &context_folded.chars)?;
            total += s;
        }
    }
    hits.sort_unstable();
    hits.dedup();
    // Scaled so that the label's length only breaks ties.
    let length = label.chars().count() as i32;
    Some(Match {
        score: total * 8 - length,
        label_hits: hits,
    })
}

/// What an entry of the palette opens.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Target {
    Command(CommandId),
    /// The project's surface of this index.
    Texture(usize),
}

/// An entry of the palette, as searched.
#[derive(Clone, Debug, PartialEq)]
pub struct Entry {
    pub target: Target,
    /// Its label in the interface language.
    pub label: String,
    /// Its menu path, group, or vehicle and part, joined with " › ".
    pub context: String,
    /// A disabled command; textures are always enabled.
    pub enabled: bool,
}

/// A row of the list.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Row {
    /// A heading (message id): Recent, Textures, Commands.
    Heading(&'static str),
    /// Entry `entry` with the matched chars of its label.
    Result { entry: usize, hits: Vec<usize> },
}

impl Row {
    /// The entry of a result row.
    pub fn entry(&self) -> Option<usize> {
        match self {
            Self::Result { entry, .. } => Some(*entry),
            Self::Heading(_) => None,
        }
    }
}

/// Indices of the entries matching `query`, best first: enabled entries
/// before disabled ones, then by score, then in list order.
pub fn rank(entries: &[Entry], query: &str) -> Vec<(usize, Match)> {
    let words = words(query);
    let mut found: Vec<(usize, Match)> = entries
        .iter()
        .enumerate()
        .filter_map(|(i, e)| score(&words, &e.label, &e.context).map(|m| (i, m)))
        .collect();
    found.sort_by_key(|(i, m)| (!entries[*i].enabled, Reverse(m.score), *i));
    found
}

/// The rows of the list. With an empty query: the `recent` commands (most
/// recent first) under Recent, the textures under Textures, then the other
/// commands in list order under Commands, each heading only when it has
/// rows. With a query: the ranked results, without headings.
pub fn rows(entries: &[Entry], query: &str, recent: &[CommandId]) -> Vec<Row> {
    if !words(query).is_empty() {
        return rank(entries, query)
            .into_iter()
            .map(|(entry, m)| Row::Result {
                entry,
                hits: m.label_hits,
            })
            .collect();
    }
    let result = |entry| Row::Result {
        entry,
        hits: Vec::new(),
    };
    let command_index =
        |id: CommandId| entries.iter().position(|e| e.target == Target::Command(id));
    let recent: Vec<usize> = recent.iter().filter_map(|&id| command_index(id)).collect();
    let textures: Vec<usize> = (0..entries.len())
        .filter(|&i| matches!(entries[i].target, Target::Texture(_)))
        .collect();
    let commands: Vec<usize> = (0..entries.len())
        .filter(|&i| matches!(entries[i].target, Target::Command(_)) && !recent.contains(&i))
        .collect();
    let mut rows = Vec::new();
    for (heading, group) in [
        ("palette-heading-recent", recent),
        ("palette-heading-textures", textures),
        ("palette-heading-commands", commands),
    ] {
        if !group.is_empty() {
            rows.push(Row::Heading(heading));
            rows.extend(group.into_iter().map(result));
        }
    }
    rows
}

/// Records `id` as the most recent command run from the palette.
pub fn push_recent(recent: &mut Vec<CommandId>, id: CommandId) {
    recent.retain(|r| *r != id);
    recent.insert(0, id);
    recent.truncate(RECENT_MAX);
}

/// A command's context: its menu path or group, translated and joined with
/// " › ".
pub fn context(path: &[&str]) -> String {
    path.iter()
        .map(|key| tp_i18n::tr(key))
        .collect::<Vec<_>>()
        .join(" › ")
}

#[cfg(test)]
mod tests;
