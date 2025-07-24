use rfd::FileDialog;

pub fn system_file_chooser() -> Option<String> {
    FileDialog::new()
        .pick_file()
        .map(|path| path.display().to_string())
}