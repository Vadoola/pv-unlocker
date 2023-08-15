#![windows_subsystem = "windows"]
//#![deny(clippy::all)]
//#![deny(clippy::pedantic)]
//#![deny(clippy::nursery)]
//#![deny(clippy::cargo)]
//#![deny(missing_docs)]

use ab_versions::{get_version, is_protected, strip_protection};
use clap::Parser;
use rayon::prelude::{IntoParallelRefIterator, ParallelIterator};
use rfd::FileDialog;
use slint::{Model, ModelRc, VecModel};
use std::{borrow::BorrowMut, cell::RefCell, collections::HashMap, path::PathBuf, rc::Rc};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
    files: Vec<PathBuf>,
}

slint::include_modules!();

fn main() -> Result<(), slint::PlatformError> {
    let args = Args::parse_from(wild::args());
    let files = Rc::new(RefCell::new(process_paths(args.files)));

    let ui = AppWindow::new()?;
    let mut file_model = Rc::new(VecModel::default());

    //if files were passed on the command line process them and add them to the UI model
    let info = get_file_info(&files.borrow());
    file_model.borrow_mut().extend(info.into_iter());

    ui.set_files(ModelRc::from(file_model.clone()));

    let unlock_file_model = file_model.clone();
    let unlock_files = files.clone();
    ui.on_unlock(move |file, idx| {
        if let Some(path) = unlock_files.borrow().get(&file.to_string()) {
            strip_protection(path).unwrap();

            //After attempting to unlock it update the model with the new protected status
            //by verifying it in the file on disk
            let unlock_file_model = unlock_file_model.as_ref();
            if let Some(mut row_data) = unlock_file_model.row_data(idx as usize) {
                row_data.locked = is_protected(&path).unwrap();
                unlock_file_model.set_row_data(idx as usize, row_data);
            } //else...hmm error updating the model?...what to do?
        } //else display some sort of toast message with the error?
    });

    let sel_file_model = file_model.clone();
    let mut sel_files = files.clone();
    ui.on_select_files(move || {
        if let Some(new_files) = FileDialog::new()
            .add_filter("FT View Files", &["mer", "MER", "apa", "APA"])
            .pick_files()
        {
            sel_files.borrow_mut().replace(process_paths(new_files));

            sel_file_model.set_vec(get_file_info(&sel_files.borrow()));
        }
    });

    ui.run()
}

fn process_paths(files: Vec<PathBuf>) -> HashMap<String, PathBuf> {
    files
        .into_iter()
        .map(|pb| {
            let name = pb
                .file_name()
                .map(std::ffi::OsStr::to_string_lossy)
                .unwrap()
                .to_string();
            (name, pb)
        })
        .collect()
}

fn get_file_info(files: &HashMap<String, PathBuf>) -> Vec<file_info> {
    files
        .par_iter()
        .map(|(name, file)| file_info {
            locked: is_protected(&file).unwrap(),
            file_name: name.into(),
            file_ver: get_version(&file).unwrap().to_string().into(),
        })
        .collect()
}
