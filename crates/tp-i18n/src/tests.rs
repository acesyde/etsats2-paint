use std::collections::{BTreeMap, BTreeSet};

use fluent_syntax::ast;

use super::*;

#[test]
fn locales_map_to_languages() {
    assert_eq!(Language::from_locale("fr-CA"), Language::French);
    assert_eq!(Language::from_locale("fr"), Language::French);
    assert_eq!(Language::from_locale("de_AT.UTF-8"), Language::German);
    assert_eq!(Language::from_locale("es-MX"), Language::Spanish);
    assert_eq!(Language::from_locale("ES"), Language::Spanish);
    assert_eq!(Language::from_locale("en-GB"), Language::English);
    assert_eq!(Language::from_locale("it-IT"), Language::English);
    assert_eq!(Language::from_locale("%%%"), Language::English);
    assert_eq!(Language::from_locale(""), Language::English);
    for l in Language::ALL {
        assert_eq!(Language::from_code(l.code()), Some(l));
    }
}

#[test]
fn lookup_fallback_and_isolation() {
    set_language(Language::French);
    assert_eq!(tr("language-system"), "Langue du système");
    // A message only in English (test file) falls back to English.
    assert_eq!(tr("test-english-only"), "Only in English");
    // Unknown ids show the id rather than nothing.
    assert_eq!(tr("no-such-message"), "no-such-message");
    // Another thread keeps its own language (English by default).
    let other = std::thread::spawn(|| tr("language-system")).join().unwrap();
    assert_eq!(other, "System default");
    set_language(Language::German);
    assert_eq!(tr("language-system"), "Systemsprache");
    set_language(Language::English);
}

#[test]
fn arguments_plurals_and_no_isolation_marks() {
    set_language(Language::Spanish);
    assert_eq!(tr!("time-ago-days", count = 2), "hace 2 días");
    set_language(Language::French);
    assert_eq!(tr!("time-ago-hours", count = 1), "il y a 1 heure");
    let text = tr!("time-ago-days", count = 5);
    assert!(
        !text.contains('\u{2068}') && !text.contains('\u{2069}'),
        "{text:?}"
    );
    set_language(Language::English);
    assert_eq!(tr!("time-ago-minutes", count = 1), "1 minute ago");
    assert_eq!(tr!("time-ago-minutes", count = 3), "3 minutes ago");
}

#[test]
fn numbers_follow_the_language() {
    set_language(Language::English);
    assert_eq!(format_number(12.5, 1), "12.5");
    assert_eq!(format_number(12.0, 2), "12");
    assert_eq!(format_number(-0.0001, 1), "0");
    for l in [Language::French, Language::Spanish, Language::German] {
        set_language(l);
        assert_eq!(decimal_separator(), ',');
        assert_eq!(format_number(12.5, 1), "12,5");
        assert_eq!(format_number(0.126, 2), "0,13");
        assert_eq!(format_number(400.0, 0), "400");
    }
    set_language(Language::English);
}

/// Ids and the variables each message uses, for one language.
fn messages(language: Language) -> BTreeMap<String, BTreeSet<String>> {
    fn pattern_vars(p: &ast::Pattern<&str>, out: &mut BTreeSet<String>) {
        for el in &p.elements {
            if let ast::PatternElement::Placeable { expression } = el {
                expr_vars(expression, out);
            }
        }
    }
    fn inline_vars(e: &ast::InlineExpression<&str>, out: &mut BTreeSet<String>) {
        match e {
            ast::InlineExpression::VariableReference { id } => {
                out.insert(id.name.to_owned());
            }
            ast::InlineExpression::FunctionReference { arguments, .. } => {
                for a in &arguments.positional {
                    inline_vars(a, out);
                }
            }
            ast::InlineExpression::Placeable { expression } => expr_vars(expression, out),
            _ => {}
        }
    }
    fn expr_vars(e: &ast::Expression<&str>, out: &mut BTreeSet<String>) {
        match e {
            ast::Expression::Inline(i) => inline_vars(i, out),
            ast::Expression::Select { selector, variants } => {
                inline_vars(selector, out);
                for v in variants {
                    pattern_vars(&v.value, out);
                }
            }
        }
    }
    let mut out = BTreeMap::new();
    for (file, source) in FILES.iter().zip(sources(language)) {
        let resource = fluent_syntax::parser::parse(source)
            .unwrap_or_else(|(_, e)| panic!("{}/{file}.ftl: {e:?}", language.code()));
        for entry in resource.body {
            if let ast::Entry::Message(m) = entry {
                let mut vars = BTreeSet::new();
                if let Some(v) = &m.value {
                    pattern_vars(v, &mut vars);
                }
                let id = m.id.name.to_owned();
                assert!(
                    out.insert(id.clone(), vars).is_none(),
                    "{}: {id} defined twice",
                    language.code()
                );
            }
        }
    }
    out
}

#[test]
fn every_language_has_every_message_with_the_same_variables() {
    let english = messages(Language::English);
    assert!(english.len() > 10);
    for language in [Language::French, Language::Spanish, Language::German] {
        let other = messages(language);
        let mut problems = Vec::new();
        for (id, vars) in &english {
            if id.starts_with("test-") {
                continue;
            }
            match other.get(id) {
                None => problems.push(format!("missing {id}")),
                Some(v) if v != vars => problems.push(format!("{id}: {v:?} ≠ {vars:?}")),
                Some(_) => {}
            }
        }
        for id in other.keys() {
            if !english.contains_key(id) {
                problems.push(format!("extra {id}"));
            }
        }
        assert!(problems.is_empty(), "{}: {problems:#?}", language.code());
    }
}

#[test]
fn every_message_formats_without_errors() {
    for language in Language::ALL {
        let bundle = build_bundle(language);
        for (id, vars) in messages(language) {
            let message = bundle.get_message(&id).expect("parsed message");
            let Some(pattern) = message.value() else {
                panic!("{}: {id} has no value", language.code());
            };
            for sample in [1, 2, 5] {
                let mut args = FluentArgs::new();
                for v in &vars {
                    args.set(v.clone(), sample);
                }
                let mut errors = Vec::new();
                let text = bundle.format_pattern(pattern, Some(&args), &mut errors);
                assert!(errors.is_empty(), "{}: {id}: {errors:?}", language.code());
                assert!(
                    !text.trim().is_empty(),
                    "{}: {id} is empty",
                    language.code()
                );
            }
        }
    }
}

#[test]
fn lengthening_makes_messages_longer() {
    set_language(Language::English);
    set_lengthening(40);
    let long = tr("language-system");
    set_lengthening(0);
    let short = tr("language-system");
    assert!(long.starts_with(&short));
    assert_eq!(
        long.chars().count(),
        short.chars().count() + (short.chars().count() * 40).div_ceil(100)
    );
}
