use super::constants::*;
use nes::types::{Addr, Data};

#[derive(Debug)]
pub struct Square {
    index: usize,
    sweep_unit_counter: usize,
    length_counter: usize,
    is_length_counter_enable: bool,
    sweep_unit_divider: usize,
    frequency: usize,
    sweep_shift_amount: usize,
    is_sweep_enabled: bool,
    sweep_mode: bool,
    divider_for_frequency: usize,
    envelope_loop_enable: bool,
    envelope_generator_counter: usize,
    envelope_rate: usize,
    envelope_volume: usize,
    envelope_enable: bool,
    enable: bool,
    playing: bool,
}

extern "C" {
    // fn start_oscillator(index: usize);
    // fn stop_oscillator(index: usize);
    // // fn close_oscillator(index: usize);
    // fn set_oscillator_frequency(index: usize, freq: usize);
    // fn change_oscillator_frequency(index: usize, freq: usize);
    // fn set_oscillator_volume(index: usize, volume: f32);
    // fn set_oscillator_pulse_width(index: usize, width: f32);
}

impl Square {
    pub fn new(index: usize) -> Self {
        Square {
            index,
            sweep_unit_counter: 0,
            length_counter: 0,
            sweep_unit_divider: 1,
            frequency: 0,
            sweep_shift_amount: 0,
            is_sweep_enabled: false,
            sweep_mode: false,
            divider_for_frequency: 1,
            envelope_loop_enable: false,
            envelope_generator_counter: 0,
            envelope_rate: 0x0F,
            envelope_volume: 0,
            envelope_enable: false,
            is_length_counter_enable: false,
            enable: false,
            playing: false,
        }
    }

    fn get_volume(&self) -> f32 {
        let vol = if self.envelope_enable {
            self.envelope_volume
        } else {
            self.envelope_rate
        };
        vol as f32 / (GROBAL_GAIN)
    }

    // fn stop_oscillator(&mut self) {
    //     unsafe {
    //         stop_oscillator(self.index);
    //     };
    // }

    // Length counter
    // When clocked by the frame counter, the length counter is decremented except when:
    // The length counter is 0, or The halt flag is set
    pub fn update_counters(&mut self) {
        if self.is_length_counter_enable && self.length_counter > 0 {
            self.length_counter -= 1;
            if self.length_counter == 0 {
                self.stop();
            }
        }

        if !self.is_sweep_enabled || !self.playing {
            return;
        };

        self.sweep_unit_counter += 1;
        if self.sweep_unit_counter % self.sweep_unit_divider == 0 {
            self.sweep_unit_counter = 0;
            // INFO:
            // sweep mode 0 : newPeriod = currentPeriod - (currentPeriod >> N)
            // sweep mode 1 : newPeriod = currentPeriod + (currentPeriod >> N)
            if self.sweep_mode {
                self.divider_for_frequency = self.divider_for_frequency
                    - (self.divider_for_frequency >> self.sweep_shift_amount);
            } else {
                self.divider_for_frequency = self.divider_for_frequency
                    + (self.divider_for_frequency >> self.sweep_shift_amount);
            };
            if self.divider_for_frequency > 0x7FF {
                self.stop();
            } else if self.divider_for_frequency < 8 {
                self.stop();
            }
            self.update_frequency();
            self.change_frequency();
        }
    }

    pub fn enable(&mut self) {
        self.enable = true;
        if !self.frequency != 0 {
            self.start();
        }
    }

    pub fn disable(&mut self) {
        self.enable = false;
        self.stop();
    }

    pub fn start(&mut self) {
        if !self.playing {
            self.playing = true;
            // unsafe {
            //     start_oscillator(self.index);
            //     set_oscillator_frequency(self.index, self.frequency);
            // };
        } else {
            self.change_frequency();
        }
    }

    pub fn stop(&mut self) {
        if self.playing {
            self.playing = false;
            // unsafe {
            //     stop_oscillator(self.index);
            // };
        }
    }

    pub fn get_pulse_width(&self, duty: usize) -> f32 {
        match duty {
            0x00 => 0.125,
            0x01 => 0.25,
            0x02 => 0.5,
            0x03 => 0.75,
            _ => 0.0,
        }
    }

    pub fn has_count_end(&self) -> bool {
        self.length_counter == 0
    }

    pub fn update_envelope(&mut self) {
        self.envelope_generator_counter -= 1;
        if self.envelope_generator_counter <= 0 {
            self.envelope_generator_counter = self.envelope_rate;
            if self.envelope_volume > 0 {
                self.envelope_volume -= 1;
            } else {
                self.envelope_volume = if self.envelope_loop_enable {
                    0x0F
                } else {
                    0x00
                };
            }
        }
        // unsafe {
        //     set_oscillator_volume(self.index, self.get_volume());
        // };
    }

    fn change_frequency(&self) {
        // unsafe {
        //     change_oscillator_frequency(self.index, self.frequency);
        // }
    }

    // fn reset(&mut self) {
    //     self.length_counter = 0;
    //     self.is_length_counter_enable = false;
    // }

    pub fn write(&mut self, addr: Addr, data: Data) {
        match addr {
            0x00 => {
                self.envelope_enable = data & 0x10 == 0;
                self.envelope_rate = data as usize & 0xF;
                self.envelope_loop_enable = (data & 0x10) != 0;
                let duty = (data >> 6) & 0x3;
                self.is_length_counter_enable = data & 0x20 == 0x00;
                // unsafe {
                //     set_oscillator_volume(self.index, self.get_volume());
                //     set_oscillator_pulse_width(self.index, self.get_pulse_width(duty as usize));
                // }
            }
            0x01 => {
                // Sweep
                self.is_sweep_enabled = data & 0x80 == 0x80;
                self.sweep_unit_divider = ((data as usize >> 4) & 0x07) + 1;
                self.sweep_mode = data & 0x08 == 0x08;
                self.sweep_shift_amount = data as usize & 0x07;
            }
            0x02 => {
                self.divider_for_frequency = (self.divider_for_frequency & 0x700) | data as usize;
                self.update_frequency();
                self.change_frequency();
            }
            0x03 => {
                // Programmable timer, length counter
                self.divider_for_frequency &= 0xFF;
                self.divider_for_frequency |= (data as usize & 0x7) << 8;
                if self.is_length_counter_enable {
                    self.length_counter = COUNTER_TABLE[(data & 0xF8) as usize >> 3] as usize / 2;
                }
                self.update_frequency();
                self.sweep_unit_counter = 0;
                // envelope
                self.envelope_generator_counter = self.envelope_rate;
                self.envelope_volume = 0x0F;
                if self.enable {
                    self.start();
                }
            }
            _ => (),
        }
    }

    fn update_frequency(&mut self) {
        self.frequency = CPU_CLOCK / ((self.divider_for_frequency + 1) * 16) as usize;
    }
}
