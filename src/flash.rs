// show logs when debugging
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use iced::{
    alignment::{Horizontal, Vertical},
    Alignment, Application, Command, Element, Length,
    widget::{Container, Button, Row, Column, column, Text, TextInput, radio}
};
use iced_aw::TabLabel;
use image::{self, GenericImageView};
use rfd::AsyncFileDialog;
use tracing::debug;
use rtxflash::{self, get_devices, get_device_info, flash_device};
use rtxflash::radio_tool_ffi::DeviceInfo;

use crate::{Icon, Message, Tab};

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

#[derive(Clone, Debug)]
pub enum FlashMessage {
    ModelChanged(RadioHW),
    DeviceChanged(u64),
    OpenFWPressed,
    OpenFile(Option<String>),
    FlashPressed,
    Tick,
}

pub struct FlashTab {
    devices: Vec<DeviceInfo>,
    selected_model: Option<RadioHW>,
    selected_device: Option<u16>,
    firmware_path: Option<String>,
    flash_in_progress: bool,
    progress: f32,
}

impl FlashTab {
    pub fn new() -> Self {
        FlashTab {
            devices: get_devices(),
            selected_model: None,
            selected_device: None,
            firmware_path: None,
            progress: 0.0,
            flash_in_progress: false,
        }
    }

    pub fn update(&mut self, message: FlashMessage) {
        match message {
            FlashMessage::ModelChanged(model) => { }
            FlashMessage::DeviceChanged(model) => { }
            FlashMessage::OpenFWPressed => {
                Command::perform(
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
                    move |f| FlashMessage::OpenFile(f),
                );
                ()
            }
            FlashMessage::FlashPressed => {
                self.progress = 0.0;
                self.flash_in_progress = true;
                debug!("flash");
                // TODO: flash_in_progress
                println!("Flashing OpenRTX firmware");
                flash_device(self.selected_device.unwrap(), self.firmware_path.as_mut().unwrap().as_ref());
                println!("Firmware flash completed");
                println!("Please reboot the radio");
            }
            FlashMessage::OpenFile(file) => {
                debug!(file);
                self.firmware_path = file;
            }
            FlashMessage::Tick => {
                if self.flash_in_progress {
                    self.progress += 5.0;
                    debug!("update progress...");
                    if self.progress > 100.0 {
                        self.flash_in_progress = false;
                    }
                }
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
        let content: Element<'_, FlashMessage> = Container::new(
            Column::new()
                .align_items(Alignment::Center)
                .max_width(600)
                .padding(20)
                .spacing(16)
                // TODO: Replace these with radio buttons
                //.push(
                //    TextInput::new("Model", &self.selected_model)
                //        .on_input(FlashMessage::ModelChanged)
                //        .padding(10)
                //        .size(32),
                //)
                //.push(
                //    TextInput::new("Device", &self.selected_device)
                //        .on_input(FlashMessage::DeviceChanged)
                //        .padding(10)
                //        .size(32)
                //        .secure(true),
                //)
                .push(
                    Row::new()
                        .spacing(10)
                        .push(
                            Button::new(
                                Text::new("Select Firmware").horizontal_alignment(Horizontal::Center),
                            )
                            .width(Length::Fill)
                            .on_press(FlashMessage::OpenFWPressed),
                        )
                        .push(
                            Button::new(
                                Text::new("Flash").horizontal_alignment(Horizontal::Center),
                            )
                            .width(Length::Fill)
                            .on_press(FlashMessage::FlashPressed),
                        ),
                ),
        )
        .align_x(Horizontal::Center)
        .align_y(Vertical::Center)
        .into();

        content.map(Message::Flash)
    }
}
