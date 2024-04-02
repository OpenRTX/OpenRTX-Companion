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
use iced_aw::{split, Split};
use iced_aw::native::{TabLabel, Tabs};
use image::{self, GenericImageView};
use rfd::AsyncFileDialog;
use tracing::debug;
use rtxflash::{self, get_devices};
// use rtxlink::{flow, link};
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
    ports.iter().map(|p| {
        SerialPort{
            name: p.name.clone(),
            vendor: p.vendor.clone().unwrap_or(String::from("")),
            product: p.product.clone().unwrap_or(String::from("")),
        }
    }).collect()
}
