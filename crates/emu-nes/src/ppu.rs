/// NES PPU (Picture Processing Unit) Implementation
/// 
/// The PPU generates the video signal for the NES. It has:
/// - 256x240 pixel resolution
/// - 64 colors (from a palette of 512)
/// - 2KB of VRAM for nametables (background)
/// - 256 bytes of OAM for sprites (64 sprites, 4 bytes each)
/// - Pattern tables (CHR-ROM/RAM) for tile graphics
/// - Scrolling and sprite capabilities

/// PPU registers (memory-mapped to CPU address space $2000-$2007)
/// 
/// The PPU has 8 registers accessible to the CPU:
/// - $2000: PPUCTRL   - PPU control register
/// - $2001: PPUMASK   - PPU mask register (rendering options)
/// - $2002: PPUSTATUS - PPU status register (read-only)
/// - $2003: OAMADDR   - OAM address port
/// - $2004: OAMDATA   - OAM data port
/// - $2005: PPUSCROLL - Scrolling position register (write x2)
/// - $2006: PPUADDR   - PPU address register (write x2)
/// - $2007: PPUDATA   - PPU data port

use bitflags::bitflags;

bitflags! {
    /// PPUCTRL register ($2000) - Controls PPU operation
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct PpuCtrl: u8 {
        /// Base nametable address (0 = $2000, 1 = $2400, 2 = $2800, 3 = $2C00)
        const NAMETABLE_X        = 0b00000001;
        const NAMETABLE_Y        = 0b00000010;
        /// VRAM address increment per CPU read/write (0: +1 across, 1: +32 down)
        const VRAM_INCREMENT     = 0b00000100;
        /// Sprite pattern table address (0: $0000, 1: $1000)
        const SPRITE_PATTERN     = 0b00001000;
        /// Background pattern table address (0: $0000, 1: $1000)
        const BG_PATTERN         = 0b00010000;
        /// Sprite size (0: 8x8, 1: 8x16)
        const SPRITE_SIZE        = 0b00100000;
        /// PPU master/slave select (unused on NES)
        const MASTER_SLAVE       = 0b01000000;
        /// Generate NMI at start of vblank
        const NMI_ENABLE         = 0b10000000;
    }
}

bitflags! {
    /// PPUMASK register ($2001) - Controls rendering options
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct PpuMask: u8 {
        /// Greyscale mode (0: normal, 1: greyscale)
        const GREYSCALE          = 0b00000001;
        /// Show background in leftmost 8 pixels
        const BG_LEFTMOST        = 0b00000010;
        /// Show sprites in leftmost 8 pixels
        const SPRITE_LEFTMOST    = 0b00000100;
        /// Show background
        const SHOW_BG            = 0b00001000;
        /// Show sprites
        const SHOW_SPRITES       = 0b00010000;
        /// Emphasize red
        const EMPHASIZE_RED      = 0b00100000;
        /// Emphasize green
        const EMPHASIZE_GREEN    = 0b01000000;
        /// Emphasize blue
        const EMPHASIZE_BLUE     = 0b10000000;
    }
}

bitflags! {
    /// PPUSTATUS register ($2002) - PPU status flags (read-only)
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct PpuStatus: u8 {
        /// Lower 5 bits are open bus (not implemented)
        const OPEN_BUS           = 0b00011111;
        /// Sprite overflow flag
        const SPRITE_OVERFLOW    = 0b00100000;
        /// Sprite 0 hit flag
        const SPRITE_ZERO_HIT    = 0b01000000;
        /// Vertical blank flag
        const VBLANK             = 0b10000000;
    }
}

/// PPU internal state
pub struct Ppu {
    /// PPUCTRL register ($2000)
    pub ctrl: PpuCtrl,
    /// PPUMASK register ($2001)
    pub mask: PpuMask,
    /// PPUSTATUS register ($2002)
    pub status: PpuStatus,
    /// OAM address register ($2003)
    pub oam_addr: u8,
    
    // Internal registers for scrolling (accessed via $2005 and $2006)
    /// Current VRAM address (15 bits)
    vram_addr: u16,
    /// Temporary VRAM address (15 bits)
    temp_vram_addr: u16,
    /// Fine X scroll (3 bits)
    fine_x: u8,
    /// Write latch (for $2005 and $2006, which need 2 writes)
    write_latch: bool,
    
    /// Read buffer for $2007 reads (reading is delayed by 1)
    read_buffer: u8,
    
    // VRAM (Video RAM)
    /// 2KB of VRAM for nametables (mirrored depending on cartridge)
    vram: [u8; 0x800],
    /// 32 bytes of palette RAM
    palette: [u8; 0x20],
    /// 256 bytes of Object Attribute Memory (OAM) for sprites
    oam: [u8; 0x100],
    
    /// Reference to CHR-ROM/RAM (from cartridge)
    chr_rom: Vec<u8>,
    
    // Rendering state
    /// Current scanline (0-261, where 261 is pre-render)
    scanline: u16,
    /// Current cycle within scanline (0-340)
    cycle: u16,
    /// Frame counter
    frame: u64,
    
    /// Framebuffer (256x240 pixels, each pixel is a palette index 0-63)
    framebuffer: Vec<u8>,
    
    /// NMI interrupt flag (signals CPU)
    pub nmi_interrupt: bool,
}

impl Ppu {
    /// Create a new PPU
    pub fn new() -> Self {
        Self {
            ctrl: PpuCtrl::empty(),
            mask: PpuMask::empty(),
            status: PpuStatus::empty(),
            oam_addr: 0,
            vram_addr: 0,
            temp_vram_addr: 0,
            fine_x: 0,
            write_latch: false,
            read_buffer: 0,
            vram: [0; 0x800],
            palette: [0; 0x20],
            oam: [0; 0x100],
            chr_rom: vec![0; 0x2000],
            scanline: 0,
            cycle: 0,
            frame: 0,
            framebuffer: vec![0; 256 * 240],
            nmi_interrupt: false,
        }
    }
    
    /// Load CHR-ROM from cartridge
    pub fn load_chr_rom(&mut self, chr_rom: Vec<u8>) {
        self.chr_rom = chr_rom;
    }
    
    /// Update CHR bank (for mappers with CHR banking)
    /// Copies 8KB from source at offset to CHR-ROM at address 0x0000
    pub fn load_chr_bank(&mut self, source: &[u8], bank: usize) {
        let offset = bank * 0x2000;
        let len = 0x2000.min(source.len().saturating_sub(offset));
        if len > 0 && offset < source.len() {
            self.chr_rom[0..len].copy_from_slice(&source[offset..offset + len]);
        }
    }
    
    /// Get framebuffer reference
    pub fn framebuffer(&self) -> &[u8] {
        &self.framebuffer
    }
    
    /// Debug: Read palette RAM directly (for testing)
    pub fn read_palette_direct(&self, addr: u16) -> u8 {
        self.palette[(addr & 0x1F) as usize]
    }
    
    /// Debug: Read nametable directly (for testing)
    pub fn read_nametable_direct(&self, addr: u16) -> u8 {
        self.read_vram_direct(addr)
    }
    
    /// Debug: Read CHR-ROM directly (for testing)
    pub fn read_chr_direct(&self, addr: u16) -> u8 {
        self.chr_rom.get(addr as usize).copied().unwrap_or(0)
    }
    
    /// Read from PPU register (CPU memory space $2000-$2007)
    pub fn read_register(&mut self, addr: u16) -> u8 {
        match addr & 0x07 {
            // $2000 PPUCTRL - write-only
            0 => 0,
            
            // $2001 PPUMASK - write-only
            1 => 0,
            
            // $2002 PPUSTATUS - read-only
            2 => {
                let status = self.status.bits();
                // Reading $2002 clears vblank flag and write latch
                self.status.remove(PpuStatus::VBLANK);
                self.write_latch = false;
                status
            }
            
            // $2003 OAMADDR - write-only
            3 => 0,
            
            // $2004 OAMDATA - read OAM data
            4 => self.oam[self.oam_addr as usize],
            
            // $2005 PPUSCROLL - write-only
            5 => 0,
            
            // $2006 PPUADDR - write-only
            6 => 0,
            
            // $2007 PPUDATA - read from VRAM
            7 => self.read_vram(),
            
            _ => unreachable!(),
        }
    }
    
    /// Write to PPU register (CPU memory space $2000-$2007)
    pub fn write_register(&mut self, addr: u16, value: u8) {
        match addr & 0x07 {
            // $2000 PPUCTRL
            0 => {
                self.ctrl = PpuCtrl::from_bits_truncate(value);
                // t: ...BA.. ........ = d: ......BA
                self.temp_vram_addr = (self.temp_vram_addr & 0xF3FF) | ((value as u16 & 0x03) << 10);
            }
            
            // $2001 PPUMASK
            1 => {
                self.mask = PpuMask::from_bits_truncate(value);
            }
            
            // $2002 PPUSTATUS - read-only
            2 => {}
            
            // $2003 OAMADDR
            3 => {
                self.oam_addr = value;
            }
            
            // $2004 OAMDATA - write OAM data
            4 => {
                self.oam[self.oam_addr as usize] = value;
                self.oam_addr = self.oam_addr.wrapping_add(1);
            }
            
            // $2005 PPUSCROLL - write scroll position (2 writes: X then Y)
            5 => {
                if !self.write_latch {
                    // First write: X scroll
                    self.fine_x = value & 0x07;
                    self.temp_vram_addr = (self.temp_vram_addr & 0xFFE0) | ((value as u16) >> 3);
                    self.write_latch = true;
                } else {
                    // Second write: Y scroll
                    self.temp_vram_addr = (self.temp_vram_addr & 0x8FFF) | (((value as u16) & 0x07) << 12);
                    self.temp_vram_addr = (self.temp_vram_addr & 0xFC1F) | (((value as u16) & 0xF8) << 2);
                    self.write_latch = false;
                }
            }
            
            // $2006 PPUADDR - write VRAM address (2 writes: high then low)
            6 => {
                if !self.write_latch {
                    // First write: high byte
                    self.temp_vram_addr = (self.temp_vram_addr & 0x00FF) | (((value as u16) & 0x3F) << 8);
                    self.write_latch = true;
                } else {
                    // Second write: low byte
                    self.temp_vram_addr = (self.temp_vram_addr & 0xFF00) | (value as u16);
                    self.vram_addr = self.temp_vram_addr;
                    self.write_latch = false;
                }
            }
            
            // $2007 PPUDATA - write to VRAM
            7 => {
                self.write_vram(value);
                // Auto-increment VRAM address
                let increment = if self.ctrl.contains(PpuCtrl::VRAM_INCREMENT) { 32 } else { 1 };
                self.vram_addr = self.vram_addr.wrapping_add(increment);
            }
            
            _ => unreachable!(),
        }
    }
    
    /// Read from PPU memory space ($0000-$3FFF)
    fn read_vram(&mut self) -> u8 {
        let addr = self.vram_addr & 0x3FFF;
        let result = self.read_buffer;
        
        // Read from appropriate memory region
        self.read_buffer = match addr {
            // Pattern tables (CHR-ROM/RAM)
            0x0000..=0x1FFF => self.chr_rom[addr as usize],
            
            // Nametables (VRAM)
            0x2000..=0x3EFF => {
                let mirror_addr = self.mirror_nametable(addr);
                self.vram[mirror_addr]
            }
            
            // Palette RAM (not buffered!)
            0x3F00..=0x3FFF => {
                let palette_addr = (addr - 0x3F00) & 0x1F;
                // Update buffer with nametable data instead
                let mirror_addr = self.mirror_nametable(addr);
                self.read_buffer = self.vram[mirror_addr];
                // Return palette data immediately
                return self.palette[palette_addr as usize];
            }
            
            _ => 0,
        };
        
        // Increment VRAM address
        self.increment_vram_addr();
        
        result
    }
    
    /// Write to PPU memory space ($0000-$3FFF)
    fn write_vram(&mut self, value: u8) {
        let addr = self.vram_addr & 0x3FFF;
        
        match addr {
            // Pattern tables (CHR-ROM/RAM)
            0x0000..=0x1FFF => {
                // Only writable if CHR-RAM
                if self.chr_rom.len() <= 0x2000 {
                    self.chr_rom[addr as usize] = value;
                }
            }
            
            // Nametables (VRAM)
            0x2000..=0x3EFF => {
                let mirror_addr = self.mirror_nametable(addr);
                self.vram[mirror_addr] = value;
            }
            
            // Palette RAM
            0x3F00..=0x3FFF => {
                let mut palette_addr = (addr - 0x3F00) & 0x1F;
                // Addresses $3F10/$3F14/$3F18/$3F1C are mirrors of $3F00/$3F04/$3F08/$3F0C
                if palette_addr >= 0x10 && palette_addr & 0x03 == 0 {
                    palette_addr -= 0x10;
                }
                self.palette[palette_addr as usize] = value;
            }
            
            _ => {}
        }
    }
    
    /// Mirror nametable address based on mirroring mode (horizontal for now)
    fn mirror_nametable(&self, addr: u16) -> usize {
        let addr = (addr - 0x2000) & 0x0FFF;
        // Horizontal mirroring: $2000=$2400, $2800=$2C00
        // For now, simple modulo
        (addr & 0x07FF) as usize
    }
    
    /// Increment VRAM address based on PPUCTRL increment flag
    fn increment_vram_addr(&mut self) {
        let increment = if self.ctrl.contains(PpuCtrl::VRAM_INCREMENT) {
            32 // Down
        } else {
            1 // Across
        };
        self.vram_addr = self.vram_addr.wrapping_add(increment) & 0x3FFF;
    }
    
    /// Tick the PPU by one cycle
    pub fn tick(&mut self) {
        // Visible scanlines: 0-239
        if self.scanline < 240 && self.is_rendering() {
            // Render pixel at current position
            if self.cycle > 0 && self.cycle <= 256 {
                self.render_pixel();
            }
        }
        
        // Advance cycle
        self.cycle += 1;
        
        // End of scanline
        if self.cycle > 340 {
            self.cycle = 0;
            self.scanline += 1;
            
            // End of frame
            if self.scanline > 261 {
                self.scanline = 0;
                self.frame += 1;
            }
        }
        
        // VBlank start (scanline 241, cycle 1)
        if self.scanline == 241 && self.cycle == 1 {
            self.status.insert(PpuStatus::VBLANK);
            if self.ctrl.contains(PpuCtrl::NMI_ENABLE) {
                self.nmi_interrupt = true;
            }
        }
        
        // VBlank end (pre-render scanline 261, cycle 1)
        if self.scanline == 261 && self.cycle == 1 {
            self.status.remove(PpuStatus::VBLANK);
            self.status.remove(PpuStatus::SPRITE_ZERO_HIT);
            self.status.remove(PpuStatus::SPRITE_OVERFLOW);
            self.nmi_interrupt = false;
        }
    }
    
    /// Render a single pixel at the current scanline/cycle position
    fn render_pixel(&mut self) {
        let x = (self.cycle - 1) as usize;
        let y = self.scanline as usize;
        
        if x >= 256 || y >= 240 {
            return;
        }
        
        let pixel_index = y * 256 + x;
        
        // Get background pixel
        let bg_pixel = if self.mask.contains(PpuMask::SHOW_BG) {
            self.get_background_pixel(x, y)
        } else {
            0 // Universal background color
        };
        
        // Get sprite pixel (to be implemented)
        let sprite_pixel = if self.mask.contains(PpuMask::SHOW_SPRITES) {
            self.get_sprite_pixel(x, y)
        } else {
            (0, false, false)
        };
        
        // Combine background and sprite with priority
        let palette_index = if sprite_pixel.1 && (sprite_pixel.2 || bg_pixel & 0x03 == 0) {
            // Sprite is visible and has priority (or BG is transparent)
            sprite_pixel.0
        } else {
            bg_pixel
        };
        
        self.framebuffer[pixel_index] = palette_index;
    }
    
    /// Get background pixel color at screen position (x, y)
    fn get_background_pixel(&self, x: usize, y: usize) -> u8 {
        // Apply scrolling using temp_vram_addr (set by $2005) and fine_x
        // temp_vram_addr layout: yyy NN YYYYY XXXXX
        //   yyy = fine Y (3 bits, pixel offset within tile)
        //   NN = nametable select (2 bits)
        //   YYYYY = coarse Y (5 bits, tile row 0-29)
        //   XXXXX = coarse X (5 bits, tile column 0-31)
        
        // Extract scroll components
        let coarse_x = (self.temp_vram_addr & 0x001F) as usize;
        let coarse_y = ((self.temp_vram_addr & 0x03E0) >> 5) as usize;
        let fine_y = ((self.temp_vram_addr & 0x7000) >> 12) as usize;
        let nametable_select = ((self.temp_vram_addr & 0x0C00) >> 10) as u16;
        
        // Calculate scrolled pixel position
        // Add current screen position to scroll offset
        let scroll_x = x + self.fine_x as usize + (coarse_x * 8);
        let scroll_y = y + fine_y + (coarse_y * 8);
        
        // Get tile coordinates
        let tile_x = (scroll_x / 8) % 32;
        let tile_y = (scroll_y / 8) % 30;
        
        // Get pixel within tile
        let pixel_x = scroll_x % 8;
        let pixel_y = scroll_y % 8;
        
        // Calculate nametable base (using base nametable from scroll registers)
        let nametable_base = 0x2000 | (nametable_select << 10);
        
        // Calculate nametable address for this tile
        let tile_addr = nametable_base + (tile_y * 32 + tile_x) as u16;
        let tile_index = self.read_vram_direct(tile_addr);
        
        // Get attribute byte (determines palette for 2x2 tile group)
        let attr_addr = nametable_base + 0x03C0 + ((tile_y / 4) * 8 + tile_x / 4) as u16;
        let attr_byte = self.read_vram_direct(attr_addr);
        
        // Extract 2-bit palette index for this tile
        let attr_shift = ((tile_y % 4) / 2) * 4 + ((tile_x % 4) / 2) * 2;
        let palette_high = (attr_byte >> attr_shift) & 0x03;
        
        // Get pattern table address (CHR-ROM)
        let pattern_table_base = if self.ctrl.contains(PpuCtrl::BG_PATTERN) {
            0x1000
        } else {
            0x0000
        };
        
        // Each tile is 16 bytes: 8 bytes for low bit plane, 8 bytes for high bit plane
        let tile_addr = pattern_table_base + (tile_index as u16) * 16;
        
        // Read bit planes for this pixel row
        let low_byte = self.chr_rom.get((tile_addr + pixel_y as u16) as usize).copied().unwrap_or(0);
        let high_byte = self.chr_rom.get((tile_addr + 8 + pixel_y as u16) as usize).copied().unwrap_or(0);
        
        // Extract pixel color (2 bits: high bit from high_byte, low bit from low_byte)
        let bit_pos = 7 - pixel_x;
        let pixel_low = (low_byte >> bit_pos) & 0x01;
        let pixel_high = (high_byte >> bit_pos) & 0x01;
        let pixel_value = (pixel_high << 1) | pixel_low;
        
        // Combine with palette index
        if pixel_value == 0 {
            // Transparent - use universal background color
            self.palette[0]
        } else {
            // Use background palette
            let palette_addr = (palette_high * 4 + pixel_value) as usize;
            self.palette[palette_addr]
        }
    }
    
    /// Get sprite pixel at screen position (x, y)
    /// Returns (palette_index, is_visible, has_priority)
    fn get_sprite_pixel(&self, x: usize, y: usize) -> (u8, bool, bool) {
        // Check all 64 sprites in OAM
        for sprite_idx in 0..64 {
            let oam_offset = sprite_idx * 4;
            
            let sprite_y = self.oam[oam_offset] as usize;
            let tile_index = self.oam[oam_offset + 1];
            let attributes = self.oam[oam_offset + 2];
            let sprite_x = self.oam[oam_offset + 3] as usize;
            
            // Sprite height (8 or 16 pixels)
            let sprite_height = if self.ctrl.contains(PpuCtrl::SPRITE_SIZE) {
                16
            } else {
                8
            };
            
            // Check if pixel is within sprite bounds
            let sprite_y_end = sprite_y.wrapping_add(sprite_height);
            if y < sprite_y || y >= sprite_y_end || x < sprite_x || x >= sprite_x + 8 {
                continue;
            }
            
            // Calculate pixel position within sprite
            let mut pixel_x = (x - sprite_x) as u8;
            let mut pixel_y = (y - sprite_y) as u8;
            
            // Handle horizontal flip
            if attributes & 0x40 != 0 {
                pixel_x = 7 - pixel_x;
            }
            
            // Handle vertical flip
            if attributes & 0x80 != 0 {
                pixel_y = (sprite_height as u8 - 1) - pixel_y;
            }
            
            // Get pattern table address
            let pattern_table_base = if self.ctrl.contains(PpuCtrl::SPRITE_PATTERN) {
                0x1000
            } else {
                0x0000
            };
            
            // Calculate tile address
            let tile_addr = pattern_table_base + (tile_index as u16) * 16;
            
            // Read bit planes
            let low_byte = self.chr_rom.get((tile_addr + pixel_y as u16) as usize).copied().unwrap_or(0);
            let high_byte = self.chr_rom.get((tile_addr + 8 + pixel_y as u16) as usize).copied().unwrap_or(0);
            
            // Extract pixel value
            let bit_pos = 7 - pixel_x;
            let pixel_low = (low_byte >> bit_pos) & 0x01;
            let pixel_high = (high_byte >> bit_pos) & 0x01;
            let pixel_value = (pixel_high << 1) | pixel_low;
            
            // If pixel is transparent, skip this sprite
            if pixel_value == 0 {
                continue;
            }
            
            // Get palette index (sprites use palettes 4-7)
            let palette_num = attributes & 0x03;
            let palette_addr = (0x10 + palette_num * 4 + pixel_value) as usize;
            let palette_index = self.palette[palette_addr];
            
            // Check priority (0 = in front of BG, 1 = behind BG)
            let behind_bg = attributes & 0x20 != 0;
            
            return (palette_index, true, !behind_bg);
        }
        
        // No sprite pixel found
        (0, false, false)
    }
    
    /// Read from VRAM without side effects (for rendering)
    fn read_vram_direct(&self, addr: u16) -> u8 {
        let addr = addr & 0x3FFF;
        match addr {
            0x0000..=0x1FFF => self.chr_rom.get(addr as usize).copied().unwrap_or(0),
            0x2000..=0x3EFF => {
                let mirror_addr = self.mirror_nametable(addr);
                self.vram[mirror_addr]
            }
            0x3F00..=0x3FFF => {
                let palette_addr = (addr - 0x3F00) & 0x1F;
                self.palette[palette_addr as usize]
            }
            _ => 0,
        }
    }
    
    /// Check if rendering is enabled
    fn is_rendering(&self) -> bool {
        self.mask.contains(PpuMask::SHOW_BG) || self.mask.contains(PpuMask::SHOW_SPRITES)
    }
}

impl Default for Ppu {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_ppu_creation() {
        let ppu = Ppu::new();
        assert_eq!(ppu.scanline, 0);
        assert_eq!(ppu.cycle, 0);
        assert_eq!(ppu.frame, 0);
    }
    
    #[test]
    fn test_ppuctrl_write() {
        let mut ppu = Ppu::new();
        ppu.write_register(0x2000, 0b10011001);
        
        assert!(ppu.ctrl.contains(PpuCtrl::NMI_ENABLE));
        assert!(ppu.ctrl.contains(PpuCtrl::BG_PATTERN));
        assert!(ppu.ctrl.contains(PpuCtrl::SPRITE_PATTERN));
        assert!(ppu.ctrl.contains(PpuCtrl::NAMETABLE_X));
    }
    
    #[test]
    fn test_ppustatus_read() {
        let mut ppu = Ppu::new();
        ppu.status.insert(PpuStatus::VBLANK);
        
        let status = ppu.read_register(0x2002);
        assert_eq!(status & 0x80, 0x80); // VBlank bit set
        
        // Reading should clear VBlank
        let status = ppu.read_register(0x2002);
        assert_eq!(status & 0x80, 0x00); // VBlank bit cleared
    }
    
    #[test]
    fn test_oam_write() {
        let mut ppu = Ppu::new();
        ppu.write_register(0x2003, 0x10); // OAMADDR = 0x10
        ppu.write_register(0x2004, 0x42); // Write to OAM
        
        assert_eq!(ppu.oam[0x10], 0x42);
        assert_eq!(ppu.oam_addr, 0x11); // Auto-incremented
    }
    
    #[test]
    fn test_vram_address_write() {
        let mut ppu = Ppu::new();
        
        // Write high byte
        ppu.write_register(0x2006, 0x20);
        assert!(ppu.write_latch); // Should be true after first write
        
        // Write low byte
        ppu.write_register(0x2006, 0x00);
        assert_eq!(ppu.vram_addr, 0x2000);
        assert!(!ppu.write_latch); // Should be false after second write
    }
    
    #[test]
    fn test_vblank_timing() {
        let mut ppu = Ppu::new();
        
        // Run to scanline 241
        for _ in 0..241 {
            for _ in 0..341 {
                ppu.tick();
            }
        }
        
        // Should enter VBlank
        ppu.tick(); // Cycle 1 of scanline 241
        assert!(ppu.status.contains(PpuStatus::VBLANK));
    }
}
