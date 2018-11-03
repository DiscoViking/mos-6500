extern crate mos_6500;

use std::cell::RefCell;
use std::cmp::{max, min};
use std::env;
use std::rc::Rc;
use std::thread;
use std::time::{Duration, Instant};

use mos_6500::emulator;
use mos_6500::emulator::clock::Ticker;
use mos_6500::emulator::ines;
use mos_6500::emulator::io;
use mos_6500::emulator::io::{Input};
use mos_6500::emulator::io::event::{Event, EventHandler, Key};
use mos_6500::emulator::io::sdl;

fn main() {
    let args: Vec<String> = env::args().collect();

    let rom_path = match args.get(2) {
        None => panic!("You must pass in a path to a iNes ROM file."),
        Some(path) => path,
    };

    let rom = ines::ROM::load(rom_path);

    let io = Rc::new(RefCell::new(sdl::IO::new()));
    let output = io::SimpleVideoOut::new(io.clone());

    let mut nes = emulator::NES::new(io.clone(), output, rom);

    let lifecycle = Rc::new(RefCell::new(Lifecycle::new()));
    lifecycle.borrow_mut().start();
    io.borrow_mut().register_event_handler(Box::new(lifecycle.clone()));

    let started_instant = Instant::now();
    let frames_per_second = 30;
    let mut frame_start = started_instant;
    let mut frame_ix = 0;
    let mut agg_cycles = 0;
    let mut agg_start = started_instant;
    let mut overflow_cycles = 0;

    while lifecycle.borrow().is_running() {
        let target_hz = lifecycle.borrow().target_hz();
        let target_frame_cycles = target_hz / frames_per_second;
        let target_frame_time_ns = 1_000_000_000 / frames_per_second;

        let mut cycles_this_frame = 0;
        let target_cycles_this_frame = target_frame_cycles - overflow_cycles;
        let mut frame_ns = 0;

        while cycles_this_frame < target_cycles_this_frame && frame_ns < target_frame_time_ns {
            // Batching ticks here is a massive perf win since finding the elapsed time is costly.
            // Reduce batch size when we're nearly done with a frame to try and get really close to
            // the exact number.
            let batch_size = 100;//max(1, min(1_000, (target_frame_cycles - cycles_this_frame) / 1000));
            for _ in 1 .. batch_size {
                cycles_this_frame += nes.tick();
            }

            let frame_time = frame_start.elapsed();
            frame_ns = frame_time.as_secs() * 1_000_000_000 + (frame_time.subsec_nanos() as u64);
        }

        io.borrow_mut().tick();

        let frame_end = Instant::now();
        let frame_time = frame_end - frame_start;
        frame_ns = frame_time.as_secs() * 1_000_000_000 + (frame_time.subsec_nanos() as u64);
        let sleep_ns = target_frame_time_ns.saturating_sub(frame_ns);

        // Set frame_start to what we INTEND for it to be, so we will adjust for the sleep not
        // being an exact amount.
        frame_start = frame_end + Duration::from_nanos(sleep_ns);
        overflow_cycles = cycles_this_frame.saturating_sub(target_cycles_this_frame);
        thread::sleep(Duration::from_nanos(sleep_ns));
        
        // Print debug info here.
        agg_cycles += cycles_this_frame;
        frame_ix = (frame_ix + 1) % frames_per_second;
        if frame_ix == 0 {
            let agg_duration = agg_start.elapsed();
            agg_start = Instant::now();

            let agg_ns = agg_duration.as_secs() * 1_000_000_000 + (agg_duration.subsec_nanos() as u64);
            let current_hz = (agg_cycles * 1_000_000_000) / agg_ns;

            println!(
                "Target: {:.3}MHz, Current: {:.3}MHz ({:.2}x)",
                (target_hz as f64) / 1_000_000f64,
                (current_hz as f64) / 1_000_000f64,
                (current_hz as f64) / (emulator::NES_MASTER_CLOCK_HZ as f64),
            );

            agg_cycles = 0;
        }
    }
}

pub struct Lifecycle {
    is_running: bool,
    unlock_speed: bool,
    target_hz: u64,
}

impl Lifecycle {
    pub fn new() -> Lifecycle {
        Lifecycle {
            is_running: false,
            unlock_speed: false,
            target_hz: emulator::NES_MASTER_CLOCK_HZ,
        }
    }

    pub fn is_running(&self) -> bool {
        self.is_running
    }

    pub fn start(&mut self) {
        self.is_running = true;
    }

    pub fn speed_is_unlocked(&self) -> bool {
        self.unlock_speed
    }

    pub fn target_hz(&self) -> u64 {
        self.target_hz
    }
}

impl EventHandler for Lifecycle {
    fn handle_event(&mut self, event: Event) {
        match event {
            Event::KeyDown(key) => {
                match key {
                    Key::Escape => self.is_running = false,
                    Key::Tab => self.unlock_speed = !self.unlock_speed,
                    Key::Minus => self.target_hz /= 2,
                    Key::Equals => self.target_hz *= 2,
                    Key::Num0 => self.target_hz = emulator::NES_MASTER_CLOCK_HZ,
                    _ => (),
                };
            },
            _ => (),
        };
    }
}
