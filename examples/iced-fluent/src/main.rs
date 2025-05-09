use iced::{
    pick_list, scrollable, Align, Column, Container, Element, Length, PickList, Sandbox,
    Scrollable, Settings, Space, Text,
};
use unic_langid::LanguageIdentifier;

use i18n_embed::{
    fluent::{fluent_language_loader, FluentLanguageLoader},
    select, DesktopLanguageRequester, LanguageLoader,
};
use rust_embed::RustEmbed;

#[derive(RustEmbed)]
#[folder = "i18n"] // path to the compiled localization resources
struct Translations;

const TRANSLATIONS: Translations = Translations {};

lazy_static::lazy_static! {
    static ref LANGUAGE_LOADER: FluentLanguageLoader = fluent_language_loader!();
}

macro_rules! fl {
    ($message_id:literal) => {{
        i18n_embed_fl::fl!(LANGUAGE_LOADER, $message_id)
    }};

    ($message_id:literal, $($args:expr),*) => {{
        i18n_embed_fl::fl!(LANGUAGE_LOADER, $message_id, $($args), *)
    }};
}

pub fn main() -> iced::Result {
    Example::run(Settings::default())
}

struct Example {
    available_languages: Vec<LanguageIdentifier>,
    pick_list: pick_list::State<LanguageIdentifier>,
    selected_language: LanguageIdentifier,
}

#[derive(Debug, Clone)]
enum Message {
    LanguageSelected(LanguageIdentifier),
}

impl Sandbox for Example {
    type Message = Message;

    fn new() -> Self {
        let available_languages = (*LANGUAGE_LOADER)
            .available_languages(&TRANSLATIONS)
            .unwrap();
        let requested_languages = DesktopLanguageRequester::requested_languages();
        let selected_languages =
            select(&*LANGUAGE_LOADER, &TRANSLATIONS, &requested_languages).unwrap();
        let selected_language = selected_languages.get(0).unwrap().clone();

        Self {
            available_languages,
            pick_list: pick_list::State::default(),
            selected_language,
        }
    }

    fn title(&self) -> String {
        String::from(fl!("window-title"))
    }

    fn update(&mut self, message: Message) {
        match message {
            Message::LanguageSelected(language) => {
                let requested_languages = vec![language];
                let selected_languages =
                    select(&*LANGUAGE_LOADER, &TRANSLATIONS, &requested_languages).unwrap();
                self.selected_language = selected_languages.get(0).unwrap().clone();
            }
        }
    }

    fn view(&mut self) -> Element<Message> {
        let pick_list = PickList::new(
            &mut self.pick_list,
            self.available_languages.as_slice(),
            Some(self.selected_language.clone()),
            Message::LanguageSelected,
        );

        Column::new()
            .padding(20)
            .align_items(Align::Center)
            .push(Text::new(fl!("select-language-label")))
            .push(pick_list)
            .into()
    }
}
