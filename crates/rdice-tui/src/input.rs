#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InputAction {
    ToggleTray(usize),
    OpenTray(usize),
    Roll,
    ToggleText,
    ToggleRange,
    ToggleEv,
    PreviousPage,
    NextPage,
    EnterCommandMode,
    Quit,
    Escape,
    AddDie,
    OpenManager,
    NewTarget,
    DeleteTarget(u32),
    EditTarget(usize),
    ToggleSlotLock(u32),
}

#[derive(Debug, Default)]
pub struct InputState {
    pending_prefix: Option<char>,
}

impl InputState {
    pub fn push(&mut self, ch: char) -> Option<InputAction> {
        if let Some(prefix) = self.pending_prefix.take() {
            return match (prefix, ch.to_digit(10)) {
                ('o', Some(value @ 1..=9)) => Some(InputAction::OpenTray(value as usize)),
                ('l', Some(value @ 1..=9)) => Some(InputAction::ToggleSlotLock(value)),
                ('d', Some(value @ 1..=9)) => Some(InputAction::DeleteTarget(value)),
                ('e', Some(value @ 1..=9)) => Some(InputAction::EditTarget(value as usize)),
                _ => None,
            };
        }

        match ch {
            '1'..='9' => Some(InputAction::ToggleTray(ch.to_digit(10).unwrap() as usize)),
            'o' | 'l' | 'd' | 'e' => {
                self.pending_prefix = Some(ch);
                None
            }
            'r' => Some(InputAction::Roll),
            't' => Some(InputAction::ToggleText),
            'R' => Some(InputAction::ToggleRange),
            'E' => Some(InputAction::ToggleEv),
            ':' => Some(InputAction::EnterCommandMode),
            'q' => Some(InputAction::Quit),
            'a' => Some(InputAction::AddDie),
            'm' => Some(InputAction::OpenManager),
            'n' => Some(InputAction::NewTarget),
            _ => None,
        }
    }

    pub fn pending_hint(&self) -> Option<&'static str> {
        match self.pending_prefix {
            Some('o') => Some("open"),
            Some('l') => Some("lock/unlock"),
            Some('d') => Some("delete/remove"),
            Some('e') => Some("edit"),
            _ => None,
        }
    }

    pub fn clear(&mut self) {
        self.pending_prefix = None;
    }
}
