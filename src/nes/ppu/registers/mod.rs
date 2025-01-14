mod oam;
mod ppu_addr;
mod ppu_data;
mod ppu_scroll;

use super::super::types::{Addr, Data};
use super::super::Ram;
use super::palette::*;
use super::PpuCtx;
// use super::super::helper::*;
use self::oam::Oam;
use self::ppu_addr::PpuAddr;
use self::ppu_data::PpuData;
use self::ppu_scroll::PpuScroll;

#[derive(Debug)]
pub struct Registers {
    pub ppu_ctrl1: Data,
    pub ppu_ctrl2: Data,
    pub ppu_status: Data,
    pub oam: Oam,
    pub ppu_addr: PpuAddr,
    pub ppu_data: PpuData,
    pub ppu_scroll: PpuScroll,
}

// PPU power up state
// see. https://wiki.nesdev.com/w/index.php/PPU_power_up_state
//
// Memory map
/*
| addr           |  description               |
+----------------+----------------------------+
| 0x0000-0x0FFF  |  Pattern table#0           |
| 0x1000-0x1FFF  |  Pattern table#1           |
| 0x2000-0x23BF  |  Name table                |
| 0x23C0-0x23FF  |  Attribute table           |
| 0x2400-0x27BF  |  Name table                |
| 0x27C0-0x27FF  |  Attribute table           |
| 0x2800-0x2BBF  |  Name table                |
| 0x2BC0-0x2BFF  |  Attribute table           |
| 0x2C00-0x2FBF  |  Name Table                |
| 0x2FC0-0x2FFF  |  Attribute Table           |
| 0x3000-0x3EFF  |  mirror of 0x2000-0x2EFF   |
| 0x3F00-0x3F0F  |  background Palette        |
| 0x3F10-0x3F1F  |  sprite Palette            |
| 0x3F20-0x3FFF  |  mirror of 0x3F00-0x3F1F   |
*/

pub trait PpuRegisters {
    fn read<P: PaletteRam>(&mut self, addr: Addr, ctx: &mut PpuCtx<P>) -> Data;

    fn write<P: PaletteRam>(&mut self, addr: Addr, data: Data, ctx: &mut PpuCtx<P>);

    fn is_sprite_8x8(&self) -> bool;

    fn clear_vblank(&mut self);

    fn set_vblank(&mut self);

    fn set_sprite_hit(&mut self);

    fn clear_sprite_hit(&mut self);

    fn get_sprite_table_offset(&self) -> Addr;

    fn get_background_table_offset(&self) -> Addr;

    fn get_ppu_addr_increment_value(&self) -> usize;

    fn get_name_table_id(&self) -> Data;

    fn get_scroll_x(&self) -> Data;

    fn get_scroll_y(&self) -> Data;

    fn is_irq_enable(&self) -> bool;

    fn is_background_enable(&self) -> bool;

    fn is_sprite_enable(&self) -> bool;

    fn is_background_masked(&self) -> bool;

    fn is_sprite_masked(&self) -> bool;
}

impl Registers {
    pub fn new() -> Self {
        Registers {
            ppu_ctrl1: 0,
            ppu_ctrl2: 0,
            ppu_status: 0,
            oam: Oam::new(),
            ppu_addr: PpuAddr::new(),
            ppu_data: PpuData::new(),
            ppu_scroll: PpuScroll::new(),
        }
    }

    /*
    |  status register 0x2002
    | bit  | description                                 |
    +------+---------------------------------------------+
    | 7    | 1: VBlank clear by reading this register    |
    | 6    | 1: sprite hit                               |
    | 5    | 0: less than 8, 1: 9 or more                |
    | 4-0  | invalid                                     |
    |      | bit4 VRAM write flag [0: success, 1: fail]  |
    */
    fn read_status(&mut self) -> Data {
        let data = self.ppu_status;
        self.ppu_scroll.enable_x();
        self.clear_vblank();
        self.clear_sprite_hit();
        self.ppu_addr.reser_latch();
        data
    }

    fn write_oam_addr(&mut self, data: Data) {
        self.oam.write_addr(data);
    }

    fn write_oam_data(&mut self, data: Data, sprite_ram: &mut Ram) {
        self.oam.write_data(sprite_ram, data);
    }

    fn write_ppu_addr(&mut self, data: Data) {
        self.ppu_addr.write(data);
    }

    fn read_ppu_data<P: PaletteRam>(&mut self, vram: &Ram, cram: &Ram, palette: &P) -> Data {
        let addr = self.ppu_addr.get();
        let data = self.ppu_data.read(vram, cram, addr, palette);
        let v = self.get_ppu_addr_increment_value() as u16;
        self.ppu_addr.update(v);
        data
    }

    fn write_ppu_data<P: PaletteRam>(
        &mut self,
        data: Data,
        vram: &mut Ram,
        cram: &mut Ram,
        palette: &mut P,
    ) {
        let addr = self.ppu_addr.get();
        self.ppu_data.write(vram, cram, addr, data, palette);
        let v = self.get_ppu_addr_increment_value() as u16;
        self.ppu_addr.update(v);
    }
}

impl PpuRegisters for Registers {
    fn clear_vblank(&mut self) {
        self.ppu_status &= 0x7F;
    }

    fn set_vblank(&mut self) {
        self.ppu_status |= 0x80;
    }

    fn is_sprite_8x8(&self) -> bool {
        self.ppu_ctrl1 & 0x20 != 0x20
    }

    fn clear_sprite_hit(&mut self) {
        self.ppu_status &= 0xbF;
    }

    fn set_sprite_hit(&mut self) {
        self.ppu_status |= 0x40;
    }

    fn get_ppu_addr_increment_value(&self) -> usize {
        if self.ppu_ctrl1 & 0x04 == 0x04 {
            32
        } else {
            1
        }
    }

    fn is_irq_enable(&self) -> bool {
        self.ppu_ctrl1 & 0x80 == 0x80
    }

    fn get_sprite_table_offset(&self) -> Addr {
        if self.ppu_ctrl1 & 0x08 == 0x08 {
            0x1000
        } else {
            0x0000
        }
    }

    fn get_background_table_offset(&self) -> Addr {
        if self.ppu_ctrl1 & 0x10 == 0x10 {
            0x1000
        } else {
            0x0000
        }
    }

    fn get_name_table_id(&self) -> Data {
        self.ppu_ctrl1 & 0x03
    }

    fn get_scroll_x(&self) -> Data {
        self.ppu_scroll.get_x()
    }

    fn get_scroll_y(&self) -> Data {
        self.ppu_scroll.get_y()
    }

    fn is_background_enable(&self) -> bool {
        self.ppu_ctrl2 & 0x08 == 0x08
    }

    fn is_sprite_enable(&self) -> bool {
        self.ppu_ctrl2 & 0x10 == 0x10
    }

    fn is_background_masked(&self) -> bool {
        self.ppu_ctrl2 & 0x02 == 0x02
    }

    fn is_sprite_masked(&self) -> bool {
        self.ppu_ctrl2 & 0x04 == 0x04
    }

    fn read<P: PaletteRam>(&mut self, addr: Addr, ctx: &mut PpuCtx<P>) -> Data {
        match addr {
            0x0002 => self.read_status(),
            0x0004 => self.oam.read_data(&ctx.sprite_ram),
            0x0007 => self.read_ppu_data(&ctx.vram, &ctx.cram, &ctx.palette),
            _ => 0,
        }
    }

    fn write<P: PaletteRam>(&mut self, addr: Addr, data: Data, ctx: &mut PpuCtx<P>) {
        match addr {
            /*
              Control Register1 0x2000
            | bit  | description                                 |
            +------+---------------------------------------------+
            |  7   | Assert NMI when VBlank 0: disable, 1:enable |
            |  6   | PPU master/slave, always 1                  |
            |  5   | Sprite size 0: 8x8, 1: 8x16                 |
            |  4   | Bg pattern table 0:0x0000, 1:0x1000         |
            |  3   | sprite pattern table 0:0x0000, 1:0x1000     |
            |  2   | PPU memory increment 0: +=1, 1:+=32         |
            |  1-0 | Name table 0x00: 0x2000                     |
            |      |            0x01: 0x2400                     |
            |      |            0x02: 0x2800                     |
            |      |            0x03: 0x2C00                     |
            */
            0x0000 => self.ppu_ctrl1 = data,
            /*
               Control Register2 0x2001
             | bit  | description                                 |
             +------+---------------------------------------------+
             |  7-5 | Background color  0x00: Black               |
             |      |                   0x01: Green               |
             |      |                   0x02: Blue                |
             |      |                   0x04: Red                 |
             |  4   | Enable sprite                               |
             |  3   | Enable background                           |
             |  2   | Sprite mask       render left end           |
             |  1   | Background mask   render left end           |
             |  0   | Display type      0: color, 1: mono         |
            */
            // If either of bits 3 or 4 is enabled, at any time outside of the vblank interval
            // the PPU will be making continual use to the PPU address and data bus to fetch tiles
            // to render,as well as internally fetching sprite data from the OAM.
            // If you wish to make changes to PPU memory outside of vblank (via $2007),
            // you must set both of these bits to 0 to disable rendering and prevent conflicts.
            // Disabling rendering (clear both bits 3 and 4) during a visible part of the frame can be problematic.
            // It can cause a corruption of the sprite state, which will display incorrect sprite data on the next frame.
            // (See: Errata) It is, however, perfectly fine to mask sprites but leave the background on (set bit 3, clear bit 4) at any time in the frame.
            // Sprite 0 hit does not trigger in any area where the background or sprites are hidden.
            0x0001 => self.ppu_ctrl2 = data,
            0x0003 => self.write_oam_addr(data),
            0x0004 => self.write_oam_data(data, &mut ctx.sprite_ram),
            0x0005 => self.ppu_scroll.write(data),
            0x0006 => self.write_ppu_addr(data),
            0x0007 => self.write_ppu_data(data, &mut ctx.vram, &mut ctx.cram, &mut ctx.palette),
            _ => (),
        }
    }
}
