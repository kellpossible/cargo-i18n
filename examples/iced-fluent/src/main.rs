
use iced::{executor, widget::{Column, Text}, Alignment, Application, Command, Element, Settings};

use unic_langid::LanguageIdentifier;

mod localize;

pub fn main() -> iced::Result {

    localize::localize();

    Example::run(Settings::default())
}

struct Example {}

#[derive(Debug, Clone)]
enum Message {}

impl iced::Application for Example {
    type Executor = executor::Default;
    type Message = Message;
    type Theme = iced::Theme;
    type Flags = ();

    fn new(_flags: Self::Flags) -> (Self, iced::Command<Self::Message>) {
        
        
         

         (Self {}, Command::none())
    }

    fn title(&self) -> String {
        String::from(fl!("window-title"))
    }

    fn update(&mut self, _message: Message) -> Command<Self::Message> {
        Command::none()
    }

    fn view(&self) -> Element<Self::Message, Self::Theme, iced::Renderer> {
        Column::new()
            .padding(20)
            .align_items(Alignment::Center)
            .push(Text::new(fl!("select-language-label")))
            .into()
    }
}
