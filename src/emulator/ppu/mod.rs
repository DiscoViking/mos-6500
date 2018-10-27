

use emulator::memory;

pub struct Pixel {
    r: u8,
    g: u8,
    b: u8,
}

pub trait VideoOut {
    fn emit(&mut self, p: Pixel);
}

pub struct PPU {
    // Device to output rendered pixels to.
    output: Box<VideoOut>,

    // PPU memory is laid out like so:
    // $0000-$0FFF = pattern table 0
    // $1000-$1FFF = pattern table 1
    // $2000-$23FF = name table 0
    // $2400-$27FF = name table 0
    // $2800-$2BFF = name table 0
    // $2C00-$2FFF = name table 0
    // $3000-$3EFF = mirrors of $2000-$2EFF
    // $3F00-$3F1F = palette RAM indexes
    // $3F20-$3FFF = mirrors of $3F00-$3F1F
    memory: memory::Manager,

    // -- Background State --

    // VRAM address.
    v: u16,

    // Temporary VRAM address.
    t: u16,

    // Fine X Scroll.
    fine_x: u8,

    // First/second write toggle.
    is_first_write: bool,

    // Two 16-bit shift registers containing bitmap data for 2 tiles.
    // Every 8 cycles the data for the next tile is loaded into the upper 8 bits of the register,
    // meanwhile the pixel to render is fetched from the lower 8 bits.
    tile_register_1: u16,
    tile_register_2: u16,

    // Two 8-bit shift registers containing the palette attributes for the lower 8 pixels of the
    // 16-bit register.
    // These registers are fed by a latch which contains the palette attribute for the next tile.
    // Every 8 cycles the latch is loaded with the attribute for the next tile.
    attribute_register_1: u8,
    attribute_register_2: u8,

    // -- Sprite State --

    // In addition to its main memory, the PPU has 256 bytes of memory known as OAM which determines how sprites are
    // rendered.
    // $00-$0C = Sprite Y coordinate
    // $01-$0D = Sprite tile #
    // $02-$0E = Sprite attribute
    // $03-$0F = Sprite X coordinate
    // TODO: What does this actually mean?
    oam: memory::Manager,

    // Secondary OAM holds 8 sprites to be rendered on the current scanline.
    secondary_oam: memory::Manager,

    // Eight pairs of 8-bit shift registers to hold the bitmap data for 8 sprites to be rendered on
    // the current scanline.

    // Eight latches containing the attribute bytes for the 8 sprites.

    // Eight counters containing the X positions for the 8 sprites.

    // --- Counters for tracking the current rendering stage.

    // There are 262 scanlines in total. 0-239 are visible, 240-260 occur durng vblank, and 261 is
    // idle.
    scanline: u16,

    // Each scanline takes 341 cycles to render.
    cycle: u16,

    // Rendering can be disabled, which changes the operation of the PPU.
    rendering_is_enabled: bool,
}

impl PPU {
    // Returns how many PPU cycles the tick took.
    pub fn tick(&mut self) -> u16 {
        let cycles = match self.scanline {
            0 ... 239 | 261 => self.tick_render(),
            240 => self.tick_idle_scanline(),
            241 => self.tick_vblank(),
            _ => panic!("Scanline index should never exceed 261.  Got {}.", self.scanline),
        };

        self.cycle = self.cycle + cycles;

        if self.cycle > 341 {
            panic!("Cycle index should never exceed 341.  Got: {}.", self.cycle);
        }

        if self.cycle == 341 {
            self.cycle = 0;
            self.scanline = (self.scanline + 1) % 262;
        }

        cycles
    }

    fn tick_render(&mut self) -> u16 {
        // Rendering stages.
        let cycles = match self.cycle {
            // Cycle 0 is an idle cycle.
            0 => self.tick_idle_cycle(),

            // The data for each tile is fetched durnig this phase.
            // This where the actual pixels for the scanline are output.
            1 ... 256 => self.tick_render_cycle(),

            // The tile data for the sprites on the next scanline are fetched during this phase.
            257 ... 320 => self.tick_sprite_fetch_cycle(),

            // This is where the first two tiles of the next scanline are fetched and loaded into
            // the shift registers.
            321 ... 336 => self.tick_prefetch_tiles_cycle(),

            // Finally, here two bytes are fetched, but the purpose is unknown.
            337 ... 340 => self.tick_unknown_fetch(),

            _ => panic!("PPU cycle index should never exceed 341.  Got {}.", self.cycle),
        };

        // Scrolling.
        self.handle_scrolling();

        cycles
    }

    fn tick_idle_scanline(&mut self) -> u16 {
        // PPU does nothing on the idle scanline.
        // Just idle for 341 cycles.
        341
    }

    fn tick_vblank(&mut self) -> u16 {
        if self.scanline == 241 && self.cycle == 1 {
            // TODO: Set VBlank flag.
        }
        // Otherwise idle.
        1
    }

    fn tick_idle_cycle(&mut self) -> u16 {
        // PPU does nothing during idle cycle.
        1
    }

    fn tick_render_cycle(&mut self) -> u16 {
        1
    }

    fn tick_sprite_fetch_cycle(&mut self) -> u16 {
        1
    }

    fn tick_prefetch_tiles_cycle(&mut self) -> u16 {
        1
    }

    fn tick_unknown_fetch(&mut self) -> u16 {
        1
    }

    // --- SCROLLING
    // Put all scrolling logic in one place.
    fn handle_scrolling(&mut self) {
        // No scrolling happens if rendering is disabled.
        if !self.rendering_is_enabled {
            return;
        }

        // If rendering is enabled, on dot 256 of each scanline, the PPU increments y position.
        if self.cycle == 256 {
            self.increment_y();
        }

        // If rendering is enabled, on dot 257 of each scanline, copy all horizontal bits from t to v.
        if self.cycle == 257 {
            let horizontal_bitmask = 0b0000100_00011111;
            self.v = self.v & !horizontal_bitmask;
            self.v = self.v | (self.t & horizontal_bitmask);
        }

        // If rendering is enabled, between dots 280 to 304 of the pre-render scanline, the PPU repeatedly copies the
        // vertical bits from t to v.
        if self.scanline == 261 && self.cycle >= 280 && self.cycle <= 304 {
            let vertical_bitmask = 0b1111011_11100000;
            self.v = self.v & !vertical_bitmask;
            self.v = self.v | (self.t & vertical_bitmask);
        }

        // Between dot 328 of a scanline, and 256 of the next scanline, x scroll is incremented
        // on every multiple of 8 dots except 0.  i.e. 328, 336, 8, 16, ..., 256.
        if ((self.cycle > 0 && self.cycle <= 256) || self.cycle >= 328) && (self.cycle % 8 == 0) {
            self.increment_coarse_x();
        }
    }

    // During rendering the VRAM address v is laid out like so:
    // yyy NN YYYYY XXXXX
    // ||| || ||||| +++++-- coarse X scroll
    // ||| || +++++-------- coarse Y scroll
    // ||| ++-------------- nametable select
    // +++----------------- fine Y scroll
    //
    // Here are some convenience methods to pull out these values.
    fn fine_y_scroll(&self) -> u16 {
        ((self.v >> 12) & 0b111) as u16
    }

    fn nametable_select(&self) -> u16 {
        ((self.v >> 10) & 0b11) as u16
    }

    fn coarse_y_scroll(&self) -> u16 {
        ((self.v >> 5) & 0b11111) as u16
    }

    fn coarse_x_scroll(&self) -> u16 {
        (self.v & 0b11111) as u16
    }

    // Scrolling is complex, so split out the logic here.
    fn increment_coarse_x(&mut self) {
        if self.coarse_x_scroll() == 31 {
            self.v &= !0x001F;  // Coarse X = 0.
            self.v ^= 0x0400;  // Switch horizontal nametable.
        } else {
            self.v += 1;  // Increment coarse X.
        }
    }

    fn increment_y(&mut self) {
        if self.fine_y_scroll() < 7 {
            self.v += 0x100;  // Increment fine Y.
        } else {
            self.v &= !0x700;  // Fine Y = 0.
            let mut coarse_y = self.coarse_y_scroll();
            if coarse_y == 29 {
                coarse_y = 0;
                self.v ^= 0x0800;  // Switch vertical nametable.
            } else if coarse_y == 31 {
                coarse_y = 0;
            } else {
                coarse_y += 1;
            }

            self.v = (self.v & !0x03E0) | ((coarse_y as u16) << 5);  // Put coarse_y back into v.
        }
    }

    // And then methods to load the tile and attribute addresses to load next.
    fn tile_address(&self) -> u16 {
        0x2000 | (self.v & 0x0FFF)
    }

    fn attribute_address(&self) -> u16 {
        // This formula copied from nesdev wiki.  I should try to understand it later.
        0x23C0 | self.nametable_select() | ((self.v >> 4) & 0x38) | ((self.v >> 2) & 0x07)
    }
}