// show logs when debugging
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use iced::widget::{
    button, column, horizontal_space, /* vertical_rule */
    /* Text, */ progress_bar, /* slider, */
    radio, row, text, Container,
};
use iced::window::icon::from_rgba;
use iced::window::{Icon, Settings as Window};
use iced::{executor, theme, Color, Padding};
use iced::{Alignment, Application, Command, Element, Length, Settings, Subscription, Theme};
use iced_aw::{split, Split};
use image::{self, GenericImageView};
use rfd::AsyncFileDialog;
use tracing::debug;
use rtxflash::{self, get_devices};

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

    return App::run(settings);
}

struct App {
    // The counter value
    value: i32,
    progress: f32,
    ver_divider_position: Option<u16>,
    selection: Option<RadioHW>,
    flashing: bool,
}

#[derive(Debug, Clone)]
pub enum Message {
    IncrementPressed,
    DecrementPressed,
    OnVerResize(u16),
    SliderChanged(f32),
    LanguageSelected(RadioHW),
    Tick,
    FlashPressed,
    OpenFWPressed,
    OpenFile(Option<String>),
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
            text: Color::BLACK,
            //primary: Color::from_rgb(0.8, 0.8, 0.8),
            primary: Color::from_rgb(0.98, 0.70, 0.07),
            success: Color::from_rgb(0.0, 1.0, 0.0),
            danger: Color::from_rgb(1.0, 0.0, 0.0),
        })
    }

    fn new(_flags: ()) -> (App, Command<Self::Message>) {
        (
            Self {
                value: 0,
                progress: 0.0,
                ver_divider_position: Some(150),
                selection: None,
                flashing: false,
            },
            Command::none(),
        )
    }

    fn title(&self) -> String {
        String::from("OpenRTX Companion")
    }

    fn update(&mut self, message: Message) -> Command<Self::Message> {
        match message {
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
                if self.flashing {
                    self.progress += 5.0;
                    debug!("update progress...");
                    if self.progress > 100.0 {
                        self.flashing = false;
                    }
                }
            }
            Message::FlashPressed => {
                self.progress = 0.0;
                self.flashing = true;
                debug!("flash");
            }
            Message::OpenFWPressed => {
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
            Message::OpenFile(f) => {
                debug!(f);
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
            .spacing(10),
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .center_x()
        .center_y();

        let second = Container::new(
            //     column![
            //      button("Increment").on_press(Message::IncrementPressed),
            //      text(self.value).size(50),
            //      button("Decrement").on_press(Message::DecrementPressed),
            //      progress_bar(0.0..=100.0, self.progress),
            //      slider(0.0..=100.0, self.progress, Message::SliderChanged).step(0.01)
            //  ])
            column![
                row![
                    column![text("Version:").size(15),]
                        .padding(Padding::from([0, 10, 0, 0]))
                        .width(80),
                    column![text("0.3.0").size(15),]
                ],
                row![
                    column![text("Device:").size(15),]
                        .padding(Padding::from([0, 10, 0, 0]))
                        .width(80),
                    column![text("/dev/ttyACM0, Fast, not slow, 16MB flash, etc ").size(15),]
                ],
                row![
                    column![text("Notes:").size(15),]
                        .padding(Padding::from([0, 10, 0, 0]))
                        .width(80),
                    column![text("What it is").size(15),]
                ],
                column![
                    progress_bar(0.0..=100.0, self.progress),
                    row![
                        button("Open FW File").on_press(Message::OpenFWPressed),
                        horizontal_space(10),
                        button("Flash Radio").on_press(Message::FlashPressed),
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

        // row![
        //     column![
        //         button("Increment").on_press(Message::IncrementPressed),
        //         text(self.value).size(50),
        //         button("Decrement").on_press(Message::DecrementPressed)
        //     ],
        //     vertical_rule(1),
        //     column![
        //         button("Increment").on_press(Message::IncrementPressed),
        //         text("This is a test").size(15),
        //         button("Decrement").on_press(Message::DecrementPressed)
        //     ]
        // ]
        // .padding(20)
        // .align_items(Alignment::Center)
        // .into()
    }

    fn subscription(&self) -> Subscription<Message> {
        iced::time::every(std::time::Duration::from_millis(500)).map(|_| Message::Tick)
    }
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RadioHW {
    Mduv3x0,
    Twrplus,
    Md3x0,
    Other,
}

impl RadioHW {
    fn all() -> [RadioHW; 4] {
        [
            RadioHW::Md3x0,
            RadioHW::Mduv3x0,
            RadioHW::Twrplus,
            RadioHW::Other,
        ]
    }
}

impl From<RadioHW> for String {
    fn from(language: RadioHW) -> String {
        String::from(match language {
            RadioHW::Mduv3x0 => "MD-UV390",
            RadioHW::Twrplus => "TWR-T Plus",
            RadioHW::Md3x0 => "MD380",
            RadioHW::Other => "Other",
        })
    }
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
