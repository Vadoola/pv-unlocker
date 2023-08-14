fn main() {
    #[cfg(windows)]
    windres::Build::new().compile("pv-unlocker.rc").unwrap();
    
    slint_build::compile("ui/appwindow.slint").unwrap();
}
