#![windows_subsystem = "windows"]
//#![deny(clippy::all)]
//#![deny(clippy::pedantic)]
//#![deny(clippy::nursery)]
//#![deny(clippy::cargo)]
//#![deny(missing_docs)]

use ab_versions::{get_version, is_protected, strip_protection};
use clap::Parser;
use log::error;
use rayon::prelude::{IntoParallelRefIterator, ParallelIterator};
use rfd::FileDialog;
use simplelog::{CombinedLogger, Config, LevelFilter, SimpleLogger, WriteLogger};
use slint::{Model, ModelRc, VecModel};
use std::{
    borrow::BorrowMut, cell::RefCell, collections::HashMap, fs::File, path::PathBuf, rc::Rc,
};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
    files: Vec<PathBuf>,
}

slint::include_modules!();

fn main() -> Result<(), slint::PlatformError> {
    CombinedLogger::init(vec![
        #[cfg(feature = "termcolor")]
        TermLogger::new(
            LevelFilter::Warn,
            Config::default(),
            TerminalMode::Mixed,
            ColorChoice::Auto,
        ),
        #[cfg(not(feature = "termcolor"))]
        SimpleLogger::new(LevelFilter::Warn, Config::default()),
        WriteLogger::new(
            LevelFilter::Info,
            Config::default(),
            File::create("my_rust_binary.log").expect("Failed to create log file"),
        ),
    ])
    .expect("Failed to create logging infrastructure");

    let args = Args::parse_from(wild::args());
    let files = Rc::new(RefCell::new(process_paths(args.files)));

    let ui = AppWindow::new()?;
    let mut file_model = Rc::new(VecModel::default());

    //if files were passed on the command line process them and add them to the UI model
    let info = get_file_info(&files.borrow());
    file_model.borrow_mut().extend(info);

    ui.set_files(ModelRc::from(file_model.clone()));

    let unlock_file_model = file_model.clone();
    let unlock_files = files.clone();
    ui.on_unlock(move |file, idx| {
        if let Some(path) = unlock_files.borrow().get(&file.to_string()) {
            match strip_protection(path) {
                Ok(_) => {
                    //After attempting to unlock it update the model with the new protected status
                    //by verifying it in the file on disk
                    let unlock_file_model = unlock_file_model.as_ref();
                    if let Some(mut row_data) = unlock_file_model.row_data(idx as usize) {
                        match is_protected(&path) {
                            Ok(lck) => {
                                row_data.locked = lck;
                                unlock_file_model.set_row_data(idx as usize, row_data);
                            }
                            Err(e) => {
                                error!(
                                    "Unable to confirm file {} was unlocked. Reason: {e}",
                                    path.display()
                                )
                            }
                        }
                    }
                }
                Err(e) => {
                    error!("Failed to unlock file {}. Reason: {e}", path.display())
                }
            }
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
            let name = {
                let tmp_name = pb.file_name().map(std::ffi::OsStr::to_string_lossy);

                if let Some(tmp_name) = tmp_name {
                    tmp_name.to_string()
                } else {
                    pb.display().to_string()
                }
            };
            (name, pb)
        })
        .collect()
}

fn get_file_info(files: &HashMap<String, PathBuf>) -> Vec<file_info> {
    files
        .par_iter()
        .filter_map(|(name, file)| match is_protected(&file) {
            Ok(lckd) => match get_version(&file) {
                Ok(ver) => Some(file_info {
                    locked: lckd,
                    file_name: name.into(),
                    file_ver: ver.to_string().into(),
                }),
                Err(e) => {
                    error!(
                        "Unable to get file information from {}. Reason: {e}",
                        file.display()
                    );
                    None
                }
            },
            Err(e) => {
                error!(
                    "Unable to get file information from {}. Reason: {e}",
                    file.display()
                );
                None
            }
        })
        .collect()
}
