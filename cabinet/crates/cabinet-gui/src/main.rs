use eframe::NativeOptions;

fn main() -> eframe::Result<()> {
    let options = NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1280.0, 720.0])
            .with_min_inner_size([800.0, 600.0]),
        ..Default::default()
    };
    eframe::run_native(
        "Cabinet GUI",
        options,
        Box::new(|cc| Box::new(CabinetApp::new(cc))),
    )
}

pub struct CabinetApp {
    input_text: String,
    query_text: String,
    results: Vec<String>,
    status: String,
}

impl CabinetApp {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        CabinetApp {
            input_text: String::new(),
            query_text: String::new(),
            results: Vec::new(),
            status: "Ready".to_string(),
        }
    }
}

impl eframe::App for CabinetApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Cabinet - HSH Memory Explorer");
            ui.separator();

            ui.horizontal(|ui| {
                ui.label("插入文本:");
                ui.text_edit_multiline(&mut self.input_text);
                if ui.button("Insert").clicked() {
                    self.status = format!("Inserted: {} chars", self.input_text.len());
                }
            });

            ui.separator();

            ui.horizontal(|ui| {
                ui.label("查询:");
                ui.text_edit_singleline(&mut self.query_text);
                if ui.button("Query").clicked() {
                    self.results = vec![
                        format!("Query: {}", self.query_text),
                        "Result 1: doc=1, score=0.95".to_string(),
                        "Result 2: doc=2, score=0.82".to_string(),
                    ];
                }
            });

            ui.separator();
            ui.label("Results:");
            for r in &self.results {
                ui.label(r);
            }

            ui.separator();
            ui.label(format!("Status: {}", self.status));
        });
    }
}
