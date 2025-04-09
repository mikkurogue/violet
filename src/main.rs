use crossterm::{
    cursor::{EnableBlinking, Show},
    event::{self, Event, KeyCode, KeyEvent, KeyModifiers},
    execute,
    terminal::{
        EnableLineWrap, EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode,
        enable_raw_mode,
    },
};
use std::time::Duration;
use std::{env, io::stdout};
use violet::editor::editor::Editor;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    enable_raw_mode()?;
    let mut stdout = stdout();
    execute!(
        stdout,
        EnterAlternateScreen,
        Show,
        EnableBlinking,
        EnableLineWrap
    )?;

    let args: Vec<String> = env::args().collect();
    let filename = args.get(1).cloned();

    let mut editor = Editor::new(filename);

    let mut should_quit = false;
    while !should_quit {
        editor.render(&mut stdout)?;

        if event::poll(Duration::from_millis(16))? {
            if let Event::Key(KeyEvent {
                code, modifiers, ..
            }) = event::read()?
            {
                if code == KeyCode::Char('c') && modifiers.contains(KeyModifiers::CONTROL) {
                    // lets not close on signal ctrl c lol
                    should_quit = false;
                } else {
                    should_quit = editor.handle_keypress(code);
                }
            }
        }
    }

    disable_raw_mode()?;
    execute!(stdout, LeaveAlternateScreen, Show)?;

    Ok(())
}
