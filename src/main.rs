use eframe::egui;
use std::fs;
use std::process::Command;

struct Veltrix {
    configs: Vec<String>,
    selected: Option<usize>,
    laptop_model: String,
    temperature: String,
    current_fan_speed: String,
    target_fan_speed: String,
    fan_speed_value: u8,
}

impl Veltrix {
    fn name() -> &'static str {
        "Veltrix"
    }

    fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        let configs = get_config_list();
        let laptop_model = read_laptop_model();
        let selected = configs.iter().position(|c| c == &laptop_model);

        let mut app = Self {
            configs,
            selected,
            laptop_model,
            temperature: String::new(),
            current_fan_speed: String::new(),
            target_fan_speed: String::new(),
            fan_speed_value: 0,
        };

        app.refresh();
        app
    }

    fn refresh(&mut self) {
        let status = get_status();

        self.temperature = status
            .iter()
            .find(|line| line.contains("Temperature"))
            .cloned()
            .unwrap_or_else(|| "Unknown Temp".to_string());

        self.current_fan_speed = status
            .iter()
            .find(|line| line.contains("Current Fan Speed"))
            .cloned()
            .unwrap_or_else(|| "Unknown Speed".to_string());

        self.target_fan_speed = status
            .iter()
            .find(|line| line.contains("Target Fan Speed"))
            .cloned()
            .unwrap_or_else(|| "Unknown Speed".to_string());
    }

    fn apply_fan_speed(&self) {
        match Command::new("nbfc")
            .arg("set")
            .arg("-s")
            .arg(self.fan_speed_value.to_string())
            .output()
        {
            Ok(output) if !output.status.success() => {
                eprintln!("Failed to apply fan speed: {:?}", output);
            }
            Err(e) => eprintln!("Error running nbfc: {}", e),
            _ => {}
        }
    }

    fn set_fan_speed_to_auto(&self) {
        Command::new("nbfc")
            .arg("set")
            .arg("-a")
            .output()
            .expect("failed to set fan speed to auto");
    }
}

impl eframe::App for Veltrix {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.heading("Veltrix");
                ui.label("Notebook Fan Controller GUI");
            });

            ui.add_space(10.0);

            // 🔹 Config Section
            ui.group(|ui| {
                ui.heading("📂 Configuration");

                if !self.configs.is_empty() {
                    let selected_text = self
                        .selected
                        .map(|i| self.configs[i].as_str())
                        .unwrap_or("Select NBFC Config");

                    egui::ComboBox::from_label("Available configs")
                        .selected_text(selected_text)
                        .show_ui(ui, |ui| {
                            for (i, config) in self.configs.iter().enumerate() {
                                if ui
                                    .selectable_label(self.selected == Some(i), config)
                                    .clicked()
                                {
                                    self.selected = Some(i);
                                }
                            }
                        });

                    if let Some(i) = self.selected {
                        ui.colored_label(
                            egui::Color32::GREEN,
                            format!("✅ Selected: {}", self.configs[i]),
                        );
                    } else {
                        ui.colored_label(egui::Color32::YELLOW, "⚠ No config selected");
                    }
                } else {
                    ui.colored_label(egui::Color32::RED, "❌ No config found!");
                }
            });

            ui.add_space(10.0);

            ui.group(|ui| {
                ui.heading("📊 System Status");
                ui.label(format!("💻 Laptop: {}", self.laptop_model));
                ui.label(&self.temperature);
                ui.label(&self.current_fan_speed);
                ui.label(&self.target_fan_speed);

                if ui.button("🔄 Refresh").clicked() {
                    self.refresh();
                }
            });

            ui.add_space(10.0);

            ui.group(|ui| {
                ui.heading("⚡ Fan Control");

                ui.add(
                    egui::Slider::new(&mut self.fan_speed_value, 0..=96)
                        .text("Manual Fan Speed"),
                );

                ui.horizontal(|ui| {
                    if ui.button("✅ Apply").clicked() {
                        self.apply_fan_speed();
                        self.refresh();
                    }

                    if ui.button("♻ Set Auto").clicked() {
                        self.set_fan_speed_to_auto();
                        self.refresh();
                    }
                });
            });
        });
    }
}

fn main() -> eframe::Result<()> {
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size((400.0, 450.0)),
        ..eframe::NativeOptions::default()
    };

    eframe::run_native(
        Veltrix::name(),
        native_options,
        Box::new(|cc| Ok(Box::new(Veltrix::new(cc)))),
    )
}

fn read_laptop_model() -> String {
    fs::read_to_string("/sys/class/dmi/id/product_name")
        .unwrap_or_else(|_| "Unknown".into())
        .trim()
        .to_string()
}

fn get_config_list() -> Vec<String> {
    let output = Command::new("nbfc")
        .arg("config")
        .arg("--list")
        .output()
        .expect("failed to get config list");

    String::from_utf8_lossy(&output.stdout)
        .lines()
        .map(|line| line.trim().to_string())
        .filter(|line| !line.is_empty())
        .collect()
}

fn get_status() -> Vec<String> {
    let output = Command::new("nbfc")
        .arg("status")
        .arg("-a")
        .output()
        .expect("failed to get status");

    String::from_utf8_lossy(&output.stdout)
        .lines()
        .map(|line| line.trim().to_string())
        .filter(|line| !line.is_empty())
        .collect()
}
