pub(crate) struct WindowInfo {
    pub(crate) title: String,
    pub(crate) class_name: String,
    pub(crate) process_name: String,
    pub(crate) formatted_json: String,
}

pub(crate) enum AppEvent {
    CaptureStarted,
    CaptureFinished(Result<WindowInfo, String>),
    Show,
    Exit,
}
