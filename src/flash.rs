// show logs when debugging
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use iced::{
    alignment::{Horizontal, Vertical},
    Alignment, Command, Element, Length, Padding,
    widget::{Container, Button, Row, row, Column, column, Text, text,
             combo_box}
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
    DeviceSelected(DeviceInfo),
    OpenFWPressed,
    OpenFile(Option<String>),
    FlashPressed,
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
        let devices = get_devices();
        FlashTab {
            devices: devices.clone(),
            selected_model: None,
            selected_device: None,
            device_combo_state: combo_box::State::new(devices),
            firmware_path: None,
            progress: 0.0,
            flash_in_progress: false,
        }
    }

    pub fn update(&mut self, message: FlashMessage) -> Command<FlashMessage> {
        match message {
            FlashMessage::DeviceSelected(device) => {
                self.selected_device = Some(device);
            }
            FlashMessage::OpenFWPressed => {
                return Command::perform(
                    open_fw_file(),
                    move |f| FlashMessage::OpenFile(f),
                );
            }
            FlashMessage::OpenFile(file) => {
                debug!(file);
                self.firmware_path = file;
            }
            FlashMessage::FlashPressed => {
                self.progress = 0.0;
                self.flash_in_progress = true;
                debug!("flash");
                // TODO: flash_in_progress
                println!("Flashing OpenRTX firmware");
                flash_device(self.selected_device.as_mut().unwrap(), self.firmware_path.as_mut().unwrap().as_ref());
                println!("Firmware flash completed");
                println!("Please reboot the radio");
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
        Command::none()
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
                .align_items(Alignment::Center)
                .max_width(600)
                .push(
                    row![
                        column![text("Device:").size(15),]
                            .padding(Padding::from([0, 10, 0, 0]))
                            .width(120),
                        device_combo_box,
                    ].padding(30),
                )
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
