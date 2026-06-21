use rdice_tui::command::{Command, parse_command};

#[test]
fn parses_navigation_commands() {
    assert_eq!(parse_command("manager dice").unwrap(), Command::ManagerDice);
    assert_eq!(
        parse_command("manager trays").unwrap(),
        Command::ManagerTrays
    );
    assert_eq!(parse_command("overview").unwrap(), Command::Overview);
    assert_eq!(
        parse_command("tray combat").unwrap(),
        Command::OpenTray("combat".into())
    );
    assert_eq!(parse_command("quit").unwrap(), Command::Quit);
}

#[test]
fn parses_dice_commands() {
    assert_eq!(
        parse_command("dice new fate -1 0 +").unwrap(),
        Command::DiceNew {
            name: "fate".into(),
            faces: vec!["-1".into(), "0".into(), "+".into()],
        }
    );
    assert_eq!(
        parse_command("dice delete fate").unwrap(),
        Command::DiceDelete("fate".into())
    );
    assert_eq!(
        parse_command("dice edit fate -1 0 1").unwrap(),
        Command::DiceEdit {
            name: "fate".into(),
            faces: vec!["-1".into(), "0".into(), "1".into()],
        }
    );
}

#[test]
fn parses_tray_commands() {
    assert_eq!(
        parse_command("tray new combat").unwrap(),
        Command::TrayNew("combat".into())
    );
    assert_eq!(
        parse_command("tray delete combat").unwrap(),
        Command::TrayDelete("combat".into())
    );
    assert_eq!(
        parse_command("tray rename combat battle").unwrap(),
        Command::TrayRename {
            old_name: "combat".into(),
            new_name: "battle".into(),
        }
    );
}

#[test]
fn rejects_invalid_commands() {
    assert!(parse_command("").is_err());
    assert!(parse_command("manager").is_err());
    assert!(parse_command("dice new only_name").is_err());
    assert!(parse_command("dice edit only_name").is_err());
    assert!(parse_command("tray rename only_old").is_err());
}
