use std::{path::Path, sync::OnceLock};
use directories::ProjectDirs;
use log::{error, LevelFilter};

#[cfg(unix)]
use systemd_journal_logger::{connected_to_journal, JournalLog};

static DATA_DIR: OnceLock<Option<ProjectDirs>> = OnceLock::new();

pub fn logging() {
    #[cfg(unix)] //is this true for Mac? how would I test for non-mac *nix? I don't even have a Mac to test against
    let fallback_needed = if connected_to_journal() {
        //if systemd journal is found use it
        if let Ok(jl) = JournalLog::new() {
            //if is err return true so we setup a fall back log
            jl.with_extra_fields(vec![("VERSION", env!("CARGO_PKG_VERSION"))])
            .with_syslog_identifier("PV Unlocker".to_string())
            .install().is_err()
        } else {
            //we DO need a fallback since connecting to systemd journal failed
            true
        }
    } else {
        //No sysemd journal check for syslog, and then fall back to text file
        //My dev machine here DOES have a systemd journal....is there an easy way to disable that
        //and test syslog? Never tried, or do I need to spin up a different VM?
        
        let formatter= syslog::Formatter3164 {
            facility: syslog::Facility::LOG_USER,
            hostname: None,
            process: "PV Unlocker".into(),
            pid: 0,
        };

        if let Ok(logger) = syslog::unix(formatter) {
            use syslog::BasicLogger;
            log::set_boxed_logger(Box::new(BasicLogger::new(logger))).is_err()
        } else {
            true
        }
    };

    #[cfg(windows)]
    // Essentially for Windows Events to show up in Event Viewer you need registry keys to be created
    // This typically occurs at installation (as the user may not have Admin rights to create the keys)
    // For such a simple application that right now can just be run directly I'm not sure it's worth
    // the bother, and if it's not "installed" you would need something like TraceViewer up and running
    // before hand. For now I think it's easier to just log to a file in <User>/AppData. If I ever get around
    // to creating an installer to register a right click handler, and/or have some sort of shell extension
    // to open multiple files from Explorer without launching new instances this might be worth it
    // See Below Links for more information
    // https://learn.microsoft.com/en-us/answers/questions/979742/logs-are-not-appearing-in-windows-event-viewer
    // https://learn.microsoft.com/en-us/windows/win32/eventlog/event-sources
    let fallback_needed = true;

    #[cfg(target_os = "macos")]
    //https://github.com/steven-joruk/oslog
    let fallback_needed = true;
    
    if fallback_needed {
        //setup text file logging if:
        //    * nix systems: systemd journal or syslog fails
        //    * Windows: Event log fails
        //    * Mac: always currently
        if let Some(data_dir) = get_dir() {
            eprintln!("{:?}", data_dir);
        }
    }

    //default to warn until we can read from settings or ENV
    log::set_max_level(LevelFilter::Warn);
}

//Get the appropriate directory for app settings, either the XDG directory spec on Linux
//or AppData for the user on Windows, so this can be used as a fallback for text based logging
//should systemd journals/syslog/Windows ETW/etc fails
fn get_dir() -> Option<&'static Path> {
    if let Some(dirs) = DATA_DIR.get_or_init(|| ProjectDirs::from("org", "Vadoola", "PV Unlocker")) {
        Some(dirs.config_dir())
    } else {
        None
    }
}

/*
let file = cfg_dir.join("preferences.json");
if let Ok(set_file) = File::open(file) {
let reader = BufReader::new(set_file);
*/