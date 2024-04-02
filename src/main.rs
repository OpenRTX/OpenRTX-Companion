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
use serial_enumerator::get_serial_list;
use std::sync::mpsc::{channel, Receiver};

fn icon() -> Icon {
    let image = image::load_from_memory(include_bytes!("../res/img/logo/icon.png")).unwrap();
    let (w, h) = image.dimensions();

    from_rgba(image.as_bytes().to_vec(), w, h).unwrap()
}

pub fn main() -> iced::Result {
    win_attach_terminal();
    init_logging();

    let devices = get_devices();
    println!("{:?}", devices);

    let settings: Settings<()> = Settings {
        window: Window {
            size: (600, 300),
            resizable: true,
            decorations: true,
            icon: Some(icon()),
            min_size: Some((300, 300)),
            ..iced::window::Settings::default()
        },
        // try_opengles_first: true,
        antialiasing: true,
        default_text_size: 17.0,
        ..iced::Settings::default()
    };

    App::run(settings)
}

// GUI Tabs
#[derive(Clone, PartialEq, Eq, Debug)]
enum TabId {
    Flash,
    Backup,
    Files,
}

struct App {
    // The counter value
    active_tab: TabId,
    value: i32,
    progress: f32,
    ver_divider_position: Option<u16>,
    selection: Option<RadioHW>,
    backup_in_progress: bool,
    backup_progress: Option<Receiver<(usize, usize)>>,
    serial_ports: Vec<SerialPort>,
    serial_port: Option<SerialPort>,
    ports_combo_state: combo_box::State<SerialPort>,
    status_text: String,
}

#[derive(Debug, Clone)]
pub enum Message {
    TabSelected(TabId),
    IncrementPressed,
    DecrementPressed,
    OnVerResize(u16),
    SliderChanged(f32),
    LanguageSelected(RadioHW),
    Tick,
    FlashPressed,
    OpenFWPressed,
    OpenFile(Option<String>),
    BackupPressed,
    StartBackup(Option<String>),
    PortSelected(SerialPort),
}

impl Application for App {
    type Executor = executor::Default;
    type Flags = ();
    type Message = Message;
    type Theme = Theme;

    fn theme(&self) -> Self::Theme {
        //Theme::Dark
        Theme::custom(theme::Palette {
            //background: Color::from_rgb(0.4, 0.4, 0.4),
            background: Color::from_rgb(0.1, 0.1, 0.1),
            text: Color::from_rgb(0.8, 0.8, 0.8),
            //primary: Color::from_rgb(0.8, 0.8, 0.8),
            primary: Color::from_rgb(0.98, 0.70, 0.07),
            success: Color::from_rgb(0.0, 1.0, 0.0),
            danger: Color::from_rgb(1.0, 0.0, 0.0),
        })
    }

    fn new(_flags: ()) -> (App, Command<Self::Message>) {
        let ports = get_ports();
        (
            Self {
                active_tab: TabId::Flash,
                value: 0,
                progress: 0.0,
                ver_divider_position: Some(150),
                selection: None,
                flash_in_progress: false,
                backup_in_progress: false,
                backup_progress: None,
                serial_ports: ports.clone(),
                serial_port: None,
                ports_combo_state: combo_box::State::new(ports),
                status_text: String::from("Select an action"),
            },
            Command::none(),
        )
    }

    fn title(&self) -> String {
        String::from("OpenRTX Companion")
    }

    fn update(&mut self, message: Message) -> Command<Self::Message> {
        match message {
            Message::TabSelected(selected) => { self.active_tab = selected },
            Message::IncrementPressed => {
                self.value += 1;
                debug!("Inc");
            }
            Message::DecrementPressed => {
                self.value -= 1;
                debug!("Dec");
            }
            Message::OnVerResize(_position) => self.ver_divider_position = Some(150), //Some(position),
            Message::SliderChanged(x) => self.progress = x,
            Message::LanguageSelected(radio_hw) => {
                self.selection = Some(radio_hw);
            }
            Message::Tick => {
                if self.flash_in_progress {
                    self.progress += 5.0;
                    debug!("update progress...");
                    if self.progress > 100.0 {
                        self.flash_in_progress = false;
                    }
                }
                if self.backup_in_progress {
                    if self.backup_progress.is_some() {
                        let (transferred_bytes, total_bytes) = match self.backup_progress.as_ref().unwrap().try_iter().last() {
                            Some(x) => x,
                            None => { self.status_text = String::from("Backup complete!"); (100, 100) },
                        };
                        self.progress = transferred_bytes as f32 / total_bytes as f32 * 100.0;
                        if transferred_bytes < total_bytes {
                            self.status_text = String::from(format!("{transferred_bytes}/{total_bytes}"));
                        }
                    }
                }
            }
            // Backup functionality needs a folder where to store backups
            Message::BackupPressed => {
                self.backup_in_progress = true;
                self.flash_in_progress = false;
                return Command::perform(
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
                );
            }
            Message::StartBackup(path) => {
                // Open link with configured serial port
                let port = match &self.serial_port {
                    Some(p) => p.name.clone(),
                    None => panic!("No serial port selected!"),
                };
                let (progress_tx, progress_rx) = channel();
                self.backup_progress = Some(progress_rx);
                std::thread::spawn(move || {
                    rtxlink::link::Link::new(&port).expect("Error in opening serial port!");
                    rtxlink::flow::backup(path, Some(&progress_tx));
                });
            }
            Message::PortSelected(port) => {
                self.serial_port = Some(port);
            }
        }
        Command::none()
    }

    fn view(&self) -> Element<Message> {
        let first = Container::new(
            //Text::new("Left")
            column(
                RadioHW::all()
                    .iter()
                    .cloned()
                    .map(|language| {
                        radio(
                            language,
                            language,
                            self.selection,
                            Message::LanguageSelected,
                        )
                    })
                    .map(Element::from)
                    .collect(),
            )
            .spacing(10)
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .center_x()
        .center_y();

        Tabs::new(Message::TabSelected)
            .tab_icon_position(iced_aw::tabs::Position::Bottom)
            .on_close(Message::TabClosed)
            .push(
                TabId::Flash,
                state.flash_tab.tab_label(),
                state.flash_tab.view(),
            )
            .push(
                TabId::Backup,
                state.backup_tab.tab_label(),
                state.backup_tab.view(),
            )
            .push(
                TabId::Files,
                state.files_tab.tab_label(),
                state.files_tab.view(),
            )
            .set_active_tab(&state.active_tab)
            .tab_bar_style(theme.clone())
            .icon_font(ICON)
            .tab_bar_position(match position {
                TabBarPosition::Top => iced_aw::TabBarPosition::Top,
                TabBarPosition::Bottom => iced_aw::TabBarPosition::Bottom,
            })
            .into();

        let port_combo_box = combo_box(
            &self.ports_combo_state,
            "Select a serial port",
            self.serial_port.as_ref(),
            Message::PortSelected,
        )
        // .on_option_hovered(Message::OptionHovered)
        // .on_close(Message::Closed)
        .width(250);

        let second = Container::new(
            column![
                row![
                    column![text("Version:").size(15),]
                        .padding(Padding::from([0, 10, 0, 0]))
                        .width(80),
                    column![text("0.3.0").size(15),]
                ],
                row![
                    column![text("Serial port:").size(15),]
                        .padding(Padding::from([0, 10, 0, 0]))
                        .width(120),
                    port_combo_box,
                ].padding(30),
                row![
                    column![text(&self.status_text).size(20),]
                        .width(500)
                        .align_items(Alignment::Center)
                ],
                column![
                    progress_bar(0.0..=100.0, self.progress),
                    row![
                        button("1. Open FW File").on_press(Message::OpenFWPressed),
                        horizontal_space(10),
                        button("2. Flash Radio").on_press(Message::FlashPressed),
                        horizontal_space(10),
                        button("3. Backup").on_press(Message::BackupPressed),
                    ]
                    .padding(15)
                ]
                .padding(Padding::from([40, 0, 0, 0]))
                .align_items(Alignment::Center)
            ]
            .padding(20)
            .align_items(Alignment::Start),
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .center_x()
        .center_y();

        Split::new(
            first,
            second,
            self.ver_divider_position,
            split::Axis::Vertical,
            Message::OnVerResize,
        )
        .into()

    }

    fn subscription(&self) -> Subscription<Message> {
        iced::time::every(std::time::Duration::from_millis(500)).map(|_| Message::Tick)
    }
}

trait Tab {
    type Message;

    fn title(&self) -> String;

    fn tab_label(&self) -> TabLabel;

    fn view(&self) -> Element<'_, Self::Message> {
        let column = Column::new()
            .spacing(20)
            .push(Text::new(self.title()).size(HEADER_SIZE))
            .push(self.content())
            .align_items(iced::Alignment::Center);

        Container::new(column)
            .width(Length::Fill)
            .height(Length::Fill)
            .align_x(Horizontal::Center)
            .align_y(Vertical::Center)
            .padding(TAB_PADDING)
            .into()
    }

    fn content(&self) -> Element<'_, Self::Message>;
}

fn init_logging() {
    use tracing::subscriber::set_global_default;
    use tracing::Level;
    use tracing_subscriber::FmtSubscriber;

    // all spans/events with a level higher than TRACE (e.g, debug, info, warn, etc.)
    // will be written to stdout.
    set_global_default(
        FmtSubscriber::builder()
            .with_max_level(Level::TRACE)
            .finish(),
    )
    .expect("setting default subscriber failed");
}

/// ``WINDOWS ONLY``: Have the application write to the terminal even with
/// ``[windows_subsystem = "windows"]``
///
/// This allows logs to be displayed when launched from the terminal.
fn win_attach_terminal() {
    #[cfg(windows)]
    unsafe {
        use winapi::um::wincon::{AttachConsole, ATTACH_PARENT_PROCESS};
        let _ = AttachConsole(ATTACH_PARENT_PROCESS);
    }
}
