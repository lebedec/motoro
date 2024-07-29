#[cfg(target_os = "windows")]
pub mod native {

    #[link(name = "user32")]
    extern "C" {
        ///  System DPI aware. This window does not scale for DPI changes.
        pub fn SetProcessDPIAware();
    }

    pub fn setup_process_dpi() {
        unsafe {
            SetProcessDPIAware();
        }
    }
}

#[cfg(not(target_os = "windows"))]
pub mod native {
    pub fn setup_process_dpi() {}
}
