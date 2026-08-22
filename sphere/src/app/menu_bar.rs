pub enum MenuAction {
    None,
    OpenFile,
    Quit,
    ToggleTrackInfo,
}

pub fn menu_bar(ui: &mut egui::Ui) -> MenuAction {
    let mut action = MenuAction::None;
    egui::menu::MenuBar::default().ui(ui, |ui| {
        ui.menu_button("File", |ui| {
            if ui.button("Open").clicked() {
                action = MenuAction::OpenFile;
            }
            if ui.button("Quit").clicked() {
                action = MenuAction::Quit;
            }
        });
        ui.menu_button("Info", |ui| {
            if ui.button("Track Info").clicked() {
                action = MenuAction::ToggleTrackInfo;
            }
        });
    });
    action
}
