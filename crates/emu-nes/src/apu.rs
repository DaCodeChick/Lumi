/// NES APU (Audio Processing Unit) Implementation
/// 
/// The APU generates audio for the NES. It has 5 channels:
/// - 2 Pulse channels (square waves) for melody/harmony
/// - 1 Triangle channel for bass lines
/// - 1 Noise channel for percussion/sound effects
/// - 1 DMC (Delta Modulation Channel) for playing samples
///
/// Audio output rate: ~1.789773 MHz (NTSC) / 2 = 894,886.5 Hz clock
/// Sample rate: typically 44.1 kHz or 48 kHz for output
///
/// APU Registers:
/// $4000-$4003: Pulse 1
/// $4004-$4007: Pulse 2
/// $4008-$400B: Triangle
/// $400C-$400F: Noise
/// $4010-$4013: DMC
/// $4015: Status
/// $4017: Frame Counter

/// Pulse channel (2 of these in the APU)
/// Generates square waves with various duty cycles
#[derive(Debug, Clone)]
pub struct PulseChannel {
    /// Enable flag
    enabled: bool,
    
    /// Duty cycle (0-3): 12.5%, 25%, 50%, 75%
    duty: u8,
    
    /// Length counter halt / envelope loop flag
    length_halt: bool,
    
    /// Constant volume / envelope flag (false = use envelope)
    constant_volume: bool,
    
    /// Volume / envelope period
    volume: u8,
    
    /// Sweep enabled
    sweep_enabled: bool,
    
    /// Sweep period
    sweep_period: u8,
    
    /// Sweep negate flag
    sweep_negate: bool,
    
    /// Sweep shift count
    sweep_shift: u8,
    
    /// Timer period (11 bits)
    timer_period: u16,
    
    /// Length counter load
    length_counter: u8,
    
    // Internal state
    /// Current timer value
    timer: u16,
    
    /// Current duty position (0-7)
    duty_position: u8,
    
    /// Envelope divider
    envelope_divider: u8,
    
    /// Envelope counter
    envelope_counter: u8,
    
    /// Envelope start flag
    envelope_start: bool,
}

impl PulseChannel {
    pub fn new() -> Self {
        Self {
            enabled: false,
            duty: 0,
            length_halt: false,
            constant_volume: false,
            volume: 0,
            sweep_enabled: false,
            sweep_period: 0,
            sweep_negate: false,
            sweep_shift: 0,
            timer_period: 0,
            length_counter: 0,
            timer: 0,
            duty_position: 0,
            envelope_divider: 0,
            envelope_counter: 0,
            envelope_start: false,
        }
    }
    
    /// Write to register 0 (duty, length halt, constant volume, volume)
    pub fn write_reg0(&mut self, value: u8) {
        self.duty = (value >> 6) & 0x03;
        self.length_halt = (value & 0x20) != 0;
        self.constant_volume = (value & 0x10) != 0;
        self.volume = value & 0x0F;
    }
    
    /// Write to register 1 (sweep unit)
    pub fn write_reg1(&mut self, value: u8) {
        self.sweep_enabled = (value & 0x80) != 0;
        self.sweep_period = (value >> 4) & 0x07;
        self.sweep_negate = (value & 0x08) != 0;
        self.sweep_shift = value & 0x07;
    }
    
    /// Write to register 2 (timer low)
    pub fn write_reg2(&mut self, value: u8) {
        self.timer_period = (self.timer_period & 0x0700) | (value as u16);
    }
    
    /// Write to register 3 (length counter load, timer high)
    pub fn write_reg3(&mut self, value: u8) {
        self.timer_period = (self.timer_period & 0x00FF) | (((value & 0x07) as u16) << 8);
        
        if self.enabled {
            // Load length counter
            self.length_counter = LENGTH_TABLE[(value >> 3) as usize];
        }
        
        // Restart envelope and reset phase
        self.envelope_start = true;
        self.duty_position = 0;
    }
    
    /// Enable/disable the channel
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
        if !enabled {
            self.length_counter = 0;
        }
    }
    
    /// Get length counter status (for $4015 reads)
    pub fn length_counter_status(&self) -> bool {
        self.length_counter > 0
    }
    
    /// Clock the timer (called every APU cycle)
    pub fn clock_timer(&mut self) {
        if self.timer == 0 {
            self.timer = self.timer_period;
            self.duty_position = (self.duty_position + 1) & 0x07;
        } else {
            self.timer -= 1;
        }
    }
    
    /// Clock the envelope (called on quarter frame)
    pub fn clock_envelope(&mut self) {
        if self.envelope_start {
            self.envelope_start = false;
            self.envelope_counter = 15;
            self.envelope_divider = self.volume;
        } else if self.envelope_divider == 0 {
            self.envelope_divider = self.volume;
            
            if self.envelope_counter > 0 {
                self.envelope_counter -= 1;
            } else if self.length_halt {
                self.envelope_counter = 15;
            }
        } else {
            self.envelope_divider -= 1;
        }
    }
    
    /// Clock the length counter (called on half frame)
    pub fn clock_length(&mut self) {
        if !self.length_halt && self.length_counter > 0 {
            self.length_counter -= 1;
        }
    }
    
    /// Get current output sample (0-15)
    pub fn output(&self) -> u8 {
        // Duty cycle patterns (8 steps each)
        const DUTY_TABLE: [[u8; 8]; 4] = [
            [0, 1, 0, 0, 0, 0, 0, 0], // 12.5%
            [0, 1, 1, 0, 0, 0, 0, 0], // 25%
            [0, 1, 1, 1, 1, 0, 0, 0], // 50%
            [1, 0, 0, 1, 1, 1, 1, 1], // 75% (inverted 25%)
        ];
        
        // Must be enabled and have length counter
        if !self.enabled || self.length_counter == 0 {
            return 0;
        }
        
        // Get duty cycle output
        let duty_out = DUTY_TABLE[self.duty as usize][self.duty_position as usize];
        
        if duty_out == 0 {
            return 0;
        }
        
        // Use constant volume or envelope
        if self.constant_volume {
            self.volume
        } else {
            self.envelope_counter
        }
    }
}

/// Triangle channel
/// Generates triangle waves for bass lines
#[derive(Debug, Clone)]
pub struct TriangleChannel {
    /// Enable flag
    enabled: bool,
    
    /// Length counter halt / linear counter control
    length_halt: bool,
    
    /// Linear counter load
    linear_counter_load: u8,
    
    /// Timer period (11 bits)
    timer_period: u16,
    
    /// Length counter load
    length_counter: u8,
    
    // Internal state
    /// Linear counter
    linear_counter: u8,
    
    /// Linear counter reload flag
    linear_counter_reload: bool,
    
    /// Current timer value
    timer: u16,
    
    /// Sequence position (0-31)
    sequence_position: u8,
}

impl TriangleChannel {
    pub fn new() -> Self {
        Self {
            enabled: false,
            length_halt: false,
            linear_counter_load: 0,
            timer_period: 0,
            length_counter: 0,
            linear_counter: 0,
            linear_counter_reload: false,
            timer: 0,
            sequence_position: 0,
        }
    }
    
    /// Write to register 0 (linear counter)
    pub fn write_reg0(&mut self, value: u8) {
        self.length_halt = (value & 0x80) != 0;
        self.linear_counter_load = value & 0x7F;
    }
    
    /// Write to register 2 (timer low)
    pub fn write_reg2(&mut self, value: u8) {
        self.timer_period = (self.timer_period & 0x0700) | (value as u16);
    }
    
    /// Write to register 3 (length counter load, timer high)
    pub fn write_reg3(&mut self, value: u8) {
        self.timer_period = (self.timer_period & 0x00FF) | (((value & 0x07) as u16) << 8);
        
        if self.enabled {
            self.length_counter = LENGTH_TABLE[(value >> 3) as usize];
        }
        
        self.linear_counter_reload = true;
    }
    
    /// Enable/disable the channel
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
        if !enabled {
            self.length_counter = 0;
        }
    }
    
    /// Get length counter status
    pub fn length_counter_status(&self) -> bool {
        self.length_counter > 0
    }
    
    /// Clock the timer
    pub fn clock_timer(&mut self) {
        if self.timer == 0 {
            self.timer = self.timer_period;
            
            // Only advance if both counters are non-zero
            if self.linear_counter > 0 && self.length_counter > 0 {
                self.sequence_position = (self.sequence_position + 1) & 0x1F;
            }
        } else {
            self.timer -= 1;
        }
    }
    
    /// Clock the linear counter (called on quarter frame)
    pub fn clock_linear_counter(&mut self) {
        if self.linear_counter_reload {
            self.linear_counter = self.linear_counter_load;
        } else if self.linear_counter > 0 {
            self.linear_counter -= 1;
        }
        
        if !self.length_halt {
            self.linear_counter_reload = false;
        }
    }
    
    /// Clock the length counter (called on half frame)
    pub fn clock_length(&mut self) {
        if !self.length_halt && self.length_counter > 0 {
            self.length_counter -= 1;
        }
    }
    
    /// Get current output sample (0-15)
    pub fn output(&self) -> u8 {
        // Triangle waveform sequence (32 steps)
        const TRIANGLE_TABLE: [u8; 32] = [
            15, 14, 13, 12, 11, 10, 9, 8, 7, 6, 5, 4, 3, 2, 1, 0,
            0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15,
        ];
        
        // Ultrasonic frequencies are silenced (timer < 2)
        if !self.enabled || self.timer_period < 2 {
            return 0;
        }
        
        TRIANGLE_TABLE[self.sequence_position as usize]
    }
}

/// Noise channel
/// Generates pseudo-random noise for percussion
#[derive(Debug, Clone)]
pub struct NoiseChannel {
    /// Enable flag
    enabled: bool,
    
    /// Length counter halt / envelope loop
    length_halt: bool,
    
    /// Constant volume / envelope flag
    constant_volume: bool,
    
    /// Volume / envelope period
    volume: u8,
    
    /// Mode flag (false = mode 0, true = mode 1)
    mode: bool,
    
    /// Timer period (4 bits, indexes into period table)
    timer_period: u8,
    
    /// Length counter
    length_counter: u8,
    
    // Internal state
    /// Current timer value
    timer: u16,
    
    /// 15-bit shift register (Linear Feedback Shift Register)
    shift_register: u16,
    
    /// Envelope divider
    envelope_divider: u8,
    
    /// Envelope counter
    envelope_counter: u8,
    
    /// Envelope start flag
    envelope_start: bool,
}

impl NoiseChannel {
    pub fn new() -> Self {
        Self {
            enabled: false,
            length_halt: false,
            constant_volume: false,
            volume: 0,
            mode: false,
            timer_period: 0,
            length_counter: 0,
            timer: 0,
            shift_register: 1,
            envelope_divider: 0,
            envelope_counter: 0,
            envelope_start: false,
        }
    }
    
    /// Write to register 0 (envelope)
    pub fn write_reg0(&mut self, value: u8) {
        self.length_halt = (value & 0x20) != 0;
        self.constant_volume = (value & 0x10) != 0;
        self.volume = value & 0x0F;
    }
    
    /// Write to register 2 (mode, period)
    pub fn write_reg2(&mut self, value: u8) {
        self.mode = (value & 0x80) != 0;
        self.timer_period = value & 0x0F;
    }
    
    /// Write to register 3 (length counter load)
    pub fn write_reg3(&mut self, value: u8) {
        if self.enabled {
            self.length_counter = LENGTH_TABLE[(value >> 3) as usize];
        }
        self.envelope_start = true;
    }
    
    /// Enable/disable the channel
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
        if !enabled {
            self.length_counter = 0;
        }
    }
    
    /// Get length counter status
    pub fn length_counter_status(&self) -> bool {
        self.length_counter > 0
    }
    
    /// Clock the timer
    pub fn clock_timer(&mut self) {
        const NOISE_PERIOD_TABLE: [u16; 16] = [
            4, 8, 16, 32, 64, 96, 128, 160, 202, 254, 380, 508, 762, 1016, 2034, 4068
        ];
        
        if self.timer == 0 {
            self.timer = NOISE_PERIOD_TABLE[self.timer_period as usize];
            
            // Clock the shift register
            let feedback = if self.mode {
                // Mode 1: bits 0 and 6
                (self.shift_register & 1) ^ ((self.shift_register >> 6) & 1)
            } else {
                // Mode 0: bits 0 and 1
                (self.shift_register & 1) ^ ((self.shift_register >> 1) & 1)
            };
            
            self.shift_register >>= 1;
            self.shift_register |= feedback << 14;
        } else {
            self.timer -= 1;
        }
    }
    
    /// Clock the envelope (called on quarter frame)
    pub fn clock_envelope(&mut self) {
        if self.envelope_start {
            self.envelope_start = false;
            self.envelope_counter = 15;
            self.envelope_divider = self.volume;
        } else if self.envelope_divider == 0 {
            self.envelope_divider = self.volume;
            
            if self.envelope_counter > 0 {
                self.envelope_counter -= 1;
            } else if self.length_halt {
                self.envelope_counter = 15;
            }
        } else {
            self.envelope_divider -= 1;
        }
    }
    
    /// Clock the length counter (called on half frame)
    pub fn clock_length(&mut self) {
        if !self.length_halt && self.length_counter > 0 {
            self.length_counter -= 1;
        }
    }
    
    /// Get current output sample (0-15)
    pub fn output(&self) -> u8 {
        // Output is 0 if bit 0 of shift register is set
        if !self.enabled || self.length_counter == 0 || (self.shift_register & 1) != 0 {
            return 0;
        }
        
        if self.constant_volume {
            self.volume
        } else {
            self.envelope_counter
        }
    }
}

/// DMC (Delta Modulation Channel)
/// Plays 1-bit delta-encoded samples
#[derive(Debug, Clone)]
pub struct DmcChannel {
    /// Enable flag
    enabled: bool,
    
    /// IRQ enable flag
    irq_enabled: bool,
    
    /// Loop flag
    loop_flag: bool,
    
    /// Rate index (0-15)
    rate: u8,
    
    /// Direct load (7-bit value)
    direct_load: u8,
    
    /// Sample address ($C000 + address * 64)
    sample_address: u16,
    
    /// Sample length (length * 16 + 1 bytes)
    sample_length: u16,
    
    // Internal state
    /// Current output level (7-bit)
    output_level: u8,
    
    /// Bytes remaining
    bytes_remaining: u16,
    
    /// Current address
    current_address: u16,
}

impl DmcChannel {
    pub fn new() -> Self {
        Self {
            enabled: false,
            irq_enabled: false,
            loop_flag: false,
            rate: 0,
            direct_load: 0,
            sample_address: 0xC000,
            sample_length: 0,
            output_level: 0,
            bytes_remaining: 0,
            current_address: 0xC000,
        }
    }
    
    /// Write to register 0 (flags, rate)
    pub fn write_reg0(&mut self, value: u8) {
        self.irq_enabled = (value & 0x80) != 0;
        self.loop_flag = (value & 0x40) != 0;
        self.rate = value & 0x0F;
    }
    
    /// Write to register 1 (direct load)
    pub fn write_reg1(&mut self, value: u8) {
        self.direct_load = value & 0x7F;
        self.output_level = self.direct_load;
    }
    
    /// Write to register 2 (sample address)
    pub fn write_reg2(&mut self, value: u8) {
        self.sample_address = 0xC000 + ((value as u16) << 6);
    }
    
    /// Write to register 3 (sample length)
    pub fn write_reg3(&mut self, value: u8) {
        self.sample_length = ((value as u16) << 4) + 1;
    }
    
    /// Enable/disable the channel
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
        if !enabled {
            self.bytes_remaining = 0;
        } else if self.bytes_remaining == 0 {
            self.restart_sample();
        }
    }
    
    /// Restart the sample
    fn restart_sample(&mut self) {
        self.current_address = self.sample_address;
        self.bytes_remaining = self.sample_length;
    }
    
    /// Get sample status (for $4015 reads)
    pub fn sample_status(&self) -> bool {
        self.bytes_remaining > 0
    }
    
    /// Get current output sample (0-127)
    pub fn output(&self) -> u8 {
        self.output_level
    }
}

/// Length counter lookup table
const LENGTH_TABLE: [u8; 32] = [
    10, 254, 20, 2, 40, 4, 80, 6, 160, 8, 60, 10, 14, 12, 26, 14,
    12, 16, 24, 18, 48, 20, 96, 22, 192, 24, 72, 26, 16, 28, 32, 30,
];

/// NES APU
pub struct Apu {
    /// Pulse channel 1
    pub pulse1: PulseChannel,
    
    /// Pulse channel 2
    pub pulse2: PulseChannel,
    
    /// Triangle channel
    pub triangle: TriangleChannel,
    
    /// Noise channel
    pub noise: NoiseChannel,
    
    /// DMC channel
    pub dmc: DmcChannel,
    
    /// Frame counter mode (false = 4-step, true = 5-step)
    frame_counter_mode: bool,
    
    /// IRQ inhibit flag
    irq_inhibit: bool,
    
    /// Current cycle count
    cycle: u64,
    
    /// Frame counter step
    frame_step: u8,
}

impl Apu {
    pub fn new() -> Self {
        Self {
            pulse1: PulseChannel::new(),
            pulse2: PulseChannel::new(),
            triangle: TriangleChannel::new(),
            noise: NoiseChannel::new(),
            dmc: DmcChannel::new(),
            frame_counter_mode: false,
            irq_inhibit: false,
            cycle: 0,
            frame_step: 0,
        }
    }
    
    /// Reset the APU
    pub fn reset(&mut self) {
        *self = Self::new();
    }
    
    /// Write to APU register
    pub fn write_register(&mut self, addr: u16, value: u8) {
        match addr {
            // Pulse 1
            0x4000 => self.pulse1.write_reg0(value),
            0x4001 => self.pulse1.write_reg1(value),
            0x4002 => self.pulse1.write_reg2(value),
            0x4003 => self.pulse1.write_reg3(value),
            
            // Pulse 2
            0x4004 => self.pulse2.write_reg0(value),
            0x4005 => self.pulse2.write_reg1(value),
            0x4006 => self.pulse2.write_reg2(value),
            0x4007 => self.pulse2.write_reg3(value),
            
            // Triangle
            0x4008 => self.triangle.write_reg0(value),
            0x400A => self.triangle.write_reg2(value),
            0x400B => self.triangle.write_reg3(value),
            
            // Noise
            0x400C => self.noise.write_reg0(value),
            0x400E => self.noise.write_reg2(value),
            0x400F => self.noise.write_reg3(value),
            
            // DMC
            0x4010 => self.dmc.write_reg0(value),
            0x4011 => self.dmc.write_reg1(value),
            0x4012 => self.dmc.write_reg2(value),
            0x4013 => self.dmc.write_reg3(value),
            
            // Status
            0x4015 => {
                self.pulse1.set_enabled((value & 0x01) != 0);
                self.pulse2.set_enabled((value & 0x02) != 0);
                self.triangle.set_enabled((value & 0x04) != 0);
                self.noise.set_enabled((value & 0x08) != 0);
                self.dmc.set_enabled((value & 0x10) != 0);
            }
            
            // Frame Counter
            0x4017 => {
                self.frame_counter_mode = (value & 0x80) != 0;
                self.irq_inhibit = (value & 0x40) != 0;
                
                // Reset frame counter
                self.frame_step = 0;
                
                // If 5-step mode, clock immediately
                if self.frame_counter_mode {
                    self.clock_quarter_frame();
                    self.clock_half_frame();
                }
            }
            
            _ => {} // Ignore writes to other addresses
        }
    }
    
    /// Read from APU register
    pub fn read_register(&self, addr: u16) -> u8 {
        match addr {
            0x4015 => {
                let mut status = 0;
                
                if self.pulse1.length_counter_status() {
                    status |= 0x01;
                }
                if self.pulse2.length_counter_status() {
                    status |= 0x02;
                }
                if self.triangle.length_counter_status() {
                    status |= 0x04;
                }
                if self.noise.length_counter_status() {
                    status |= 0x08;
                }
                if self.dmc.sample_status() {
                    status |= 0x10;
                }
                
                // TODO: DMC IRQ (bit 7), Frame IRQ (bit 6)
                
                status
            }
            _ => 0, // Open bus for other reads
        }
    }
    
    /// Clock the APU (called every CPU cycle)
    pub fn clock(&mut self) {
        // The APU runs at half CPU speed for most things
        if self.cycle % 2 == 0 {
            self.pulse1.clock_timer();
            self.pulse2.clock_timer();
            self.noise.clock_timer();
        }
        
        // Triangle runs at CPU speed
        self.triangle.clock_timer();
        
        // Frame counter (4-step mode: ~120 Hz, 5-step mode: ~96 Hz)
        // Simplified implementation - clock every ~7457 cycles
        if self.cycle % 7457 == 0 {
            self.clock_frame_counter();
        }
        
        self.cycle += 1;
    }
    
    /// Clock the frame counter
    fn clock_frame_counter(&mut self) {
        if self.frame_counter_mode {
            // 5-step mode
            match self.frame_step {
                0 | 2 => self.clock_quarter_frame(),
                1 | 3 => {
                    self.clock_quarter_frame();
                    self.clock_half_frame();
                }
                4 => {} // Do nothing
                _ => unreachable!(),
            }
            
            self.frame_step = (self.frame_step + 1) % 5;
        } else {
            // 4-step mode
            match self.frame_step {
                0 | 2 => self.clock_quarter_frame(),
                1 => {
                    self.clock_quarter_frame();
                    self.clock_half_frame();
                }
                3 => {
                    self.clock_quarter_frame();
                    self.clock_half_frame();
                    // TODO: Set frame IRQ if not inhibited
                }
                _ => unreachable!(),
            }
            
            self.frame_step = (self.frame_step + 1) % 4;
        }
    }
    
    /// Clock quarter frame (envelope and triangle linear counter)
    fn clock_quarter_frame(&mut self) {
        self.pulse1.clock_envelope();
        self.pulse2.clock_envelope();
        self.triangle.clock_linear_counter();
        self.noise.clock_envelope();
    }
    
    /// Clock half frame (length counters and sweep units)
    fn clock_half_frame(&mut self) {
        self.pulse1.clock_length();
        self.pulse2.clock_length();
        self.triangle.clock_length();
        self.noise.clock_length();
        // TODO: Clock sweep units
    }
    
    /// Get mixed audio output sample
    /// Returns a float in range [-1.0, 1.0]
    pub fn output(&self) -> f32 {
        // Get individual channel outputs
        let pulse1 = self.pulse1.output() as f32;
        let pulse2 = self.pulse2.output() as f32;
        let triangle = self.triangle.output() as f32;
        let noise = self.noise.output() as f32;
        let dmc = self.dmc.output() as f32;
        
        // Non-linear mixing (as per NESDev wiki)
        let pulse_out = if pulse1 + pulse2 > 0.0 {
            95.88 / ((8128.0 / (pulse1 + pulse2)) + 100.0)
        } else {
            0.0
        };
        
        let tnd_out = if triangle + noise + dmc > 0.0 {
            159.79 / ((1.0 / (triangle / 8227.0 + noise / 12241.0 + dmc / 22638.0)) + 100.0)
        } else {
            0.0
        };
        
        // Mix and normalize to [-1.0, 1.0]
        (pulse_out + tnd_out) * 2.0 - 1.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_apu_creation() {
        let apu = Apu::new();
        assert_eq!(apu.cycle, 0);
        assert!(!apu.pulse1.enabled);
    }
    
    #[test]
    fn test_enable_channels() {
        let mut apu = Apu::new();
        apu.write_register(0x4015, 0x0F); // Enable all channels except DMC
        
        assert!(apu.pulse1.enabled);
        assert!(apu.pulse2.enabled);
        assert!(apu.triangle.enabled);
        assert!(apu.noise.enabled);
        assert!(!apu.dmc.enabled);
    }
    
    #[test]
    fn test_pulse_duty_cycles() {
        let mut pulse = PulseChannel::new();
        pulse.enabled = true;
        pulse.length_counter = 10;
        pulse.constant_volume = true;
        pulse.volume = 15;
        
        // Test 12.5% duty
        pulse.duty = 0;
        pulse.duty_position = 0;
        assert_eq!(pulse.output(), 0);
        pulse.duty_position = 1;
        assert_eq!(pulse.output(), 15);
        
        // Test 50% duty
        pulse.duty = 2;
        pulse.duty_position = 0;
        assert_eq!(pulse.output(), 0);
        pulse.duty_position = 2;
        assert_eq!(pulse.output(), 15);
    }
}
