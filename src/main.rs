// SPDX-License-Identifier: MIT

use anyhow::Result;
use clap::{Parser, ValueEnum};
use hidreport::hid::*;
use serde::Serialize;
use std::io::Write;
use std::path::PathBuf;
use std::process::ExitCode;

#[derive(Clone, Debug, ValueEnum)]
enum Format {
    JsonV1,
}

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Cli {
    /// Print debugging information
    #[arg(short, long, default_value_t = false)]
    debug: bool,

    #[arg(long, default_value_t = false)]
    pretty: bool,

    /// Don't include the data values for the descriptor
    /// and items in the JSON output. This option
    /// is primarily used for debugging to make it easier
    /// to read the output.
    ///
    /// Implies --pretty
    #[arg(long, default_value_t = false)]
    skip_data: bool,

    #[arg(long, default_value_t = ("-").to_string())]
    output_file: String,

    #[arg(long, value_enum, default_value_t = Format::JsonV1)]
    format: Format,

    /// Path to a hid report descriptor file
    path: PathBuf,
}

// Warning: These enum value names are JSON API
#[derive(Serialize)]
enum JsonItemType {
    Global,
    Main,
    Local,
    Unknown,
}

// Warning: These enum value names are JSON API
#[derive(Serialize)]
enum JsonItemName {
    Unknown,
    Input,
    Output,
    Feature,
    Collection,
    EndCollection,
    UsagePage,
    LogicalMinimum,
    LogicalMaximum,
    PhysicalMinimum,
    PhysicalMaximum,
    UnitExponent,
    Unit,
    ReportSize,
    ReportId,
    ReportCount,
    Push,
    Pop,
    Reserved,
    Usage,
    UsageMinimum,
    UsageMaximum,
    DesignatorIndex,
    DesignatorMinimum,
    DesignatorMaximum,
    StringIndex,
    StringMinimum,
    StringMaximum,
    Delimiter,
}

impl<T> From<&T> for JsonItemName
    where T: Item,
{
    fn from(item: &T) -> JsonItemName {
        match item.item_type() {
            ItemType::Main(mi) => {
                match mi {
                    MainItem::Input(_) => JsonItemName::Input,
                    MainItem::Output(_) => JsonItemName::Output,
                    MainItem::Feature(_) => JsonItemName::Feature,
                    MainItem::Collection(_) => JsonItemName::Collection,
                    MainItem::EndCollection => JsonItemName::EndCollection,
                }
            }, 
            ItemType::Global(gi) => {
                match gi {
                    GlobalItem::UsagePage { .. } => JsonItemName::UsagePage,
                    GlobalItem::LogicalMinimum { .. } => JsonItemName::LogicalMinimum,
                    GlobalItem::LogicalMaximum { .. } => JsonItemName::LogicalMaximum,
                    GlobalItem::PhysicalMinimum { .. } => JsonItemName::PhysicalMinimum,
                    GlobalItem::PhysicalMaximum { .. } => JsonItemName::PhysicalMaximum,
                    GlobalItem::UnitExponent { .. } => JsonItemName::UnitExponent,
                    GlobalItem::Unit { .. } => JsonItemName::Unit,
                    GlobalItem::ReportSize { .. } => JsonItemName::ReportSize,
                    GlobalItem::ReportId { .. } => JsonItemName::ReportId,
                    GlobalItem::ReportCount { .. } => JsonItemName::ReportCount,
                    GlobalItem::Push => JsonItemName::Push,
                    GlobalItem::Pop => JsonItemName::Pop,
                    GlobalItem::Reserved => JsonItemName::Reserved,
                }
            },
            ItemType::Local(li) => {
                match li {
                    LocalItem::Usage { .. } => JsonItemName::Usage,
                    LocalItem::UsageMinimum { .. } => JsonItemName::UsageMinimum,
                    LocalItem::UsageMaximum { .. } => JsonItemName::UsageMaximum,
                    LocalItem::DesignatorIndex { .. } => JsonItemName::DesignatorIndex,
                    LocalItem::DesignatorMinimum { .. } => JsonItemName::DesignatorMinimum,
                    LocalItem::DesignatorMaximum { .. } => JsonItemName::DesignatorMaximum,
                    LocalItem::StringIndex { .. } => JsonItemName::StringIndex,
                    LocalItem::StringMinimum { .. } => JsonItemName::StringMinimum,
                    LocalItem::StringMaximum { .. } => JsonItemName::StringMaximum,
                    LocalItem::Delimiter { .. } => JsonItemName::Delimiter,
                    LocalItem::Reserved { .. } => JsonItemName::Reserved,
                }
            },
            _ => JsonItemName::Unknown,
        }
    }
}

#[derive(Serialize)]
struct JsonDescriptor {
    length: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    data: Option<Vec<u8>>,
}

#[derive(Serialize)]
struct JsonItem {
    offset: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    data: Option<Vec<u8>>,
    item_type: JsonItemType,
    item_name: JsonItemName,
    #[serde(skip_serializing_if = "Option::is_none")]
    value: Option<i32>,
}

#[derive(Serialize)]
struct JsonDecode<'a> {
    version: &'a str,
    descriptor: JsonDescriptor,
    items: Vec<JsonItem>,
}

fn hid_decode() -> Result<()> {
    let cli = Cli::parse();

    let stream: Box<dyn Write> = if cli.output_file == "-" {
        Box::new(std::io::stdout())
    } else {
        Box::new(std::fs::File::create(cli.output_file).unwrap())
    };

    let bytes = std::fs::read(&cli.path)?;

    let version = "1.0";
    let descriptor = JsonDescriptor {
        length: bytes.len(),
        data: if cli.skip_data {
            None
        } else {
            Some(bytes.to_vec())
        },
    };

    let rdesc_items = ReportDescriptorItems::try_from(bytes.as_slice())?;
    let items = rdesc_items
        .iter()
        .map(|rdesc_item| {
            let item = rdesc_item.item();
            let offset = rdesc_item.offset();
            let item_type = match item.item_type() {
                ItemType::Main(_) => JsonItemType::Main,
                ItemType::Global(_) => JsonItemType::Global,
                ItemType::Local(_) => JsonItemType::Local,    
                _ => JsonItemType::Unknown,
            };
            let item_name = JsonItemName::from(item);
            let value = match item.data() {
                None => None,
                Some(data) => Some(u32::try_from(&data).unwrap() as i32),
            };

            JsonItem {
                offset,
                data: if cli.skip_data {
                    None
                } else {
                    Some(item.bytes().to_owned().to_vec())
                },
                item_type,
                item_name,
                value,
            }
        })
        .collect::<Vec<JsonItem>>();

    let decode = JsonDecode {
        version,
        descriptor,
        items,
    };
    if cli.skip_data || cli.pretty {
        serde_json::to_writer_pretty(stream, &decode)?;
    } else {
        serde_json::to_writer(stream, &decode)?;
    }

    Ok(())
}

fn main() -> ExitCode {
    let rc = hid_decode();
    match rc {
        Ok(_) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("Error: {e:#}");
            ExitCode::FAILURE
        }
    }
}
