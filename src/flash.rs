// show logs when debugging
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use iced::widget::{
    button, column, horizontal_space, /* vertical_rule */
    /* Text, */ progress_bar, /* slider, */
    radio, row, text, Container,
    combo_box,
};
use iced::window::icon::from_rgba;
use iced::window::{Icon, Settings as Window};
use iced::{executor, theme, Color, Padding};
use iced::{Alignment, Application, Command, Element, Length, Settings, Subscription, Theme};
use iced_aw::{split, Split, TabLabel, Tabs};
use image::{self, GenericImageView};
use rfd::AsyncFileDialog;
use tracing::debug;
use rtxflash::{self, get_devices};
// use rtxlink::{flow, link};
use serial_enumerator::get_serial_list;
use std::sync::mpsc::{channel, Receiver};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RadioHW {
    Mduv3x0,
    Twrplus,
    Md3x0,
}

impl RadioHW {
    fn all() -> [RadioHW; 3] {
        [
            RadioHW::Md3x0,
            RadioHW::Mduv3x0,
            RadioHW::Twrplus,
        ]
    }
}

impl From<RadioHW> for String {
    fn from(radio: RadioHW) -> String {
        String::from(match radio {
            RadioHW::Md3x0 => "MD3x0",
            RadioHW::Mduv3x0 => "MD-UV3x0",
            RadioHW::Twrplus => "T-TWR Plus",
        })
    }
}

pub struct FlashTab {
    devices: DeviceInfo,
    progress: f32,
    flash_in_progress: bool,
}

impl FlashTab {
    pub fn new() -> Self {
        FlashTab {
            progress: 0.0,
            flash_in_progress: false,
        }
    }

    pub fn update(&mut self, message: LoginMessage) {
        match message {
            FlashMessage::FlashPressed => {
                self.progress = 0.0;
                self.flash_in_progress = true;
                self.backup_in_progress = false;
                debug!("flash");
                // TODO: flash_in_progress
                println!("Flashing OpenRTX firmware");
                if let Err(err) = ffi::flash_radio() {
                    eprintln!("Error: {}", err);
                    // process::exit(1);
                }
                println!("Firmware flash completed");
                println!("Please reboot the radio");
            }
            FlashMessage::OpenFWPressed => {
                return Command::perform(
                    async {
                        let file = AsyncFileDialog::new().pick_file().await;
                        if let Some(file) = file {
                            Some(format!(
                                "file:///{}",
                                file.path().to_str().unwrap().to_string()
                            ))
                        } else {
                            None
                        }
                    },
                    move |f| Message::OpenFile(f),
                );
            }
            FlashMessage::OpenFile(file) => {
                debug!(file);
            }
        }
    }
}

impl Tab for FlashTab {
    type Message = Message;

    fn title(&self) -> String {
        String::from("Flash")
    }

    fn tab_label(&self) -> TabLabel {
        //TabLabel::Text(self.title())
        TabLabel::IconText(Icon::User.into(), self.title())
    }

    fn content(&self) -> Element<'_, Self::Message> {
        let content: Element<'_, LoginMessage> = Container::new(
            Column::new()
                .align_items(Alignment::Center)
                .max_width(600)
                .padding(20)
                .spacing(16)
                .push(
                    TextInput::new("Username", &self.username)
                        .on_input(LoginMessage::UsernameChanged)
                        .padding(10)
                        .size(32),
                )
                .push(
                    TextInput::new("Password", &self.password)
                        .on_input(LoginMessage::PasswordChanged)
                        .padding(10)
                        .size(32)
                        .secure(true),
                )
                .push(
                    Row::new()
                        .spacing(10)
                        .push(
                            Button::new(
                                Text::new("Clear").horizontal_alignment(Horizontal::Center),
                            )
                            .width(Length::Fill)
                            .on_press(LoginMessage::ClearPressed),
                        )
                        .push(
                            Button::new(
                                Text::new("Login").horizontal_alignment(Horizontal::Center),
                            )
                            .width(Length::Fill)
                            .on_press(LoginMessage::LoginPressed),
                        ),
                ),
        )
        .align_x(Horizontal::Center)
        .align_y(Vertical::Center)
        .into();

        content.map(Message::Login)
    }
}
