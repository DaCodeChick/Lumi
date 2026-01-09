use std::sync::{Arc, Mutex};
use std::collections::VecDeque;
use std::thread;
use std::time::{Duration, Instant};
use emu_nes::system::NesSystem;
use emu_core::Button;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{Stream, StreamConfig, SampleRate};

slint::include_modules!();

/// Audio sample rate (Hz)
const SAMPLE_RATE: u32 = 44100;

/// Samples per frame at 60 FPS: 44100 / 60 = 735
const SAMPLES_PER_FRAME: usize = 735;

/// Audio buffer size (how many samples to buffer)
const AUDIO_BUFFER_SIZE: usize = 4096;

/// Audio system for playing NES audio
struct AudioSystem {
    _stream: Stream,
    sample_buffer: Arc<Mutex<VecDeque<f32>>>,
}

impl AudioSystem {
    fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let host = cpal::default_host();
        let device = host.default_output_device()
            .ok_or("No audio output device available")?;
        
        let config = StreamConfig {
            channels: 1, // Mono
            sample_rate: SampleRate(SAMPLE_RATE),
            buffer_size: cpal::BufferSize::Default,
        };
        
        println!("Audio config: {:?}", config);
        
        // Shared buffer for audio samples
        let sample_buffer = Arc::new(Mutex::new(VecDeque::with_capacity(AUDIO_BUFFER_SIZE)));
        let buffer_clone = sample_buffer.clone();
        
        let stream = device.build_output_stream(
            &config,
            move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
                let mut buffer = buffer_clone.lock().unwrap();
                let mut last_sample = -1.0; // APU silence level
                
                // Fill output buffer
                for sample in data.iter_mut() {
                    if let Some(s) = buffer.pop_front() {
                        *sample = s;
                        last_sample = s;
                    } else {
                        // Buffer underrun - repeat last sample to avoid clicking
                        *sample = last_sample;
                    }
                }
                
                // Debug: warn if buffer is running low
                if buffer.len() < 100 {
                    // eprintln!("Audio buffer low: {} samples", buffer.len());
                }
            },
            move |err| {
                eprintln!("Audio stream error: {}", err);
            },
            None,
        )?;
        
        // Pre-fill buffer with a small amount of silence to prevent initial underrun
        // Just enough to cover the first audio callback (~1-2ms)
        {
            let mut buffer = sample_buffer.lock().unwrap();
            for _ in 0..256 {
                buffer.push_back(-1.0);
            }
        }
        
        stream.play()?;
        
        Ok(Self {
            _stream: stream,
            sample_buffer,
        })
    }
    
    /// Send audio samples to the playback buffer
    fn send_samples(&self, samples: &[f32]) {
        let mut buffer = self.sample_buffer.lock().unwrap();
        
        // Add samples if buffer has space
        for &sample in samples {
            if buffer.len() < AUDIO_BUFFER_SIZE {
                buffer.push_back(sample);
            } else {
                // Buffer full - drop samples to avoid unbounded growth
                break;
            }
        }
    }
    
    /// Fade out audio buffer to prevent pop
    /// Gradually fades current samples to silence (-1.0)
    fn fade_out(&self) {
        let mut buffer = self.sample_buffer.lock().unwrap();
        let fade_samples = 441; // ~10ms fade at 44.1kHz
        
        // Clear existing buffer and add fade-out samples
        let current_level = buffer.back().copied().unwrap_or(-1.0);
        buffer.clear();
        
        for i in 0..fade_samples {
            let t = i as f32 / fade_samples as f32;
            let sample = current_level * (1.0 - t) + (-1.0) * t;
            buffer.push_back(sample);
        }
    }
}

pub struct EmulatorApp {
    window: MainWindow,
    #[allow(dead_code)]
    emulator: Arc<Mutex<Option<NesSystem>>>,
}

impl EmulatorApp {
    pub fn new() -> Result<Self, slint::PlatformError> {
        let window = MainWindow::new()?;
        let emulator = Arc::new(Mutex::new(None));

        // Setup callbacks
        Self::setup_callbacks(&window, emulator.clone());

        Ok(Self { window, emulator })
    }

    fn setup_callbacks(window: &MainWindow, emulator: Arc<Mutex<Option<NesSystem>>>) {
        // Shared flag to control whether emulation thread is running
        let running = Arc::new(Mutex::new(false));
        // Load ROM callback
        let emulator_clone = emulator.clone();
        let window_weak = window.as_weak();
        window.on_load_rom(move || {
            println!("Load ROM button clicked");
            
            match native_dialog::FileDialog::new()
                .add_filter("NES ROM", &["nes"])
                .show_open_single_file()
            {
                Ok(Some(path)) => {
                    println!("Selected file: {:?}", path);
                    
                    let mut emu_lock = emulator_clone.lock().unwrap();
                    match NesSystem::new(&path) {
                        Ok(system) => {
                            println!("ROM loaded successfully!");
                            *emu_lock = Some(system);
                            if let Some(window) = window_weak.upgrade() {
                                let path_str = path.to_string_lossy().into_owned();
                                window.set_rom_path(path_str.into());
                                println!("ROM path set in UI");
                            }
                        }
                        Err(e) => {
                            eprintln!("Failed to load ROM: {:?}", e);
                        }
                    }
                }
                Ok(None) => {
                    println!("File dialog cancelled");
                }
                Err(e) => {
                    eprintln!("File dialog error: {:?}", e);
                }
            }
        });

        // Start emulation callback
        let emulator_clone = emulator.clone();
        let window_weak = window.as_weak();
        let running_clone = running.clone();
        window.on_start_emulation(move || {
            println!("Start emulation clicked");
            
            // Check if ROM is loaded and reset it
            {
                let mut emu_lock = emulator_clone.lock().unwrap();
                if let Some(ref mut system) = *emu_lock {
                    system.reset();
                    println!("Emulator reset - starting from beginning");
                } else {
                    println!("No ROM loaded, cannot start");
                    return;
                }
            }

            // Check if already running
            {
                let mut running_lock = running_clone.lock().unwrap();
                if *running_lock {
                    println!("Emulation already running");
                    return;
                }
                *running_lock = true;
            }

            println!("Starting emulation thread...");

            // Set running state
            if let Some(window) = window_weak.upgrade() {
                window.set_emulator_running(true);
            }

            let emulator_thread = emulator_clone.clone();
            let window_weak_clone = window_weak.clone();
            let running_thread = running_clone.clone();

            thread::spawn(move || {
                println!("Emulation thread started");
                
                // Initialize audio in emulation thread (cpal Stream is not Send)
                let audio = match AudioSystem::new() {
                    Ok(audio_system) => {
                        println!("✓ Audio system initialized");
                        Some(audio_system)
                    }
                    Err(e) => {
                        eprintln!("⚠ Failed to initialize audio: {}", e);
                        eprintln!("  Continuing without audio...");
                        None
                    }
                };
                
                let target_fps = 60.0;
                let frame_duration = Duration::from_secs_f64(1.0 / target_fps);
                let mut frame_count = 0;
                let mut fps_timer = Instant::now();
                
                // Audio sampling: collect samples throughout frame execution
                let mut audio_buffer = Vec::with_capacity(SAMPLES_PER_FRAME);

                loop {
                    // Check if we should continue running
                    {
                        let running_lock = running_thread.lock().unwrap();
                        if !*running_lock {
                            println!("Emulation stopped by user");
                            // Fade out audio to prevent pop
                            if let Some(ref audio_system) = audio {
                                audio_system.fade_out();
                            }
                            break;
                        }
                    }
                    
                    let frame_start = Instant::now();

                    // Run one frame, collect audio samples, and get framebuffer
                    let (should_continue, rgba_data) = {
                        let mut emu_lock = emulator_thread.lock().unwrap();
                        if let Some(ref mut system) = *emu_lock {
                            audio_buffer.clear();
                            
                            // Run for one frame (29780 CPU cycles ≈ 1/60th second)
                            // We need 735 audio samples, so run in 735 chunks
                            const CYCLES_PER_FRAME: u64 = 29780;
                            let cycles_per_sample = CYCLES_PER_FRAME / SAMPLES_PER_FRAME as u64; // ~40 cycles
                            
                            for i in 0..SAMPLES_PER_FRAME {
                                // Run cycles for this sample period
                                let cycles_to_run = if i == SAMPLES_PER_FRAME - 1 {
                                    // Last sample - run remaining cycles
                                    CYCLES_PER_FRAME - (cycles_per_sample * i as u64)
                                } else {
                                    cycles_per_sample
                                };
                                
                                if let Err(e) = system.run_cycles(cycles_to_run) {
                                    eprintln!("Emulation error: {:?}", e);
                                    return;
                                }
                                
                                // Sample audio after running cycles
                                audio_buffer.push(system.audio_sample());
                            }

                            // Convert framebuffer to image
                            let framebuffer = system.framebuffer();
                            let rgba_data = Self::framebuffer_to_rgba(framebuffer);
                            
                            (true, rgba_data)
                        } else {
                            println!("Emulator stopped");
                            return;
                        }
                    };

                    if !should_continue {
                        break;
                    }
                    
                    // Send audio samples to audio thread
                    if let Some(ref audio_system) = audio {
                        audio_system.send_samples(&audio_buffer);
                    }

                    // Update display on UI thread
                    let window_weak_update = window_weak_clone.clone();
                    slint::invoke_from_event_loop(move || {
                        if let Some(window) = window_weak_update.upgrade() {
                            let buffer = slint::SharedPixelBuffer::clone_from_slice(
                                &rgba_data,
                                256,
                                240,
                            );
                            let image = slint::Image::from_rgba8(buffer);
                            window.set_screen_image(image);
                        }
                    }).ok();

                    // FPS calculation
                    frame_count += 1;
                    if fps_timer.elapsed() >= Duration::from_secs(1) {
                        let fps = frame_count;
                        let window_weak_fps = window_weak_clone.clone();
                        slint::invoke_from_event_loop(move || {
                            if let Some(window) = window_weak_fps.upgrade() {
                                window.set_fps_text(format!("FPS: {}", fps).into());
                            }
                        }).ok();
                        frame_count = 0;
                        fps_timer = Instant::now();
                    }

                    // Frame timing
                    let elapsed = frame_start.elapsed();
                    if elapsed < frame_duration {
                        thread::sleep(frame_duration - elapsed);
                    }
                }

                println!("Emulation thread ended");
                
                // Clear screen and running state when stopped
                slint::invoke_from_event_loop(move || {
                    if let Some(window) = window_weak_clone.upgrade() {
                        window.set_emulator_running(false);
                        window.set_fps_text("FPS: 0".into());
                        
                        // Create a black screen
                        let black_screen = vec![0u8; 256 * 240 * 4];
                        let buffer = slint::SharedPixelBuffer::clone_from_slice(
                            &black_screen,
                            256,
                            240,
                        );
                        let image = slint::Image::from_rgba8(buffer);
                        window.set_screen_image(image);
                    }
                }).ok();
            });
        });

        // Stop emulator callback
        let emulator_clone = emulator.clone();
        let window_weak = window.as_weak();
        let running_clone = running.clone();
        window.on_stop_emulation(move || {
            println!("Stop emulation clicked");
            
            // Set running flag to false to stop the emulation thread
            {
                let mut running_lock = running_clone.lock().unwrap();
                *running_lock = false;
            }
            
            // Reset the emulator state (so next Start begins fresh)
            {
                let mut emu_lock = emulator_clone.lock().unwrap();
                if let Some(ref mut system) = *emu_lock {
                    system.reset();
                    println!("Emulator reset to initial state");
                }
            }
            
            // Clear the screen and update UI state
            if let Some(window) = window_weak.upgrade() {
                window.set_emulator_running(false);
                
                // Create a black screen
                let black_screen = vec![0u8; 256 * 240 * 4]; // All black pixels (RGBA)
                let buffer = slint::SharedPixelBuffer::clone_from_slice(
                    &black_screen,
                    256,
                    240,
                );
                let image = slint::Image::from_rgba8(buffer);
                window.set_screen_image(image);
                window.set_fps_text("FPS: 0".into());
            }
            println!("Emulation stopped and reset (ROM still loaded)");
        });

        // Keyboard press handler
        let emulator_clone = emulator.clone();
        window.on_key_pressed(move |key| {
            let mut emu_lock = emulator_clone.lock().unwrap();
            if let Some(ref mut system) = *emu_lock {
                let controller = system.controller1().state();
                
                match key.as_str() {
                    "↑" | "w" | "W" => controller.press(Button::UP),
                    "↓" | "s" | "S" => controller.press(Button::DOWN),
                    "←" | "a" | "A" => controller.press(Button::LEFT),
                    "→" | "d" | "D" => controller.press(Button::RIGHT),
                    "z" | "Z" => controller.press(Button::A),
                    "x" | "X" => controller.press(Button::B),
                    "\n" | "\r" => controller.press(Button::START),
                    " " => controller.press(Button::SELECT),
                    _ => {}
                }
            }
        });

        // Keyboard release handler
        let emulator_clone = emulator.clone();
        window.on_key_released(move |key| {
            let mut emu_lock = emulator_clone.lock().unwrap();
            if let Some(ref mut system) = *emu_lock {
                let controller = system.controller1().state();
                
                match key.as_str() {
                    "↑" | "w" | "W" => controller.release(Button::UP),
                    "↓" | "s" | "S" => controller.release(Button::DOWN),
                    "←" | "a" | "A" => controller.release(Button::LEFT),
                    "→" | "d" | "D" => controller.release(Button::RIGHT),
                    "z" | "Z" => controller.release(Button::A),
                    "x" | "X" => controller.release(Button::B),
                    "\n" | "\r" => controller.release(Button::START),
                    " " => controller.release(Button::SELECT),
                    _ => {}
                }
            }
        });
    }

    fn framebuffer_to_rgba(framebuffer: &[u8]) -> Vec<u8> {
        // NES palette (NTSC colors)
        const NES_PALETTE: [(u8, u8, u8); 64] = [
            (84, 84, 84), (0, 30, 116), (8, 16, 144), (48, 0, 136),
            (68, 0, 100), (92, 0, 48), (84, 4, 0), (60, 24, 0),
            (32, 42, 0), (8, 58, 0), (0, 64, 0), (0, 60, 0),
            (0, 50, 60), (0, 0, 0), (0, 0, 0), (0, 0, 0),
            (152, 150, 152), (8, 76, 196), (48, 50, 236), (92, 30, 228),
            (136, 20, 176), (160, 20, 100), (152, 34, 32), (120, 60, 0),
            (84, 90, 0), (40, 114, 0), (8, 124, 0), (0, 118, 40),
            (0, 102, 120), (0, 0, 0), (0, 0, 0), (0, 0, 0),
            (236, 238, 236), (76, 154, 236), (120, 124, 236), (176, 98, 236),
            (228, 84, 236), (236, 88, 180), (236, 106, 100), (212, 136, 32),
            (160, 170, 0), (116, 196, 0), (76, 208, 32), (56, 204, 108),
            (56, 180, 204), (60, 60, 60), (0, 0, 0), (0, 0, 0),
            (236, 238, 236), (168, 204, 236), (188, 188, 236), (212, 178, 236),
            (236, 174, 236), (236, 174, 212), (236, 180, 176), (228, 196, 144),
            (204, 210, 120), (180, 222, 120), (168, 226, 144), (152, 226, 180),
            (160, 214, 228), (160, 162, 160), (0, 0, 0), (0, 0, 0),
        ];

        let mut rgba = Vec::with_capacity(256 * 240 * 4);
        for &color_idx in framebuffer {
            let (r, g, b) = NES_PALETTE[(color_idx & 0x3F) as usize];
            rgba.push(r);
            rgba.push(g);
            rgba.push(b);
            rgba.push(255); // Alpha
        }
        rgba
    }

    pub fn run(&self) -> Result<(), slint::PlatformError> {
        self.window.run()
    }
}

impl Default for EmulatorApp {
    fn default() -> Self {
        Self::new().expect("Failed to create EmulatorApp")
    }
}
