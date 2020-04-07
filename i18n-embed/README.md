# i18n-embed

## Example

```rust
use i18n_embed::{I18nEmbed, LanguageLoader, DesktopLanguageRequester};
use rust_embed::RustEmbed;

#[derive(RustEmbed, I18nEmbed)]
#[folder = "i18n/mo"] // path to the compiled localization resources
struct Translations;

#[derive(LanguageLoader)]
struct MyLanguageLoader;

fn main() {
    let language_loader = MyLanguageLoader {};

    // Use the language requester for the desktop platform (linux, windows, mac).
    // There is also a requester available for the web-sys WASM platform called
    // WebLanguageRequester, or you can implement your own.
    let language_requester = DesktopLanguageRequester::new();
    Translations::select(&language_requester, &language_loader);
}
```
