use rdice_tui::input::{InputAction, InputState};

#[test]
fn overview_digit_selects_tray() {
    let mut state = InputState::default();
    assert_eq!(state.push('3'), Some(InputAction::ToggleTray(3)));
}

#[test]
fn overview_o_digit_opens_tray() {
    let mut state = InputState::default();
    assert_eq!(state.push('o'), None);
    assert_eq!(state.push('3'), Some(InputAction::OpenTray(3)));
}

#[test]
fn tray_l_digit_locks_slot() {
    let mut state = InputState::default();
    assert_eq!(state.push('l'), None);
    assert_eq!(state.push('3'), Some(InputAction::ToggleSlotLock(3)));
}

#[test]
fn tray_d_digit_removes_slot() {
    let mut state = InputState::default();
    assert_eq!(state.push('d'), None);
    assert_eq!(state.pending_hint(), Some("delete/remove"));
    assert_eq!(state.push('3'), Some(InputAction::DeleteTarget(3)));
}

#[test]
fn command_key_enters_command_mode() {
    let mut state = InputState::default();
    assert_eq!(state.push(':'), Some(InputAction::EnterCommandMode));
}

#[test]
fn manager_key_opens_contextual_manager() {
    let mut state = InputState::default();
    assert_eq!(state.push('m'), Some(InputAction::OpenManager));
}

#[test]
fn manager_shortcuts_start_new_and_edit_targets() {
    let mut state = InputState::default();
    assert_eq!(state.push('n'), Some(InputAction::NewTarget));
    assert_eq!(state.push('e'), None);
    assert_eq!(state.pending_hint(), Some("edit"));
    assert_eq!(state.push('2'), Some(InputAction::EditTarget(2)));
}
