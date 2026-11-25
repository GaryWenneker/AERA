#![forbid(unsafe_code)]

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Panel {
    Files,
    Disks,
    Settings,
    About,
    Terminal,
}

impl Panel {
    pub const ALL: [Self; 5] = [
        Self::Files,
        Self::Disks,
        Self::Settings,
        Self::About,
        Self::Terminal,
    ];

    pub fn title(self) -> &'static str {
        match self {
            Self::Files => "Files",
            Self::Disks => "Disks",
            Self::Settings => "Settings",
            Self::About => "About",
            Self::Terminal => "Terminal",
        }
    }
}

pub fn search(query: &str) -> Vec<Panel> {
    let query = query.trim().to_ascii_lowercase();
    Panel::ALL
        .into_iter()
        .filter(|panel| panel.title().to_ascii_lowercase().contains(&query))
        .collect()
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum WindowState {
    #[default]
    Closed,
    Floating,
    Minimized,
    Maximized,
}

#[derive(Default)]
struct Workspace {
    windows: [WindowState; 5],
}

pub struct Desktop {
    workspaces: Vec<Workspace>,
    active: usize,
}

impl Default for Desktop {
    fn default() -> Self {
        Self {
            workspaces: vec![Workspace::default()],
            active: 0,
        }
    }
}

impl Desktop {
    pub const MAX_WORKSPACES: usize = 8;

    pub fn active(&self) -> usize {
        self.active
    }

    pub fn count(&self) -> usize {
        self.workspaces.len()
    }

    pub fn add_workspace(&mut self) -> bool {
        if self.count() >= Self::MAX_WORKSPACES {
            return false;
        }
        self.workspaces.push(Workspace::default());
        self.active = self.count() - 1;
        true
    }

    pub fn switch(&mut self, index: usize) -> bool {
        if index >= self.count() {
            return false;
        }
        self.active = index;
        true
    }

    pub fn state(&self, panel: Panel) -> WindowState {
        self.workspaces[self.active].windows[panel as usize]
    }

    pub fn is_open(&self, panel: Panel) -> bool {
        self.state(panel) != WindowState::Closed
    }

    pub fn maximized(&self) -> Option<Panel> {
        Panel::ALL
            .into_iter()
            .find(|panel| self.state(*panel) == WindowState::Maximized)
    }

    pub fn set_state(&mut self, panel: Panel, state: WindowState) {
        if state == WindowState::Maximized {
            if let Some(previous) = self.maximized() {
                self.workspaces[self.active].windows[previous as usize] = WindowState::Floating;
            }
        }
        self.workspaces[self.active].windows[panel as usize] = state;
    }

    pub fn set_open(&mut self, panel: Panel, open: bool) {
        if !open {
            self.set_state(panel, WindowState::Closed);
        } else if matches!(
            self.state(panel),
            WindowState::Closed | WindowState::Minimized
        ) {
            self.set_state(panel, WindowState::Floating);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn search_filters_and_normalizes() {
        assert_eq!(search("  SET "), vec![Panel::Settings]);
        assert_eq!(search(""), Panel::ALL);
        assert!(search("missing").is_empty());
    }

    #[test]
    fn maximize_is_exclusive_and_minimize_can_restore() {
        let mut desktop = Desktop::default();
        desktop.set_state(Panel::Files, WindowState::Maximized);
        desktop.set_state(Panel::Disks, WindowState::Maximized);
        assert_eq!(desktop.state(Panel::Files), WindowState::Floating);
        assert_eq!(desktop.maximized(), Some(Panel::Disks));
        desktop.set_state(Panel::Disks, WindowState::Minimized);
        assert!(desktop.is_open(Panel::Disks));
        assert_eq!(desktop.maximized(), None);
        desktop.set_open(Panel::Disks, true);
        assert_eq!(desktop.state(Panel::Disks), WindowState::Floating);
    }

    #[test]
    fn workspaces_preserve_independent_windows_and_limits() {
        let mut desktop = Desktop::default();
        desktop.set_state(Panel::Files, WindowState::Maximized);
        assert!(desktop.add_workspace());
        assert!(!desktop.is_open(Panel::Files));
        desktop.set_open(Panel::Disks, true);
        assert!(desktop.switch(0));
        assert_eq!(desktop.maximized(), Some(Panel::Files));
        assert!(!desktop.is_open(Panel::Disks));
        assert!(!desktop.switch(99));
        while desktop.add_workspace() {}
        assert_eq!(desktop.count(), Desktop::MAX_WORKSPACES);
    }

    #[test]
    fn windows_are_independent() {
        let mut desktop = Desktop::default();
        assert!(!desktop.is_open(Panel::Files));
        desktop.set_open(Panel::Files, true);
        desktop.set_open(Panel::Settings, true);
        desktop.set_open(Panel::Files, false);
        assert!(!desktop.is_open(Panel::Files));
        assert!(desktop.is_open(Panel::Settings));
    }
}
