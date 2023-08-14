#![windows_subsystem = "windows"]
//#![deny(clippy::all)]
//#![deny(clippy::pedantic)]
//#![deny(clippy::nursery)]
//#![deny(clippy::cargo)]
//#![deny(missing_docs)]

use std::path::PathBuf;
use clap::Parser;
use rayon::prelude::{IntoParallelRefIterator, ParallelIterator};
use ab_versions::{get_version, is_protected, strip_protection};
use slint::{Color, ModelRc, VecModel};
use std::rc::Rc;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
    files: Vec<PathBuf>
}

slint::include_modules!();

fn main() -> Result<(), slint::PlatformError> {
    let args = Args::parse_from(wild::args());

    let ui = AppWindow::new()?;

    if !args.files.is_empty() {
        let info: Vec<file_info> = args.files.par_iter().map(|f| {
            file_info {
                locked: is_protected(f).unwrap(),
                file_name: format!("{:?}", f.file_name().unwrap()).into(),
                file_ver: get_version(f).unwrap().to_string().into(),
            }
        }).collect();
        let info = ModelRc::from(Rc::new(VecModel::from(info)));
        ui.set_files(info);
    }

    /*let ui_handle = ui.as_weak();
    ui.on_request_increase_value(move || {
        let ui = ui_handle.unwrap();
        ui.set_counter(ui.get_counter() + 1);
    });*/

    ui.run()
}

/*
let model: Rc<VecModel<JsonTheme>> = Rc::new(VecModel::from(themes));

        ModelRc::from(marg
         */
