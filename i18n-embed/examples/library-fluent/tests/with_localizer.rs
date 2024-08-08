use i18n_embed::{LanguageLoader, Localizer};
use library_fluent::{hello_world, localizer};

use std::collections::HashSet;
use std::iter::FromIterator;

/// Test that the expected languages and fallback language are
/// available.
#[test]
fn test_available_languages() {
    let localizer = localizer();
    assert_eq!(
        &localizer.language_loader.fallback_language().to_string(),
        "en"
    );

    let available_ids: HashSet<String> = HashSet::from_iter(
        localizer
            .available_languages()
            .unwrap()
            .into_iter()
            .map(|id| id.to_string()),
    );

    let expected_available_ids: HashSet<String> =
        HashSet::from_iter(vec!["en".to_string(), "fr".to_string(), "eo".to_string()]);

    assert_eq!(available_ids, expected_available_ids)
}

/// Test loading the `en` language.
#[test]
fn test_select_english() {
    localizer().select(&["en".parse().unwrap()]).unwrap();
    assert_eq!("Hello World!", &hello_world())
}

/// Test loading the `fr` language.
#[test]
fn test_select_french() {
    localizer().select(&["fr".parse().unwrap()]).unwrap();
    assert_eq!("Bonjour le monde!", &hello_world())
}
