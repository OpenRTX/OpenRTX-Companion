// show logs when debugging
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use iced::{
    alignment::{Horizontal, Vertical},
    widget::{combo_box, progress_bar, row, text, Button, Column, Container, Row, Text},
    Alignment, Element, Font, Length, Padding, Task,
};
use iced_aw::{TabLabel, Tabs};
use image::{self, GenericImageView};
use rfd::AsyncFileDialog;
use rtxflash::{flash, target};
use std::sync::mpsc::{channel, Receiver};
use tracing::debug;

use crate::{Icon, Message, Tab};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RadioHW {
    Mduv3x0,
    Twrplus,
    Md3x0,
}

impl RadioHW {
    fn all() -> [RadioHW; 3] {
        [RadioHW::Md3x0, RadioHW::Mduv3x0, RadioHW::Twrplus]
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
    DeviceSelected(rtxflash::target::DeviceInfo),
    TargetSelected(rtxflash::target::Target),
    OpenFWPressed,
    OpenFile(Option<String>),
    FlashPressed,
    FilePath(Option<String>),
    Tick,
}

pub struct FlashTab {
    devices: Vec<rtxflash::target::DeviceInfo>,
    targets: Vec<rtxflash::target::Target>,
    selected_model: Option<RadioHW>,
    selected_device: Option<rtxflash::target::DeviceInfo>,
    selected_target: Option<rtxflash::target::Target>,
    device_combo_state: combo_box::State<rtxflash::target::DeviceInfo>,
    target_combo_state: combo_box::State<rtxflash::target::Target>,
    firmware_path: Option<String>,
    flash_in_progress: bool,
    flash_progress: Option<Receiver<(usize, usize)>>,
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

impl Default for FlashTab {
    fn default() -> Self {
        let devices = target::get_devices();
        let mut targets = vec![] as Vec<target::Target>;
        for t in target::get_targets() {
            targets.push(t);
        }
        Self {
            devices: devices.clone(),
            targets: targets.clone(),
            selected_model: None,
            selected_device: None,
            selected_target: None,
            device_combo_state: combo_box::State::new(devices),
            target_combo_state: combo_box::State::new(targets),
            firmware_path: None,
            flash_in_progress: false,
            flash_progress: None,
            progress: 0.0,
            status_text: String::from("Select an action"),
        }
    }
}

impl FlashTab {
    pub fn update(&mut self, message: FlashMessage) -> Task<Message> {
        match message {
            FlashMessage::DeviceSelected(device) => {
                self.selected_device = Some(device);
                Task::none()
            }
            FlashMessage::TargetSelected(target) => {
                self.selected_target = Some(target);
                Task::none()
            }
            FlashMessage::OpenFWPressed => {
                Task::perform(open_fw_file(), move |f| Message::FilePath(f))
            }
            FlashMessage::FlashPressed => {
                self.progress = 1.0;
                self.flash_in_progress = true;
                self.status_text = String::from("Flashing firmware...");
                // rtxflash expects base path, not URI
                let file_uri = self.firmware_path.clone().unwrap();
                let bare_path = file_uri.strip_prefix("file:///").unwrap().to_string();
                let target = self.selected_target.clone().unwrap();
                let port = self.selected_device.clone().unwrap().port;

                // Start flash in a separate thread
                let (progress_tx, progress_rx) = channel();
                self.flash_progress = Some(progress_rx);
                std::thread::spawn(move || {
                    let _ = flash::flash(target, port, bare_path, Some(&progress_tx));
                });
                Task::none()
            }
            FlashMessage::FilePath(path) => {
                self.firmware_path = path.clone();
                match path {
                    Some(p) => {
                        self.status_text = format!("Loaded firmware: {p}");
                    }
                    None => self.status_text = String::from("Error in reading firmware!"),
                };
                Task::none()
            }
            FlashMessage::Tick => {
                if self.flash_in_progress {
                    if self.flash_progress.is_some() {
                        match self.flash_progress.as_ref().unwrap().try_iter().last() {
                            Some(x) => {
                                let (transferred_bytes, total_bytes) = x;
                                self.progress =
                                    transferred_bytes as f32 / total_bytes as f32 * 100.0;
                                self.status_text = String::from(format!(
                                    "Flashed chunk {transferred_bytes}/{total_bytes}"
                                ));
                            }
                            None => {
                                self.status_text = String::from("");
                                ()
                            }
                        };
                    }
                };
                Task::none()
            }
            _ => Task::none(),
        }
    }
}

impl Tab for FlashTab {
    type Message = Message;

    fn title(&self) -> String {
        String::from("Flash")
    }

    fn tab_label(&self) -> TabLabel {
        TabLabel::Text(self.title())
        // TabLabel::IconText(Icon::CogAlt.into(), self.title())
    }

    fn content(&self) -> Element<'_, Self::Message> {
        let device_combo_box = combo_box(
            &self.device_combo_state,
            "Select a device to flash",
            self.selected_device.as_ref(),
            FlashMessage::DeviceSelected,
        )
        .width(250);
        let target_combo_box = combo_box(
            &self.target_combo_state,
            "Select a target",
            self.selected_target.as_ref(),
            FlashMessage::TargetSelected,
        )
        // .on_option_hovered(Message::OptionHovered)
        // .on_close(Message::Closed)
        .width(250);

        let content: Element<'_, FlashMessage> = Container::new(
            Column::new()
                .max_width(600)
                .push(
                    row![
                        Column::new().width(120).push(text("Device:").size(15)),
                        device_combo_box,
                    ]
                    .padding(10),
                )
                .push(
                    row![
                        Column::new().width(120).push(text("Target:").size(15)),
                        target_combo_box,
                    ]
                    .padding(10),
                )
                .push(row![Column::new()
                    .width(600)
                    .align_x(Alignment::Center)
                    .push(
                        text(&self.status_text)
                            .wrapping(text::Wrapping::Word)
                            .size(20)
                    ),])
                .push(row![progress_bar(0.0..=100.0, self.progress),].padding(20))
                .push(
                    Row::new()
                        .spacing(20)
                        .push(
                            Button::new(Text::new("Select Firmware").align_x(Horizontal::Center))
                                .width(Length::Fill)
                                .on_press(FlashMessage::OpenFWPressed),
                        )
                        .push(
                            Button::new(Text::new("Flash").align_x(Horizontal::Center))
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
