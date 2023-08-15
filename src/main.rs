#![windows_subsystem = "windows"]
//#![deny(clippy::all)]
//#![deny(clippy::pedantic)]
//#![deny(clippy::nursery)]
//#![deny(clippy::cargo)]
//#![deny(missing_docs)]

use std::borrow::BorrowMut;
use std::path::PathBuf;
use clap::Parser;
use rayon::prelude::{IntoParallelRefIterator, ParallelIterator};
use ab_versions::{get_version, is_protected, strip_protection};
use slint::{ModelRc, VecModel};
use std::rc::Rc;
use std::collections::HashMap;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
    files: Vec<PathBuf>
}

slint::include_modules!();

fn main() -> Result<(), slint::PlatformError> {
    let args = Args::parse_from(wild::args());

    let ui = AppWindow::new()?;
    let files: HashMap<String, PathBuf> = args.files.iter().map(|pb| {
        let name = pb.file_name().map(|name| name.to_string_lossy()).unwrap().to_string();
        (name, pb.to_owned())
    }).collect();

    let mut file_model = Rc::new(VecModel::default());

    //if files were passed on the command line process them and add them to the UI model
    if !files.is_empty() {
        let info: Vec<file_info> = files.par_iter().map(|(name, file)| {
            file_info {
                locked: is_protected(&file).unwrap(),
                file_name: name.into(),
                file_ver: get_version(&file).unwrap().to_string().into(),
            }
        }).collect();
        file_model.borrow_mut().extend(info.into_iter());
    }

    ui.set_files(ModelRc::from(file_model));

    let unlock_handle = ui.as_weak();
    ui.on_unlock(move |file| {
        let unlock_handle = unlock_handle.upgrade().unwrap();

        if let Some(path) = files.get(&file.to_string()) {
            strip_protection(path).unwrap();
            /*let mdl = unlock_handle.get_files();
            let name = file.clone();
            mdl.map(move |mut fi| {
                if fi.file_name == name {
                    fi.locked = is_protected(path).unwrap();
                }
            });*/
        }//else display some sort of toast message with the error?
    });

    ui.run()
}

/*
let model: Rc<VecModel<JsonTheme>> = Rc::new(VecModel::from(themes));

        ModelRc::from(marg
         */
