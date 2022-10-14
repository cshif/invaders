use std::{
    error::Error,
    io,
    time::Duration,
    sync::mpsc,
    thread
};
use rusty_audio::Audio;
use crossterm::{
    terminal::{
        self,
        EnterAlternateScreen,
        LeaveAlternateScreen,
    },
    ExecutableCommand,
    cursor::{Hide, Show},
    event::{self, Event, KeyCode}
};
use invaders::{
    frame::{self, new_frame, Drawable},
    render,
    player::Player
};

fn main() -> Result<(), Box<dyn Error>> {
    let mut audio = Audio::new();
    audio.add("explode", "src/sounds/explode.wav");
    audio.add("lose", "src/sounds/lose.wav");
    audio.add("move", "src/sounds/move.wav");
    audio.add("pew", "src/sounds/pew.wav");
    audio.add("startup", "src/sounds/startup.wav");
    audio.add("win", "src/sounds/win.wav");

    audio.play("startup");

    // Terminal
    let mut stdout = io::stdout();
    terminal::enable_raw_mode()?;
    stdout.execute(EnterAlternateScreen)?;
    stdout.execute(Hide)?;

    // Render loop in a separate thread
    let (render_tx, render_rx) = mpsc::channel();
    let render_handle = thread::spawn(move || {
        let mut last_frame = frame::new_frame();
        let mut stdout = io::stdout();
        render::render(&mut stdout, &last_frame, &last_frame, true);

        loop {
            let current_frame = match render_rx.recv() {
                Ok(x) => x,
                Err(_) => break
            };
            render::render(&mut stdout, &last_frame, &current_frame, false);
            last_frame = current_frame
        }
    });

    // Game Loop
    let mut player = Player::new();
    'gameloop: loop {
        // Per-frame initialization
        let mut current_frame = new_frame();

        // Input
        while event::poll(Duration::default())? {
            if let Event::Key(key_event) = event::read()? {
                match key_event.code{
                    KeyCode::Left => player.move_left(),
                    KeyCode::Right => player.move_right(),
                    KeyCode::Esc | KeyCode::Char('q') => {
                        audio.play("lose");
                        break 'gameloop;
                    }
                    _ => {}
                }
            }
        }

        // Draw and render
        player.draw(&mut current_frame);
        let _ = render_tx.send(current_frame);
        thread::sleep(Duration::from_millis(1));
    }

    // Cleanup
    drop(render_tx);
    render_handle.join().unwrap();
    audio.wait();
    stdout.execute(Show)?;
    stdout.execute(LeaveAlternateScreen)?;
    terminal::disable_raw_mode()?;
    Ok(())
}
