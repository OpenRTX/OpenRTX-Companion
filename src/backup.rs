// show logs when debugging
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use crate::Message;
use crate::Tab;
use iced::{
    alignment::{Horizontal, Vertical},
    widget::{combo_box, progress_bar, row, text, Button, Column, Container, Row, Text},
    Alignment, Element, Length, Task,
};
use iced_aw::TabLabel;
use rfd::AsyncFileDialog;
use serial_enumerator::get_serial_list;
use std::sync::mpsc::{channel, Receiver};

// Wrapper type for SerialItem to enable trait definition
#[derive(Clone)]
pub struct SerialPort {
    name: String,
    vendor: String,
    product: String,
}

// Display trait for SeriatPortInfo
impl std::fmt::Display for SerialPort {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}

// Debug trait for SerialPortInfo
impl std::fmt::Debug for SerialPort {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SerialPort")
            .field("name", &self.name)
            .field("product", &self.product)
            .field("vendor", &self.vendor)
            .finish()
    }
}

// Unwrap result from serialport library
fn get_ports() -> Vec<SerialPort> {
    let ports = get_serial_list();
    ports
        .iter()
        .map(|p| SerialPort {
            name: p.name.clone(),
            vendor: p.vendor.clone().unwrap_or(String::from("")),
            product: p.product.clone().unwrap_or(String::from("")),
        })
        .collect()
}

#[derive(Clone, Debug)]
pub enum BackupMessage {
    BackupPressed,
    RestorePressed,
    OpenRestoreFilePressed,
    RestoreFileSelected(Option<String>),
    StartBackup(Option<String>),
    PortSelected(SerialPort),
    FilePath(Option<String>),
    Tick,
}

pub struct BackupTab {
    backup_in_progress: bool,
    backup_progress: Option<Receiver<(usize, usize)>>,
    serial_ports: Vec<SerialPort>,
    serial_port: Option<SerialPort>,
    ports_combo_state: combo_box::State<SerialPort>,
    progress: f32,
    restore_file: Option<String>,
    status_text: String,
}

impl Default for BackupTab {
    fn default() -> Self {
        let mut ports = get_ports();
        // Workaround: Iced crashes when rendering empty combo box
        if ports.len() == 0 {
            ports.push(SerialPort {
                name: String::from("No serial port found!"),
                vendor: String::from(""),
                product: String::from(""),
            });
        }
        Self {
            progress: 0.0,
            backup_in_progress: false,
            backup_progress: None,
            serial_ports: ports.clone(),
            serial_port: None,
            ports_combo_state: combo_box::State::new(ports),
            restore_file: None,
            status_text: String::from("Select an action"),
        }
    }
}

impl BackupTab {
    pub fn update(&mut self, message: BackupMessage) -> Task<Message> {
        match message {
            BackupMessage::BackupPressed => {
                self.progress = 0.0;
                self.backup_in_progress = true;
                Task::perform(
                    async {
                        let file = AsyncFileDialog::new().pick_folder().await;
                        if let Some(file) = file {
                            Some(format!(
                                "file:///{}",
                                file.path().to_str().unwrap().to_string()
                            ))
                        } else {
                            None
                        }
                    },
                    move |f| Message::StartBackup(f),
                )
            }
            BackupMessage::RestorePressed => Task::none(),
            BackupMessage::OpenRestoreFilePressed => Task::perform(
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
                move |f| Message::FilePath(f),
            ),
            BackupMessage::RestoreFileSelected(restore_file) => {
                self.restore_file = restore_file;
                Task::none()
            }
            BackupMessage::StartBackup(path) => {
                // Open link with configured serial port
                let port = match &self.serial_port {
                    Some(p) => p.name.clone(),
                    None => panic!("No serial port selected!"),
                };
                let (progress_tx, progress_rx) = channel();
                self.backup_progress = Some(progress_rx);
                std::thread::spawn(move || {
                    rtxlink::link::Link::new(&port);
                    rtxlink::flow::backup(path, Some(&progress_tx));
                });
                Task::none()
            }
            BackupMessage::PortSelected(port) => {
                self.serial_port = Some(port);
                Task::none()
            }
            BackupMessage::Tick => {
                if self.backup_in_progress {
                    if self.backup_progress.is_some() {
                        let (transferred_bytes, total_bytes) =
                            match self.backup_progress.as_ref().unwrap().try_iter().last() {
                                Some(x) => x,
                                None => {
                                    self.status_text = String::from("Backup complete!");
                                    (100, 100)
                                }
                            };
                        self.progress = transferred_bytes as f32 / total_bytes as f32 * 100.0;
                        if transferred_bytes < total_bytes {
                            self.status_text =
                                String::from(format!("{transferred_bytes}/{total_bytes}"));
                        }
                    }
                };
                Task::none()
            }
            _ => Task::none(),
        }
    }
}

impl Tab for BackupTab {
    type Message = Message;

    fn title(&self) -> String {
        String::from("Backup")
    }

    fn tab_label(&self) -> TabLabel {
        TabLabel::Text(self.title())
        // TabLabel::IconText(Icon::CogAlt.into(), self.title())
    }

    fn content(&self) -> Element<'_, Self::Message> {
        let port_combo_box = combo_box(
            &self.ports_combo_state,
            "Select a serial port",
            self.serial_port.as_ref(),
            BackupMessage::PortSelected,
        )
        // .on_option_hovered(Message::OptionHovered)
        // .on_close(Message::Closed)
        .width(250);

        let content: Element<'_, BackupMessage> = Container::new(
            Column::new()
                .max_width(600)
                .push(
                    row![
                        Column::new().width(120).push(text("Serial port:").size(15)),
                        port_combo_box,
                    ]
                    .padding(20),
                )
                .push(row![Column::new()
                    .width(600)
                    .align_x(Alignment::Center)
                    .push(text(&self.status_text).size(20)),])
                .push(row![progress_bar(0.0..=100.0, self.progress),].padding(20))
                .push(
                    Row::new()
                        .spacing(20)
                        .push(
                            Button::new(Text::new("Backup").align_x(Horizontal::Center))
                                .width(Length::Fill)
                                .on_press(BackupMessage::BackupPressed),
                        )
                        .push(
                            Button::new(Text::new("Restore").align_x(Horizontal::Center))
                                .width(Length::Fill)
                                .on_press(BackupMessage::RestorePressed),
                        ),
                ),
        )
        .align_x(Horizontal::Center)
        .align_y(Vertical::Center)
        .into();

        content.map(Message::Backup)
    }
}
