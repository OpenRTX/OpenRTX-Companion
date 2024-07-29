// show logs when debugging
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use iced::{
    alignment::{Horizontal, Vertical},
    widget::{Column, Container, text, Text, combo_box, row, Row, Button, progress_bar},
    Element, Font, Length, Command, Alignment, Padding,
};
use iced_aw::{TabLabel, Tabs};
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
    DeviceSelected(DeviceInfo),
    OpenFWPressed,
    OpenFile(Option<String>),
    FlashPressed,
    FilePath(Option<String>),
    Tick,
}

pub struct FlashTab {
    devices: Vec<DeviceInfo>,
    selected_model: Option<RadioHW>,
    selected_device: Option<DeviceInfo>,
    device_combo_state: combo_box::State<DeviceInfo>,
    firmware_path: Option<String>,
    flash_in_progress: bool,
    progress: f32,
    status_text: String,
}

async fn open_fw_file() -> Option<String> {
    let file = AsyncFileDialog::new().pick_file().await;
    if let Some(file) = file {
        Some(format!(
            "file:///{}",
            file.path().to_str().unwrap().to_string()
        ))
    } else {
        None
    }
}

impl FlashTab {
    pub fn new() -> Self {
        let mut devices = get_devices();
        // Workaround: Iced crashes when rendering empty combo box
        if devices.len() == 0 {
            devices.push(DeviceInfo {
                index: 0,
                manufacturer: String::from(""),
                model: String::from("!"),
                port: String::from("No radios found"),
            });
        }
        FlashTab {
            devices: devices.clone(),
            selected_model: None,
            selected_device: None,
            device_combo_state: combo_box::State::new(devices),
            firmware_path: None,
            flash_in_progress: false,
            progress: 0.0,
            status_text: String::from("Select an action"),
        }
    }

    pub fn update(&mut self, message: FlashMessage) -> Command<Message> {
        match message {
            FlashMessage::DeviceSelected(device) => {
                self.selected_device = Some(device);
                Command::none()
            }
            FlashMessage::OpenFWPressed => {
                Command::perform(
                    open_fw_file(),
                    move |f| Message::FilePath(f)
                )
            }
            FlashMessage::FlashPressed => {
                self.progress = 1.0;
                self.flash_in_progress = true;
                self.status_text = String::from("Flashing firmware...");
                // rtxflash expects base path, not URI
                let file_uri = self.firmware_path.as_mut().unwrap();
                let bare_path = file_uri.strip_prefix("file:///");
                flash_device(self.selected_device.as_mut().unwrap(), bare_path.unwrap());
                self.progress = 100.0;
                self.status_text = String::from("Flashing firmware...done! Reboot the radio.");
                Command::none()
            }
            FlashMessage::FilePath(path) => {
                self.firmware_path = path.clone();
                match path {
                    Some(p) => { self.status_text = format!("Loaded firmware: {p}"); },
                    None => { self.status_text = String::from("Error in reading firmware!") }
                };
                Command::none()
            }
            FlashMessage::Tick => Command::none(),
            _ => Command::none()
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
        TabLabel::IconText(Icon::CogAlt.into(), self.title())
    }

    fn content(&self) -> Element<'_, Self::Message> {
        let device_combo_box = combo_box(
            &self.device_combo_state,
            "Select a device to flash",
            self.selected_device.as_ref(),
            FlashMessage::DeviceSelected,
        )
        // .on_option_hovered(Message::OptionHovered)
        // .on_close(Message::Closed)
        .width(250);

        let content: Element<'_, FlashMessage> = Container::new(
            Column::new()
                .max_width(600)
                .push(
                    row![
                        Column::new()
                            .width(120)
                            .push(text("Device:").size(15)),
                        device_combo_box,
                    ].padding(20)
                )
                .push(
                    row![
                        Column::new()
                            .width(600)
                            .align_items(Alignment::Center)
                            .push(text(&self.status_text).size(20)),
                    ],
                )
                .push(
                    row![
                        progress_bar(0.0..=100.0, self.progress),
                    ].padding(20),
                )
                .push(
                    Row::new()
                        .spacing(20)
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
