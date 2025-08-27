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
use slint::{Model, ModelRc, Timer, TimerMode, VecModel};
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
    let unlock_ui = ui.as_weak();
    ui.on_unlock(move |file, idx| {
        if let Some(path) = unlock_files.borrow().get(&file.to_string()) {
            if let Ok(ver) = get_version(path) {
                if ver.is_restorable() {
                    let unlock_file_model = unlock_file_model.as_ref();
                    match strip_protection(path) {
                        Ok(()) => {
                            //After attempting to unlock it update the model with the new protected status
                            //by verifying it in the file on disk
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
                                        );
                                        if let Some(unlock_ui) = unlock_ui.upgrade() {
                                            row_data.note =
                                                "Unable to confirm file was unlocked".into();
                                            unlock_file_model.set_row_data(idx as usize, row_data);
                                            unlock_ui.invoke_slide_over(idx);
                                        }
                                    }
                                }
                            }
                        }
                        Err(e) => {
                            error!("Failed to unlock file {}. Reason: {e}", path.display());
                            if let Some(mut row_data) = unlock_file_model.row_data(idx as usize) {
                                if let Some(unlock_ui) = unlock_ui.upgrade() {
                                    row_data.note = "Failed to unlock file".into();
                                    unlock_file_model.set_row_data(idx as usize, row_data);
                                    unlock_ui.invoke_slide_over(idx);
                                }
                            }
                        }
                    }
                } else if let Some(unlock_ui) = unlock_ui.upgrade() {
                    unlock_ui.invoke_slide_over(idx);
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

    let info_file_model_start = file_model.clone();
    let mut info_timers: Vec<Timer> = (0..file_model.row_count())
        .map(|_i| Timer::default())
        .collect();
    ui.on_slide_over(move |idx| {
        let idx = idx as usize;
        if idx >= info_timers.len() {
            info_timers.extend((info_timers.len()..idx + 1).map(|_| Timer::default()));
        }
        let info_file_model_stop = info_file_model_start.clone();
        let info_file_model_start = info_file_model_start.as_ref();
        if let Some(mut fi) = info_file_model_start.row_data(idx) {
            fi.note_vis = true;
            info_file_model_start.set_row_data(idx, fi);
        }
        info_timers[idx].start(
            TimerMode::SingleShot,
            std::time::Duration::from_secs(5),
            move || {
                let info_file_model_stop = info_file_model_stop.clone();
                let info_file_model_stop = info_file_model_stop.as_ref();
                if let Some(mut fi) = info_file_model_stop.row_data(idx) {
                    fi.note_vis = false;
                    info_file_model_stop.set_row_data(idx, fi);
                }
            },
        );
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
        .filter_map(|(name, file)| match get_version(&file) {
            Ok(ver) => {
                let note = if ver.is_old() {
                    if ver.is_restorable() {
                        "As an old MER some features may not restore correctly."
                    } else {
                        "Restoring a file this old is currently unsupported."
                    }
                } else {
                    ""
                };

                let lckd = if ver.is_restorable() {
                    match is_protected(&file) {
                        Ok(prot) => prot,
                        Err(e) => {
                            error!(
                                "Unable to get file protected status from {}. Reason: {e}",
                                file.display()
                            );
                            true
                        }
                    }
                } else {
                    true
                };

                Some(file_info {
                    locked: lckd,
                    file_name: name.into(),
                    file_ver: ver.to_string().into(),
                    note: note.into(),
                    note_vis: false,
                })
            }
            Err(e) => {
                error!(
                    "Unable to get file version from {}, file may not be a valid MER/APA file. Reason: {e}",
                    file.display()
                );
                None
            }
        })
        .collect()
}
