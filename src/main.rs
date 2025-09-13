use eframe::egui;
use std::fs;
use std::process::Command;

struct Veltrix {
    configs: Vec<String>,
    selected: Option<usize>,
    laptop_model: String,
}

impl Veltrix {
    fn name() -> &'static str {
        "Veltrix"
    }

    fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        let configs = get_nbfc_list();
        let laptop_model = read_laptop_model();

        let selected = configs.iter().position(|c| c == &laptop_model);

        Self {
            configs,
            selected,
            laptop_model,
        }
    }
}

impl eframe::App for Veltrix {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Welcome to Veltrix !\n");
            if self.selected.is_none() {
                ui.label("Please select a config");
            } else {
                ui.label(format!("Laptop model: {}", self.laptop_model));
            }

            if !self.configs.is_empty() {
                let selected_text = self
                    .selected
                    .map(|i| self.configs[i].as_str())
                    .unwrap_or("Select NBFC Config");

                egui::ComboBox::from_label("Config List\n")
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
                    ui.colored_label(egui::Color32::GREEN, format!("Selected Config: {}", self.configs[i]));
                } else {
                    ui.colored_label(egui::Color32::YELLOW, "No config selected!");
                }
            } else {
                ui.colored_label(egui::Color32::RED, "No config found!");
            }
        });
    }
}

fn main() -> eframe::Result<()> {
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size((600.0, 700.0)),
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

fn get_nbfc_list() -> Vec<String> {
    let output = Command::new("nbfc")
        .arg("config")
        .arg("--list")
        .output()
        .expect("failed to execute nbfc");

    String::from_utf8_lossy(&output.stdout)
        .lines()
        .map(|line| line.trim().to_string())
        .filter(|line| !line.is_empty())
        .collect()
}
