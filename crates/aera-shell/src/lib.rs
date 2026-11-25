#![forbid(unsafe_code)]

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Panel {
    Files,
    Settings,
    About,
}

impl Panel {
    pub const ALL: [Self; 3] = [Self::Files, Self::Settings, Self::About];

    pub fn title(self) -> &'static str {
        match self {
            Self::Files => "Files",
            Self::Settings => "Settings",
            Self::About => "About",
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

#[derive(Default)]
pub struct Desktop {
    open: [bool; 3],
}

impl Desktop {
    pub fn is_open(&self, panel: Panel) -> bool {
        self.open[panel as usize]
    }

    pub fn set_open(&mut self, panel: Panel, open: bool) {
        self.open[panel as usize] = open;
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
