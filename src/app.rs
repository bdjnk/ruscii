use super::keyboard::{Keyboard};
use super::terminal::{Window};

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc};
use std::{thread, time, panic};

use std::io::{self, BufRead};

pub struct Config {
    pub fps: u32,
}

impl Config {
    pub fn new() -> Config {
        Config {fps: 30}
    }

    pub fn fps(mut self, fps: u32) -> Config {
        self.fps = fps;
        self
    }
}


pub struct State {
    running: Arc<AtomicBool>,
    keyboard: Keyboard,
    pub(self) dt: time::Duration,
    pub(self) step: usize,
}

impl State {
    pub fn new() -> State {
        State {
            running: Arc::new(AtomicBool::new(false)),
            keyboard: Keyboard::new(),
            dt: time::Duration::new(0, 0),
            step: 0,
        }
    }

    pub fn run(&self) {
        self.running.store(true, Ordering::SeqCst);
    }

    pub fn stop(&self) {
        self.running.store(false, Ordering::SeqCst);
    }

    pub fn is_running(&self) -> bool {
       self.running.load(Ordering::SeqCst)
    }

    pub fn keyboard(&self) -> &Keyboard {
        &self.keyboard
    }

    pub fn dt(&self) -> &time::Duration {
        &self.dt
    }

    pub fn step(&self) -> usize {
        self.step
    }
}

pub struct App {
    config: Config,
    state: State,
    window: Window,
}

impl App {
    pub fn new() -> App {
        App {
            config: Config::new(),
            state: State::new(),
            window: Window::new(),
        }
    }

    pub fn config(config: Config) -> App {
        App {
            config,
            state: State::new(),
            window: Window::new(),
        }
    }

    pub fn window(&self) -> &Window {
        &self.window
    }

    pub fn run<F>(&mut self, mut frame_action: F)
    where F: FnMut(&mut State, &mut Window) {
        let expected_duration = time::Duration::from_nanos(1_000_000_000 / self.config.fps as u64);
        self.state.run();

        let result = panic::catch_unwind(panic::AssertUnwindSafe(||{
            self.window.open();
            while self.state.is_running() {

                let now = time::Instant::now();
                self.window.clear();

                self.state.keyboard.consume_key_events();
                frame_action(&mut self.state, &mut self.window);

                self.window.draw();

                self.state.dt = now.elapsed();
                self.state.step += 1;
                if let Some(time) = expected_duration.checked_sub(self.state.dt) {
                    thread::sleep(time);
                }
            }
            self.window.close();
        }));

        if let Err(_) = result {
            println!("\n\n[Press 'enter' to recover the terminal]");
            io::stdin().lock().lines().next().unwrap().unwrap();
            self.window.close();
        }
    }
}
