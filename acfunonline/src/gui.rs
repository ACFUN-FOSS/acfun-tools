use crate::live::keep_online;
use iced::{
    widget::{button, text_input},
    Align, Application, Button, Column, Command, Container, Element, Length, Text, TextInput,
};

/// GUI数据
#[derive(Clone, Debug, Default)]
pub struct Online {
    account: String,
    password: String,
    status: Status,
    account_status: text_input::State,
    password_status: text_input::State,
    button_status: button::State,
}

/// 界面状态
#[derive(Clone, Debug)]
enum Status {
    WaitInput,
    Online,
    Error(String),
}

impl Default for Status {
    #[inline]
    fn default() -> Self {
        Self::WaitInput
    }
}

/// 消息
#[derive(Clone, Debug)]
pub enum Message {
    AccountInput(String),
    PasswordInput(String),
    Login,
    Error(String),
}

impl Application for Online {
    type Executor = iced::executor::Default;
    type Message = Message;
    type Flags = ();

    #[inline]
    fn new(_flags: Self::Flags) -> (Self, Command<Self::Message>) {
        (Self::default(), Command::none())
    }

    #[inline]
    fn title(&self) -> String {
        String::from("AcFun Live Online")
    }

    fn update(
        &mut self,
        message: Self::Message,
        _clipboard: &mut iced::Clipboard,
    ) -> Command<Self::Message> {
        match message {
            Message::AccountInput(s) => {
                self.account = s;
                Command::none()
            }
            Message::PasswordInput(s) => {
                self.password = s;
                Command::none()
            }
            Message::Login => {
                self.status = Status::Online;
                Command::perform(
                    keep_online(self.account.clone(), self.password.clone()),
                    |r| match r {
                        Ok(_) => Message::Error("stop keeping online".to_string()),
                        Err(e) => Message::Error(e.to_string()),
                    },
                )
            }
            Message::Error(s) => {
                self.status = Status::Error(s);
                Command::none()
            }
        }
    }

    fn view(&mut self) -> Element<'_, Self::Message> {
        match &self.status {
            Status::WaitInput => {
                let account_input = TextInput::new(
                    &mut self.account_status,
                    "AcFun account's phone number or email",
                    self.account.as_str(),
                    Message::AccountInput,
                )
                .padding(5)
                .width(Length::Units(300));
                let password_input = TextInput::new(
                    &mut self.password_status,
                    "password",
                    self.password.as_str(),
                    Message::PasswordInput,
                )
                .password()
                .padding(5)
                .width(Length::Units(300));
                let button = Button::new(&mut self.button_status, Text::new("login"))
                    .on_press(Message::Login);
                let column = Column::new()
                    .spacing(10)
                    .align_items(Align::Center)
                    .push(account_input)
                    .push(password_input)
                    .push(button);
                Container::new(column)
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .center_x()
                    .center_y()
                    .into()
            }
            Status::Online => Container::new(Text::new("keeping online..."))
                .width(Length::Fill)
                .height(Length::Fill)
                .center_x()
                .center_y()
                .into(),
            Status::Error(s) => Container::new(Text::new(s.as_str()))
                .width(Length::Fill)
                .height(Length::Fill)
                .center_x()
                .center_y()
                .into(),
        }
    }
}
