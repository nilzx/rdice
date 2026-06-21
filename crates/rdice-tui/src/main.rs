use std::io;
use std::time::Duration;

use crossterm::event::{self, Event, KeyCode, KeyEvent};
use crossterm::execute;
use crossterm::terminal::{
    EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode,
};
use ratatui::Terminal;
use ratatui::backend::CrosstermBackend;
use rdice_core::error::{DiceError, Result};
use rdice_tui::app::App;
use rdice_tui::command::parse_command;
use rdice_tui::input::{InputAction, InputState};

fn main() {
    if let Err(err) = run() {
        eprintln!("Error: {err}");
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    let mut app = App::load_default()?;
    let mut terminal = setup_terminal()?;
    let mut input = InputState::default();

    let app_result = run_app(&mut terminal, &mut app, &mut input);
    let restore_result = restore_terminal(&mut terminal);

    app_result?;
    restore_result
}

fn run_app(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    app: &mut App,
    input: &mut InputState,
) -> Result<()> {
    while !app.should_quit {
        terminal
            .draw(|frame| rdice_tui::ui::render(frame, app))
            .map_err(to_storage_error)?;

        if event::poll(Duration::from_millis(100)).map_err(to_storage_error)?
            && let Event::Key(key) = event::read().map_err(to_storage_error)?
        {
            handle_key(app, input, key)?;
        }
    }

    Ok(())
}

fn setup_terminal() -> Result<Terminal<CrosstermBackend<io::Stdout>>> {
    enable_raw_mode().map_err(to_storage_error)?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen).map_err(to_storage_error)?;
    let backend = CrosstermBackend::new(stdout);
    Terminal::new(backend).map_err(to_storage_error)
}

fn restore_terminal(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>) -> Result<()> {
    disable_raw_mode().map_err(to_storage_error)?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen).map_err(to_storage_error)?;
    terminal.show_cursor().map_err(to_storage_error)?;
    Ok(())
}

fn handle_key(app: &mut App, input: &mut InputState, key: KeyEvent) -> Result<()> {
    if app.command_buffer.is_some() {
        handle_command_key(app, key);
        return Ok(());
    }

    let action = match key.code {
        KeyCode::Esc => {
            input.clear();
            Some(InputAction::Escape)
        }
        KeyCode::PageUp => Some(InputAction::PreviousPage),
        KeyCode::PageDown => Some(InputAction::NextPage),
        KeyCode::Char(ch) => {
            let action = input.push(ch);
            if action.is_none() {
                app.message = input.pending_hint().map(str::to_string);
            }
            action
        }
        _ => None,
    };

    if let Some(action) = action {
        match apply_input_action(app, action) {
            Ok(()) => {}
            Err(err) => app.message = Some(err.to_string()),
        }
    }

    Ok(())
}

fn handle_command_key(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Esc => app.leave_command_mode(),
        KeyCode::Enter => {
            let command_input = app.command_buffer.clone().unwrap_or_default();
            app.leave_command_mode();
            match parse_command(&command_input).and_then(|command| app.apply_command(command)) {
                Ok(()) => {}
                Err(err) => app.message = Some(err.to_string()),
            }
        }
        KeyCode::Backspace => {
            if let Some(buffer) = &mut app.command_buffer {
                buffer.pop();
            }
        }
        KeyCode::Char(ch) => {
            if let Some(buffer) = &mut app.command_buffer {
                buffer.push(ch);
            }
        }
        _ => {}
    }
}

fn apply_input_action(app: &mut App, action: InputAction) -> Result<()> {
    match action {
        InputAction::ToggleTray(id) => match app.screen {
            rdice_tui::app::Screen::Overview => app.toggle_tray_selection(id)?,
            rdice_tui::app::Screen::AddDie(_) => app.add_die_by_page_id(id)?,
            _ => {}
        },
        InputAction::OpenTray(id) => app.open_tray_by_page_id(id)?,
        InputAction::Roll => app.roll_from_current_screen()?,
        InputAction::ToggleText => app.toggle_text_visible(),
        InputAction::ToggleRange => app.toggle_range_visible(),
        InputAction::ToggleEv => app.toggle_ev_visible(),
        InputAction::OpenHistory => app.apply_command(rdice_tui::command::Command::History)?,
        InputAction::PreviousPage => app.previous_page(),
        InputAction::NextPage => app.next_page(),
        InputAction::EnterCommandMode => app.enter_command_mode(),
        InputAction::Quit => app.should_quit = true,
        InputAction::Escape => app.escape(),
        InputAction::AddDie => {
            if let rdice_tui::app::Screen::TrayDetail(name) = &app.screen {
                app.add_die_page = 0;
                app.screen = rdice_tui::app::Screen::AddDie(name.clone());
            }
        }
        InputAction::OpenManager => app.open_context_manager(),
        InputAction::NewTarget => app.prefill_new_target()?,
        InputAction::DeleteTarget(target_id) => match app.screen {
            rdice_tui::app::Screen::TrayDetail(_) | rdice_tui::app::Screen::AddDie(_) => {
                app.remove_slot(target_id)?
            }
            rdice_tui::app::Screen::TrayManager | rdice_tui::app::Screen::DiceManager => {
                app.prefill_delete_target(target_id as usize)?
            }
            _ => {}
        },
        InputAction::EditTarget(target_id) => app.prefill_edit_target(target_id)?,
        InputAction::ToggleSlotLock(slot_id) => app.toggle_slot_lock(slot_id)?,
    }

    Ok(())
}

fn to_storage_error(err: impl std::fmt::Display) -> DiceError {
    DiceError::StorageError(err.to_string())
}
