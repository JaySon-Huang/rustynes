mod constants;
mod noise;
mod square;
mod triangle;

use self::constants::*;
use self::noise::Noise;
use self::square::Square;
use self::triangle::Triangle;
use nes::types::{Addr, Data};

#[derive(Debug)]
pub struct Apu {
    squares: (Square, Square),
    triangle: Triangle,
    noise: Noise,
    cycle: u16,
    step: usize,
    sequencer_mode: bool,
    enable_irq: bool,
}

impl Apu {
    pub fn new() -> Self {
        Apu {
            squares: (Square::new(0), Square::new(1)),
            triangle: Triangle::new(2),
            noise: Noise::new(),
            cycle: 0,
            step: 0,
            sequencer_mode: false,
            enable_irq: false,
        }
    }

    pub fn run(&mut self, cycle: u16) {
        self.cycle += cycle;
        if self.cycle >= DIVIDE_COUNT_FOR_240HZ {
            // invoked by 240hz
            self.cycle -= DIVIDE_COUNT_FOR_240HZ;
            if self.sequencer_mode {
                self.update_by_sequence_mode1();
            } else {
                self.update_by_sequence_mode0();
            }
        }
    }

    pub fn read(&mut self, addr: Addr) -> Data {
        match addr {
            0x15 => {
                // self.interrupts.deassertIrq();
                let s0 = if self.squares.0.has_count_end() {
                    0x00
                } else {
                    0x01
                };
                let s1 = if self.squares.1.has_count_end() {
                    0x00
                } else {
                    0x02
                };
                let t = if self.triangle.has_count_end() {
                    0x00
                } else {
                    0x04
                };
                let n = if self.noise.has_count_end() {
                    0x00
                } else {
                    0x08
                };
                n | t | s1 | s0
            }
            _ => 0,
        }
    }

    pub fn write(&mut self, addr: Addr, data: Data) {
        match addr {
            0x00..=0x03 => {
                self.squares.0.write(addr, data);
            }
            0x04..=0x07 => {
                self.squares.1.write(addr - 0x04, data);
            }
            0x08..=0x0b => {
                self.triangle.write(addr - 0x08, data);
            }
            0x0c..=0x0f => {
                self.noise.write(addr - 0x0c, data);
            }
            0x15 => {
                if data & 0x01 == 0x01 {
                    self.squares.0.enable();
                } else {
                    self.squares.0.disable();
                }
                if data & 0x02 == 0x02 {
                    self.squares.1.enable();
                } else {
                    self.squares.1.disable();
                }
                if data & 0x04 == 0x04 {
                    self.triangle.enable();
                } else {
                    self.triangle.disable();
                }
                if data & 0x08 == 0x08 {
                    self.noise.start();
                } else {
                    self.noise.stop();
                }
            }
            0x17 => {
                self.sequencer_mode = data & 0x80 == 0x80;
                self.enable_irq = data & 0x40 == 0x40;
                // if self.sequencer_mode {
                self.step = 0;
                self.cycle = 0;
                // }
            }
            _ => (), //println!("addr {} data {}", addr, data),
        }
    }

    fn update_by_sequence_mode0(&mut self) {
        self.update_envelope();
        if self.step % 2 == 1 {
            self.update_counters();
        }
        self.step += 1;
        if self.step == 4 {
            if self.enable_irq {
                //self.interrupts.assert_irq();
            }
            self.step = 0;
        }
    }

    fn update_by_sequence_mode1(&mut self) {
        if self.step % 2 == 0 {
            self.update_counters();
        }
        self.step += 1;
        if self.step == 5 {
            self.step = 0;
        } else {
            self.update_envelope();
        }
    }

    fn update_counters(&mut self) {
        self.squares.0.update_counters();
        self.squares.1.update_counters();
        self.triangle.update_counter();
        self.noise.update_counter();
    }

    fn update_envelope(&mut self) {
        self.squares.0.update_envelope();
        self.squares.1.update_envelope();
        self.noise.update_envelope();
    }
}
