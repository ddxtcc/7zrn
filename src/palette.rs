#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum PaletteAction {
    NewTab,
    CloseTab,
    NextTab,
    PreviousTab,
    Search,
    Properties,
    Extract,
    Add,
    Delete,
    Rename,
    NewFolder,
    Test,
    Refresh,
    Settings,
    Help,
    ToggleHidden,
}

impl PaletteAction {
    pub(crate) const ALL: [PaletteAction; 16] = [
        PaletteAction::NewTab,
        PaletteAction::CloseTab,
        PaletteAction::NextTab,
        PaletteAction::PreviousTab,
        PaletteAction::Search,
        PaletteAction::Properties,
        PaletteAction::Extract,
        PaletteAction::Add,
        PaletteAction::Delete,
        PaletteAction::Rename,
        PaletteAction::NewFolder,
        PaletteAction::Test,
        PaletteAction::Refresh,
        PaletteAction::Settings,
        PaletteAction::Help,
        PaletteAction::ToggleHidden,
    ];

    pub(crate) fn label(self) -> &'static str {
        match self {
            PaletteAction::NewTab => "New tab",
            PaletteAction::CloseTab => "Close tab",
            PaletteAction::NextTab => "Next tab",
            PaletteAction::PreviousTab => "Previous tab",
            PaletteAction::Search => "Search current tree",
            PaletteAction::Properties => "Archive properties",
            PaletteAction::Extract => "Extract archive",
            PaletteAction::Add => "Add files to archive",
            PaletteAction::Delete => "Delete archive entries",
            PaletteAction::Rename => "Rename archive entry",
            PaletteAction::NewFolder => "New archive folder",
            PaletteAction::Test => "Test archive",
            PaletteAction::Refresh => "Refresh",
            PaletteAction::Settings => "Settings",
            PaletteAction::Help => "Keyboard shortcuts",
            PaletteAction::ToggleHidden => "Toggle hidden files",
        }
    }

    pub(crate) fn hint(self) -> &'static str {
        match self {
            PaletteAction::NewTab => "Ctrl+Shift+T",
            PaletteAction::CloseTab => "Ctrl+W",
            PaletteAction::NextTab => "Ctrl+Tab",
            PaletteAction::PreviousTab => "Ctrl+Shift+Tab",
            PaletteAction::Search => "Ctrl+Shift+F",
            PaletteAction::Extract => "Ctrl+E",
            PaletteAction::Add => "Ctrl+D",
            PaletteAction::Delete => "Delete",
            PaletteAction::Rename => "F2",
            PaletteAction::NewFolder => "F7",
            PaletteAction::Test => "Ctrl+T",
            PaletteAction::Refresh => "F5",
            PaletteAction::Settings => "Ctrl+,",
            PaletteAction::Help => "F1",
            PaletteAction::ToggleHidden => "Ctrl+H",
            PaletteAction::Properties => "",
        }
    }
}

pub(crate) fn matches(query: &str) -> Vec<PaletteAction> {
    let query = query.trim().to_lowercase();
    PaletteAction::ALL
        .into_iter()
        .filter(|action| query.is_empty() || action.label().to_lowercase().contains(&query))
        .collect()
}
