use library_fluent::{hello_world, localizer};

/// Test that the expected languages and fallback language are
/// available.
#[test]
fn test_available_languages() {
    let localizer = localizer();
    assert_eq!(
        &localizer.language_loader().fallback_language().to_string(),
        "en"
    );

    let available_ids: Vec<String> = localizer
        .available_languages()
        .unwrap()
        .into_iter()
        .map(|id| id.to_string())
        .collect();

    let expected_available_ids: Vec<String> =
        vec!["en".to_string(), "fr".to_string(), "eo".to_string()];

    assert_eq!(available_ids, expected_available_ids)
}

/// Test loading the `en` language.
#[test]
fn test_select_english() {
    localizer().select(&["en".parse().unwrap()]).unwrap();
    assert_eq!(&hello_world(), "Hello World!")
}

/// Test loading the `fr` language.
#[test]
fn test_select_french() {
    localizer().select(&["fr".parse().unwrap()]).unwrap();
    assert_eq!(&hello_world(), "Bonjour le monde!")
}
