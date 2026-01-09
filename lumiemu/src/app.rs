use std::sync::{Arc, Mutex};
use std::sync::mpsc::{channel, Sender, Receiver};
use std::thread;
use std::time::{Duration, Instant};
use emu_nes::system::NesSystem;
use emu_core::Button;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};

slint::include_modules!();

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

        // Start emulator callback
        let emulator_clone = emulator.clone();
        let window_weak = window.as_weak();
        window.on_start_emulation(move || {
            println!("Start emulation clicked");
            
            let emu_lock = emulator_clone.lock().unwrap();
            if emu_lock.is_none() {
                println!("No ROM loaded, cannot start");
                return;
            }
            drop(emu_lock);

            println!("Starting emulation thread...");

            // Set running state
            if let Some(window) = window_weak.upgrade() {
                window.set_emulator_running(true);
            }

            let emulator_thread = emulator_clone.clone();
            let window_weak_clone = window_weak.clone();

            thread::spawn(move || {
                println!("Emulation thread started");
                let target_fps = 60.0;
                let frame_duration = Duration::from_secs_f64(1.0 / target_fps);
                let mut frame_count = 0;
                let mut fps_timer = Instant::now();

                loop {
                    let frame_start = Instant::now();

                    // Run one frame and get framebuffer
                    let (should_continue, rgba_data) = {
                        let mut emu_lock = emulator_thread.lock().unwrap();
                        if let Some(ref mut system) = *emu_lock {
                            // Run for one frame (29780 CPU cycles ≈ 1/60th second)
                            if let Err(e) = system.run_cycles(29780) {
                                eprintln!("Emulation error: {:?}", e);
                                return;
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
        window.on_stop_emulation(move || {
            println!("Stop emulation clicked");
            let mut emu_lock = emulator_clone.lock().unwrap();
            *emu_lock = None;
            
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
            println!("Emulation stopped");
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
