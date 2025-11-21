use std::time::{Duration, Instant};
use crossterm::cursor::{Hide, Show};
use crossterm::event::{Event, KeyCode};
use crossterm::terminal::{EnterAlternateScreen, LeaveAlternateScreen};
use crossterm::{ExecutableCommand, event, terminal};
use invaders::invaders::Invaders;
use rusty_audio::Audio;
use std::error::Error;
use std::{io, thread};
use std::sync::mpsc;
use invaders::{frame, render};
use invaders::frame::{new_frame, Drawable};
use invaders::player::Player;

fn main() -> Result<(), Box<dyn Error>> {

    // audio
    let mut audio = Audio::new();
    audio.add("explode", "original/explode.wav");
    audio.add("lose", "original/lose.wav");
    audio.add("move", "original/move.wav");
    audio.add("pew", "original/pew.wav");
    audio.add("startup", "original/startup.wav");
    audio.add("win", "original/win.wav");

    // startup audio
    audio.play("startup");

    //terminal
    let mut stdout = io::stdout();
    terminal::enable_raw_mode()?;
    stdout.execute(EnterAlternateScreen)?;
    stdout.execute(Hide)?;

    // a loop meant for screen rendering - to be run in a separate thread
    let (render_tx, render_rx) = mpsc::channel();
    let render_handle = thread::spawn(move || {
        let mut last_frame = frame::new_frame();
        let mut stdout = io::stdout();
        render::render(&mut stdout, &last_frame, &last_frame, true);
        'rendering: loop {
            let curr_frame = match render_rx.recv() {
                Ok(x) => x,
                Err(_) => break 'rendering,
            };
            render::render(&mut stdout, &last_frame, &curr_frame, false);
            last_frame = curr_frame;
        }
    });

    let mut player = Player::new();
    let mut instant = Instant::now();
    let mut invaders = Invaders::new();

    // 'gameloop: loop {
    //     // input
    //     while event::poll(Duration::default())? {
    //         match event::read()? {
    //             Event::Key(key_event_mf) => {
    //                 match key_event_mf.code {
    //                     KeyCode::Esc | KeyCode::Char('q') => {
    //                         audio.play("lose");
    //                         break 'gameloop;
    //                     }
    //                     _ => {}
    //                 }
    //             }
    //             _ => {}
    //         }
    //     }
    // }
    'gameloop: loop {
        // per-frame init
        let delta = instant.elapsed();
        instant = Instant::now();
        let mut curr_frame = new_frame();

        //input
        while event::poll(Duration::default())? {
            if let Event::Key(key_event_mf) = event::read()? {
                match key_event_mf.code {
                    KeyCode::Left => player.move_left(),
                    KeyCode::Right => player.move_right(),
                    KeyCode::Char(' ') | KeyCode::Enter => {
                        if player.shoot_successful() {
                            audio.play("pew");
                        }
                    }
                    KeyCode::Esc | KeyCode::Char('q') => {
                        audio.play("lose");
                        break 'gameloop;
                    },
                    _ => {},
                }
            }
        }

        // Updates - timer etc
        player.update(delta);
        if invaders.update(delta) {
            audio.play("move")
        }

        // draw and render
        player.draw(&mut curr_frame); // draw player before rendering anything else
        invaders.draw(&mut curr_frame); // draw invaders before rendering anything else
        let _ = render_tx.send(curr_frame); // after startup, for some time, there'd be no receiver available. This `let` is to ignore that thing to avoid a crash
        thread::sleep(Duration::from_millis(1));
    }

    // Cleanup
    // join the threads
    drop(render_tx); // render_rx auto shuts down
    render_handle.join().unwrap();

    audio.wait();
    stdout.execute(Show)?;
    stdout.execute(LeaveAlternateScreen)?;
    terminal::disable_raw_mode()?;
    
    Ok(())
}
