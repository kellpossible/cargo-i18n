use iced::{
    executor, widget::{Column, PickList, Text}, Alignment, Application, Command, Element, Length, Settings
};

use unic_langid::LanguageIdentifier;
mod localize;


struct AppState {
    available_languages: Vec<LanguageIdentifier>,
    selected_language: LanguageIdentifier,
}

#[derive(Debug, Clone)]
enum Message {
    SelectLanguage(LanguageIdentifier)
}

impl iced::Application for AppState {
    type Executor = executor::Default;
    type Message = Message;
    type Theme = iced::Theme;
    type Flags = ();

    fn new(_flags: Self::Flags) -> (Self, iced::Command<Self::Message>) {

        let requested_languages = localize::requested_languages();

        let app_state = Self {
            available_languages: localize::available_languages(),
            selected_language: localize::select_language(&requested_languages)
                .get(0)
                .unwrap()
                .clone(),
        };

        (app_state, Command::none())
    }

    fn title(&self) -> String {
        String::from(fl!("window-title"))
    }

    fn update(&mut self, message: Message) -> Command<Self::Message> {

        match message {
            Message::SelectLanguage(language) => {
                let requested_languages = vec![language];
                self.selected_language = localize::select_language(&requested_languages)
                    .get(0)
                    .unwrap().clone();
            }
        }

        Command::none()
    }

    fn view(&self) -> Element<Self::Message, Self::Theme, iced::Renderer> {

        let pick_list = PickList::new(
            self.available_languages.clone(),
            Some(self.selected_language.clone()),
            |language| Message::SelectLanguage(language),
        );
        
        Column::new()
            .padding(20)
            .width(Length::Fill)
            .height(Length::Fill)
            .align_items(Alignment::Center)
            .push(Text::new(fl!("select-language-label")))
            .push(pick_list)
            .into()
    }
}

pub fn main() -> iced::Result {
    AppState::run(Settings::default())
}
