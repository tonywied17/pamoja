//! Canonical localization store for the pamoja project.
//!
//! Translations live once, here, as Fluent `.ftl` files embedded at compile time. This
//! crate is the shared source two consumers read: the dashboard's browser bundles, which
//! `cargo xtask dashboard i18n` generates from these same files, and on-device Rust
//! rendering (and, later, Sema), which uses the runtime in this module.
//!
//! Rendering resolves a message for a locale through [`fluent_bundle`], with plural
//! selection over CLDR categories and `NUMBER()` digit shaping through [`icu`] so a count
//! reads in the locale's own numbering system (Latin, Eastern Arabic, Devanagari). An
//! unknown locale, or a key a locale has not translated, falls back to English.
//!
//! Message keys are the dotted form the dashboard uses (`ui.status`). Fluent message ids
//! cannot contain dots, so on disk each dot is written as `__` (`ui__status`); callers
//! always use the dotted key and the mapping is internal.
//!
//! # Examples
//!
//! ```
//! use fluent_bundle::{FluentArgs, FluentValue};
//!
//! // A simple message.
//! assert_eq!(pamoja_i18n::render("en", "ui.status", &FluentArgs::new()), "System status");
//!
//! // A counted message selects the plural variant and shapes the digits per locale.
//! let mut args = FluentArgs::new();
//! args.set("n", FluentValue::from(3));
//! assert_eq!(pamoja_i18n::render("en", "ui.sensorsCount", &args), "3 sensors");
//! ```

use std::cell::RefCell;
use std::collections::HashMap;

use fluent_bundle::{FluentArgs, FluentBundle, FluentResource, FluentValue};
use icu::decimal::input::Decimal;
use icu::decimal::options::DecimalFormatterOptions;
use icu::decimal::DecimalFormatter;
use icu::locale::Locale;
use unic_langid::LanguageIdentifier;

/// A shipped locale: its tag, text direction, CLDR numbering system, and embedded source.
struct LocaleEntry {
    tag: &'static str,
    dir: &'static str,
    numbering: &'static str,
    ftl: &'static str,
}

/// The seed locales, English first as the source and fallback.
const LOCALES: &[LocaleEntry] = &[
    LocaleEntry {
        tag: "en",
        dir: "ltr",
        numbering: "latn",
        ftl: include_str!("../i18n/en.ftl"),
    },
    LocaleEntry {
        tag: "sw",
        dir: "ltr",
        numbering: "latn",
        ftl: include_str!("../i18n/sw.ftl"),
    },
    LocaleEntry {
        tag: "ar",
        dir: "rtl",
        numbering: "arab",
        ftl: include_str!("../i18n/ar.ftl"),
    },
    LocaleEntry {
        tag: "fr",
        dir: "ltr",
        numbering: "latn",
        ftl: include_str!("../i18n/fr.ftl"),
    },
    LocaleEntry {
        tag: "pt",
        dir: "ltr",
        numbering: "latn",
        ftl: include_str!("../i18n/pt.ftl"),
    },
    LocaleEntry {
        tag: "hi",
        dir: "ltr",
        numbering: "deva",
        ftl: include_str!("../i18n/hi.ftl"),
    },
];

thread_local! {
    /// Per-thread cache of decimal formatters, keyed by locale tag, so a hot render does
    /// not rebuild one each call. `DecimalFormatter` is not `Sync`, so it cannot be shared
    /// across threads, which a thread-local sidesteps.
    static FORMATTERS: RefCell<HashMap<&'static str, DecimalFormatter>> = RefCell::new(HashMap::new());
}

/// The tags of the shipped locales, English first.
///
/// # Returns
///
/// The locale tags in menu order.
pub fn locales() -> impl Iterator<Item = &'static str> {
    LOCALES.iter().map(|entry| entry.tag)
}

fn entry(locale: &str) -> &'static LocaleEntry {
    LOCALES
        .iter()
        .find(|entry| entry.tag == locale)
        .unwrap_or(&LOCALES[0])
}

/// The text direction for a locale, `"ltr"` or `"rtl"`.
///
/// # Arguments
///
/// * `locale` - the locale tag; an unknown tag falls back to English.
///
/// # Returns
///
/// The text direction.
pub fn direction(locale: &str) -> &'static str {
    entry(locale).dir
}

/// The CLDR numbering system a locale formats numbers with.
///
/// # Arguments
///
/// * `locale` - the locale tag; an unknown tag falls back to English.
///
/// # Returns
///
/// The numbering system identifier, such as `"latn"`, `"arab"`, or `"deva"`.
pub fn numbering_system(locale: &str) -> &'static str {
    entry(locale).numbering
}

// Shapes an integer in a locale's numbering system, caching the formatter per thread.
fn shape(tag: &'static str, numbering: &'static str, value: f64) -> String {
    FORMATTERS.with(|cache| {
        let mut map = cache.borrow_mut();
        let formatter = map.entry(tag).or_insert_with(|| {
            let locale: Locale = format!("{tag}-u-nu-{numbering}")
                .parse()
                .expect("locale parses");
            DecimalFormatter::try_new((&locale).into(), DecimalFormatterOptions::default())
                .expect("formatter builds from compiled data")
        });
        formatter.format_to_string(&Decimal::from(value as i64))
    })
}

fn build_bundle(entry: &'static LocaleEntry) -> FluentBundle<FluentResource> {
    let resource = FluentResource::try_new(entry.ftl.to_owned()).expect("bundled ftl parses");
    let langid: LanguageIdentifier = entry.tag.parse().expect("locale tag parses");
    let mut bundle = FluentBundle::new(vec![langid]);
    bundle.set_use_isolating(false);
    let tag = entry.tag;
    let numbering = entry.numbering;
    bundle
        .add_function("NUMBER", move |positional, _named| {
            match positional.first() {
                Some(FluentValue::Number(number)) => {
                    FluentValue::String(shape(tag, numbering, number.value).into())
                }
                Some(other) => other.clone(),
                None => FluentValue::None,
            }
        })
        .expect("register NUMBER");
    bundle.add_resource(resource).expect("add resource");
    bundle
}

/// Renders a message for a locale, falling back to English for an unknown locale or an
/// untranslated key.
///
/// # Arguments
///
/// * `locale` - the locale tag, such as `"ar"`.
/// * `key` - the dotted message key, such as `"ui.status"` or `"ui.sensorsCount"`.
/// * `args` - the Fluent arguments, such as `$n` for a counted message.
///
/// # Returns
///
/// The resolved string, with the plural variant and numbering system for `locale`.
pub fn render(locale: &str, key: &str, args: &FluentArgs) -> String {
    let id = key.replace('.', "__");
    let primary = entry(locale);
    if let Some(text) = render_in(primary, &id, args) {
        return text;
    }
    render_in(&LOCALES[0], &id, args).unwrap_or_else(|| key.to_owned())
}

fn render_in(entry: &'static LocaleEntry, id: &str, args: &FluentArgs) -> Option<String> {
    let bundle = build_bundle(entry);
    let message = bundle.get_message(id)?;
    let pattern = message.value()?;
    let mut errors = Vec::new();
    Some(
        bundle
            .format_pattern(pattern, Some(args), &mut errors)
            .into_owned(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use fluent_syntax::parser;

    fn ids(ftl: &str) -> std::collections::BTreeSet<String> {
        let resource = parser::parse(ftl).expect("ftl parses");
        resource
            .body
            .into_iter()
            .filter_map(|entry| match entry {
                fluent_syntax::ast::Entry::Message(message) => Some(message.id.name.to_owned()),
                _ => None,
            })
            .collect()
    }

    #[test]
    fn every_locale_parses_and_matches_the_english_id_set() {
        let english = ids(LOCALES[0].ftl);
        assert!(!english.is_empty());
        for entry in LOCALES {
            assert_eq!(ids(entry.ftl), english, "id set drift in {}", entry.tag);
        }
    }

    #[test]
    fn plural_selection_follows_cldr_categories() {
        let count = |locale: &str, n: i64| {
            let mut args = FluentArgs::new();
            args.set("n", FluentValue::from(n));
            render(locale, "ui.sensorsCount", &args)
        };
        // English: 1 is `one`, everything else `other`.
        assert_eq!(count("en", 1), "1 sensor");
        assert_eq!(count("en", 5), "5 sensors");
        // French: 0 and 1 are both `one`.
        assert_eq!(count("fr", 1), "1 capteur");
        assert_eq!(count("fr", 2), "2 capteurs");
        // Arabic exercises the `two` and `few` categories and Eastern Arabic digits.
        assert_eq!(count("ar", 2), "٢ مستشعران");
        assert_eq!(count("ar", 3), "٣ مستشعرات");
    }

    #[test]
    fn numbering_system_shapes_digits_per_locale() {
        let mut args = FluentArgs::new();
        args.set("n", FluentValue::from(123));
        assert_eq!(render("hi", "ui.sensorsCount", &args), "१२३ सेंसर");
        assert_eq!(numbering_system("ar"), "arab");
        assert_eq!(direction("ar"), "rtl");
    }

    #[test]
    fn an_unknown_locale_falls_back_to_english() {
        assert_eq!(
            render("zz", "ui.status", &FluentArgs::new()),
            "System status"
        );
    }

    #[test]
    fn an_empty_valued_message_renders_empty() {
        assert_eq!(render("en", "unit.state", &FluentArgs::new()), "");
    }
}
