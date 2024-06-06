// show logs when debugging
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use iced::{
    alignment::{self, Horizontal, Vertical},
    font,
    widget::{container, text, Column, Container, Text},
    Application, Command, Element, Font, Length, Settings, Theme,
    window::icon::from_rgba, executor, theme, Color,
    Subscription,
};
use iced_aw::{TabLabel, Tabs, TabBarPosition, TabBarStyles};
use image::{self, GenericImageView};

mod flash;
use flash::{FlashMessage, FlashTab};

mod backup;
use backup::{BackupMessage, BackupTab};

const HEADER_SIZE: u16 = 32;
const TAB_PADDING: u16 = 16;
const ICON_BYTES: &[u8] = include_bytes!("../fonts/icons.ttf");
const ICON: Font = Font::with_name("icons");

fn app_icon() -> iced::window::Icon {
    let image = image::load_from_memory(include_bytes!("../res/img/logo/icon.png")).unwrap();
    let (w, h) = image.dimensions();

    from_rgba(image.as_bytes().to_vec(), w, h).unwrap()
}

enum Icon {
    User,
    Heart,
    Calc,
    CogAlt,
}

impl From<Icon> for char {
    fn from(icon: Icon) -> Self {
        match icon {
            Icon::User => '\u{E800}',
            Icon::Heart => '\u{E801}',
            Icon::Calc => '\u{F1EC}',
            Icon::CogAlt => '\u{E802}',
        }
    }
}

pub fn main() -> iced::Result {
    win_attach_terminal();
    init_logging();

    let settings: Settings<()> = Settings {
        window: iced::window::Settings {
            size: iced::Size {
                width: 300.0,
                height: 600.0,
            },
            resizable: true,
            decorations: true,
            icon: Some(app_icon()),
            min_size: Some(iced::Size {
                width: 300.0,
                height: 300.0,
            }),
            ..iced::window::Settings::default()
        },
        // try_opengles_first: true,
        antialiasing: true,
        default_text_size: iced::Pixels(17.0),
        ..iced::Settings::default()
    };

    OpenRTXCompanion::run(settings)
}

struct State {
    active_tab: TabId,
    flash_tab: FlashTab,
    backup_tab: BackupTab,
}

// GUI Tabs
#[derive(Clone, PartialEq, Eq, Debug)]
enum TabId {
    Flash,
    Backup,
    Files,
}

enum OpenRTXCompanion {
    Loading,
    Loaded(State),
}

#[derive(Clone, Debug)]
enum Message {
    TabSelected(TabId),
    Flash(FlashMessage),
    Backup(BackupMessage),
    Tick,
    #[allow(dead_code)]
    Loaded(Result<(), String>),
    FontLoaded(Result<(), font::Error>),
    TabClosed(TabId),
}

async fn load() -> Result<(), String> {
    Ok(())
}

impl Application for OpenRTXCompanion {
    type Executor = executor::Default;
    type Flags = ();
    type Message = Message;
    type Theme = Theme;

    fn new(_flags: ()) -> (OpenRTXCompanion, Command<Message>) {
        (
            OpenRTXCompanion::Loading,
            Command::batch(vec![
                font::load(ICON_BYTES).map(Message::FontLoaded),
                font::load(iced_aw::BOOTSTRAP_FONT_BYTES).map(Message::FontLoaded),
                Command::perform(load(), Message::Loaded),
            ]),
        )
    }

    fn theme(&self) -> Self::Theme {
        //Theme::Dark
        Theme::custom(
            String::from("OpenRTX"),
            theme::Palette {
                //background: Color::from_rgb(0.4, 0.4, 0.4),
                background: Color::from_rgb(0.1, 0.1, 0.1),
                text: Color::from_rgb(0.8, 0.8, 0.8),
                //primary: Color::from_rgb(0.8, 0.8, 0.8),
                primary: Color::from_rgb(0.98, 0.70, 0.07),
                success: Color::from_rgb(0.0, 1.0, 0.0),
                danger: Color::from_rgb(1.0, 0.0, 0.0),
        })
    }

    fn title(&self) -> String {
        String::from("OpenRTX Companion")
    }

    fn update(&mut self, message: Message) -> Command<Self::Message> {
        match self {
            OpenRTXCompanion::Loading => {
                if let Message::Loaded(_) = message {
                    *self = OpenRTXCompanion::Loaded(State {
                        active_tab: TabId::Flash,
                        flash_tab: FlashTab::new(),
                        backup_tab: BackupTab::new(),
                    })
                }
            }
            OpenRTXCompanion::Loaded(state) => match message {
                Message::TabSelected(selected) => state.active_tab = selected,
                Message::Flash(message) => state.flash_tab.update(message),
                Message::Backup(message) => state.backup_tab.update(message),
                Message::TabClosed(id) => println!("Tab {:?} event hit", id),
                _ => {}
            },
        }

        Command::none()
    }

    fn view(&self) -> Element<'_, Self::Message> {
        match self {
            OpenRTXCompanion::Loading => container(
                text("Loading...")
                    .horizontal_alignment(alignment::Horizontal::Center)
                    .size(50),
            )
            .width(Length::Fill)
            .height(Length::Fill)
            .center_y()
            .center_x()
            .into(),
            OpenRTXCompanion::Loaded(state) => {
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
                    .set_active_tab(&state.active_tab)
                    .tab_bar_style(TabBarStyles::default())
                    .icon_font(ICON)
                    .tab_bar_position(TabBarPosition::Top)
                    .into()
            }
        }
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
