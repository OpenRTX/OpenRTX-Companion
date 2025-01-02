// show logs when debugging
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use iced::{
    alignment::{Horizontal, Vertical},
    font, theme,
    widget::{Column, Container, Text},
    window,
    window::icon::from_rgba,
    Color, Element, Font, Length, Settings, Subscription, Task, Theme,
};
use iced_aw::{TabBarPosition, TabLabel, Tabs};
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

fn main() -> iced::Result {
    win_attach_terminal();
    // init_logging();

    let mut window_settings = window::Settings::default();
    window_settings.size = iced::Size {
        width: 600.0,
        height: 400.0,
    };
    window_settings.resizable = true;
    window_settings.decorations = true;
    window_settings.icon = Some(app_icon());
    window_settings.min_size = Some(iced::Size {
        width: 300.0,
        height: 300.0,
    });
    let settings = Settings {
        id: Some(String::from("OpenRTX Companion")),
        default_text_size: iced::Pixels(17.0),
        antialiasing: true,
        ..Settings::default()
    };

    iced::application(
        "OpenRTX Companion",
        OpenRTXCompanion::update,
        OpenRTXCompanion::view,
    )
    .font(iced_fonts::REQUIRED_FONT_BYTES)
    .window(window_settings)
    .settings(settings)
    .theme(OpenRTXCompanion::theme)
    .subscription(OpenRTXCompanion::subscription)
    .run()
}

// GUI Tabs
#[derive(Clone, PartialEq, Eq, Debug, Default)]
enum TabId {
    #[default]
    Flash,
    Backup,
    // Files,
}

#[derive(Default)]
struct OpenRTXCompanion {
    active_tab: TabId,
    flash_tab: FlashTab,
    backup_tab: BackupTab,
}

#[derive(Clone, Debug)]
enum Message {
    TabSelected(TabId),
    Flash(FlashMessage),
    Backup(BackupMessage),
    // These two messages are the result of asynchronous actions and need
    // to be propagated to the respective tabs
    FilePath(Option<String>),
    StartBackup(Option<String>),
    Tick,
    #[allow(dead_code)]
    Loaded(Result<(), String>),
    FontLoaded(Result<(), font::Error>),
    TabClosed(TabId),
}

async fn load() -> Result<(), String> {
    Ok(())
}

impl OpenRTXCompanion {
    fn title(&self) -> String {
        String::from("OpenRTX Companion")
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::TabSelected(selected) => {
                self.active_tab = selected;
                Task::none()
            }
            Message::Flash(message) => self.flash_tab.update(message),
            Message::Backup(message) => self.backup_tab.update(message),
            Message::TabClosed(id) => {
                println!("Tab {:?} event hit", id);
                Task::none()
            }
            Message::FilePath(path) => match &self.active_tab {
                TabId::Flash => self.flash_tab.update(FlashMessage::FilePath(path)),
                TabId::Backup => self.backup_tab.update(BackupMessage::FilePath(path)),
            },
            Message::StartBackup(path) => self.backup_tab.update(BackupMessage::StartBackup(path)),
            Message::Tick => {
                _ = self.flash_tab.update(FlashMessage::Tick);
                _ = self.backup_tab.update(BackupMessage::Tick);
                Task::none()
            }
            _ => Task::none(),
        }
    }

    fn view(&self) -> Element<'_, Message> {
        Tabs::new(Message::TabSelected)
            .tab_icon_position(iced_aw::tabs::Position::Bottom)
            .push(
                TabId::Flash,
                self.flash_tab.tab_label(),
                self.flash_tab.view(),
            )
            .push(
                TabId::Backup,
                self.backup_tab.tab_label(),
                self.backup_tab.view(),
            )
            .set_active_tab(&self.active_tab)
            .icon_font(ICON)
            .tab_bar_position(TabBarPosition::Top)
            .into()
    }

    fn theme(&self) -> Theme {
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
            },
        )
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
            .align_x(iced::Alignment::Center)
            .push(self.content());

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
