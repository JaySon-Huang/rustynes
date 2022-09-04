extern crate log;
extern crate regex;

mod apu;
mod bus;
mod cpu;
mod cpu_registers;
mod dma;
mod helper;
mod keypad;
mod mmc;
mod parser;
mod ppu;
mod ram;
mod renderer;
mod rom;
mod types;

pub use self::keypad::*;
pub use self::ppu::background;
pub use self::ppu::Tile;
pub use self::ppu::{Sprite, SpritePosition, SpriteWithCtx};
pub use self::renderer::*;

use self::apu::*;
use self::bus::cpu_bus;
use self::dma::*;
use self::mmc::*;
use self::ppu::*;
use self::ram::Ram;
use self::rom::Rom;
use self::types::{Addr, Data};

const DMA_CYCLES: u16 = 514;

#[derive(Debug)]
pub struct Context {
    ppu: Ppu,
    program_rom: Rom,
    work_ram: Ram,
    cpu_registers: cpu_registers::Registers,
    keypad: Keypad,
    dma: Dma,
    apu: Apu,
    nmi: bool,
    renderer: Renderer,
    mmc: Mmc,
}

pub fn reset(ctx: &mut Context) {
    let mut cpu_bus = cpu_bus::Bus::new(
        &ctx.program_rom,
        &mut ctx.work_ram,
        &mut ctx.ppu,
        &mut ctx.apu,
        &mut ctx.keypad,
        &mut ctx.dma,
        &mut ctx.mmc,
    );
    cpu::reset(&mut ctx.cpu_registers, &mut cpu_bus);
}

fn reset_with_addr(ctx: &mut Context, addr: Addr) {
    let mut cpu_bus = cpu_bus::Bus::new(
        &ctx.program_rom,
        &mut ctx.work_ram,
        &mut ctx.ppu,
        &mut ctx.apu,
        &mut ctx.keypad,
        &mut ctx.dma,
        &mut ctx.mmc,
    );
    cpu::reset_with_addr(&mut ctx.cpu_registers, &mut cpu_bus, addr);
}

pub fn run(ctx: &mut Context, key_state: u8) {
    ctx.keypad.update(key_state);
    loop {
        let cycle: u16 = if ctx.dma.should_run() {
            ctx.dma.run(&ctx.work_ram, &mut ctx.ppu);
            DMA_CYCLES
        } else {
            let mut cpu_bus = cpu_bus::Bus::new(
                &ctx.program_rom,
                &mut ctx.work_ram,
                &mut ctx.ppu,
                &mut ctx.apu,
                &mut ctx.keypad,
                &mut ctx.dma,
                &mut ctx.mmc,
            );
            cpu::step(&mut ctx.cpu_registers, &mut cpu_bus, &mut ctx.nmi) as u16
        };
        ctx.apu.run(cycle);
        let is_ready = ctx.ppu.run((cycle * 3) as usize, &mut ctx.nmi, &ctx.mmc);
        if is_ready {
            if ctx.ppu.background.0.len() != 0 {
                ctx.renderer.render(&ctx.ppu.background.0, &ctx.ppu.sprites);
            }
            break;
        }
    }
}

pub fn get_render_buf(ctx: &mut Context) -> &Vec<u8> {
    ctx.renderer.get_buf()
}

impl Context {
    pub fn new(buf: &mut [Data]) -> Self {
        let cassette = parser::parse(buf);
        Context {
            cpu_registers: cpu_registers::Registers::new(),
            program_rom: Rom::new(cassette.program_rom),
            ppu: Ppu::new(
                cassette.character_ram,
                PpuConfig {
                    is_horizontal_mirror: cassette.is_horizontal_mirror,
                },
            ),
            work_ram: Ram::new(vec![0; 0x0800]),
            keypad: Keypad::new(),
            dma: Dma::new(),
            apu: Apu::new(),
            nmi: false,
            mmc: Mmc::new(cassette.mapper, 0),
            renderer: Renderer::new(),
        }
    }
}

#[cfg(test)]
mod tests {

    use std::io::BufRead;

    use crate::nes::cpu_registers::CpuRegisters;

    use super::*;

    struct NesTestLog {
        // program counter
        pc: u16,
        // CPU op code
        opbytes: String,
        // CPU op code in assembly language
        // (along with address)
        _instruction: String,
        // CPU registers except PC
        regs: String,
        // CPU & PPU clock cycles
        _ppu: String,
        _cycle: u32,
    }

    impl NesTestLog {
        fn new(line: &String) -> Self {
            let pc = &line[0..4];
            let opbytes = &line[6..14];
            let instruction = &line[15..48];
            lazy_static! {
                // static ref RE: Regex = Regex::new(r"^A:(?P<A>[0-9A-F]{2}) X:(?P<X>[0-9A-F]{2}) Y:(?P<Y>[0-9A-F]{2}) P:(?P<P>[0-9A-F]{2}) SP:(?P<SP>[0-9A-F]{2}) PPU:(?P<PPU>[0-9 ]+,[0-9 ]+) CYC:(?P<CYC>[0-9]+)$").unwrap();
                static ref RE: regex::Regex = regex::Regex::new(r"^(?P<regs>.+) PPU:(?P<PPU>[0-9 ]+,[0-9 ]+) CYC:(?P<CYC>[0-9]+)$").unwrap();
            };
            let caps = RE.captures(&line[48..]).unwrap();
            Self {
                pc: u16::from_str_radix(pc, 16).unwrap(),
                opbytes: opbytes.trim().to_string(),
                _instruction: instruction.trim().to_string(),
                regs: caps["regs"].to_string(),
                _ppu: caps["PPU"].to_string(),
                _cycle: caps["CYC"].parse().unwrap(),
            }
        }
    }

    #[test]
    // wget "http://nickmass.com/images/nestest.nes" -O resources/nestest.nes
    // wget "https://www.qmtpro.com/~nes/misc/nestest.log" -O resources/nestest.log
    fn test_run_nestest() {
        let mut test_rom = {
            let filename = "roms/nestest.nes";
            std::fs::read(filename).unwrap()
        };
        let result_lines = {
            let filename = "roms/nestest.log";
            let file = std::fs::File::open(filename).unwrap();
            std::io::BufReader::new(file).lines().enumerate()
        };

        let mut ctx = Context::new(&mut test_rom);

        // reset registers
        const VECTOR_TEST: Addr = 0xC000;

        reset_with_addr(&mut ctx, VECTOR_TEST);

        let mut cpu_bus = cpu_bus::Bus::new(
            &ctx.program_rom,
            &mut ctx.work_ram,
            &mut ctx.ppu,
            &mut ctx.apu,
            &mut ctx.keypad,
            &mut ctx.dma,
            &mut ctx.mmc,
        );

        for (lineno, line_) in result_lines {
            let lineno = lineno + 1;
            let line = line_.unwrap();
            log::debug!("{}", line);
            let expect_res = NesTestLog::new(&line);
            assert_eq!(
                ctx.cpu_registers.get_PC(),
                expect_res.pc,
                "@ lineno={}, line={}",
                lineno,
                line
            );
            // assert_eq!(
            //     debug.format_bytes(),
            //     expect_res.opbytes,
            //     "@ lineno={}, line={}",
            //     lineno,
            //     line
            // );
            assert_eq!(
                ctx.cpu_registers.to_string(),
                expect_res.regs,
                "@ lineno={}, line={}",
                lineno,
                line
            );
            cpu::step(&mut ctx.cpu_registers, &mut cpu_bus, &mut ctx.nmi);
        }
    }
}
