//! Native GUI spike powered by egui on the wgpu backend.
//! 原生 GUI spike：`egui` 做交互层，底层渲染后端固定为 `wgpu`。

use std::{
    collections::VecDeque,
    path::PathBuf,
    time::Instant,
};

use eframe::egui::{self, Color32, FontId, Pos2, Rect, Sense, Shape, Stroke, StrokeKind, Vec2};

use crate::cell_vm::{cell_seed_a, cell_seed_b, cell_seed_g, CellConfig, CellType, CellWorld};
use crate::soup::{SoupLayout, SoupRenderItem};

#[derive(Debug, Clone, Copy, Default)]
pub struct GuiRuntime {
    pub smoke_test_frames: Option<u32>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SoupRendererMode {
    EguiPainterOnWgpu,
    CustomWgpuPassReserved,
}

#[derive(Debug, Clone)]
struct GuiSummary {
    population: usize,
    total_cells: usize,
    avg_energy: f32,
    avg_freshness: f32,
    avg_cells: f32,
    max_generation: u32,
    food_pool: i32,
    eat_ratio: f32,
    digest_ratio: f32,
    refresh_ratio: f32,
    divide_ratio: f32,
}

#[derive(Debug, Clone, Copy, Default)]
struct FrameTimingSample {
    sim_ms: f32,
    layout_ms: f32,
    ui_ms: f32,
    frame_ms: f32,
}

#[derive(Debug, Clone, Copy, Default)]
struct PerfSummary {
    samples: usize,
    fps: f32,
    avg_sim_ms: f32,
    avg_layout_ms: f32,
    avg_ui_ms: f32,
    avg_frame_ms: f32,
}

#[derive(Debug, Clone)]
struct PerfTracker {
    capacity: usize,
    samples: VecDeque<FrameTimingSample>,
}

impl PerfTracker {
    fn new(capacity: usize) -> Self {
        Self {
            capacity: capacity.max(1),
            samples: VecDeque::with_capacity(capacity.max(1)),
        }
    }

    fn record(&mut self, sample: FrameTimingSample) {
        if self.samples.len() == self.capacity {
            self.samples.pop_front();
        }
        self.samples.push_back(sample);
    }

    fn summary(&self) -> PerfSummary {
        if self.samples.is_empty() {
            return PerfSummary::default();
        }

        let samples = self.samples.len();
        let sum_sim_ms = self.samples.iter().map(|sample| sample.sim_ms).sum::<f32>();
        let sum_layout_ms = self.samples.iter().map(|sample| sample.layout_ms).sum::<f32>();
        let sum_ui_ms = self.samples.iter().map(|sample| sample.ui_ms).sum::<f32>();
        let sum_frame_ms = self.samples.iter().map(|sample| sample.frame_ms).sum::<f32>();
        let avg_frame_ms = sum_frame_ms / samples as f32;

        PerfSummary {
            samples,
            fps: if avg_frame_ms > 0.0 { 1000.0 / avg_frame_ms } else { 0.0 },
            avg_sim_ms: sum_sim_ms / samples as f32,
            avg_layout_ms: sum_layout_ms / samples as f32,
            avg_ui_ms: sum_ui_ms / samples as f32,
            avg_frame_ms,
        }
    }
}

pub fn run_cell_gui(config: CellConfig, runtime: GuiRuntime) -> Result<(), eframe::Error> {
    let native_options = eframe::NativeOptions {
        renderer: eframe::Renderer::Wgpu,
        viewport: egui::ViewportBuilder::default()
            .with_title("ExoMind Cell GUI Spike")
            .with_inner_size([1440.0, 920.0])
            .with_min_inner_size([1024.0, 720.0]),
        ..Default::default()
    };

    eframe::run_native(
        "ExoMind Cell GUI Spike",
        native_options,
        Box::new(move |creation_context| {
            configure_ui_fonts(&creation_context.egui_ctx);
            creation_context.egui_ctx.set_visuals(egui::Visuals::dark());
            Ok(Box::new(CellGuiApp::new(config, runtime)))
        }),
    )
}

fn configure_ui_fonts(ctx: &egui::Context) {
    let Some(font_path) = pick_first_existing_font_path(&ui_font_candidates()) else {
        eprintln!(
            "GUI font warning: no CJK font candidate found; Chinese text may render incorrectly."
        );
        return;
    };

    let Ok(font_bytes) = std::fs::read(&font_path) else {
        eprintln!(
            "GUI font warning: failed to read UI font {}; Chinese text may render incorrectly.",
            font_path.display()
        );
        return;
    };

    let font_name = format!(
        "exomind-cjk-{}",
        font_path
            .file_stem()
            .and_then(|stem| stem.to_str())
            .unwrap_or("fallback")
    );

    let mut fonts = egui::FontDefinitions::default();
    install_font_as_fallback(&mut fonts, &font_name, font_bytes);
    ctx.set_fonts(fonts);

    eprintln!("GUI font info: loaded CJK fallback font {}", font_path.display());
}

fn ui_font_candidates() -> Vec<PathBuf> {
    let mut candidates = Vec::new();

    if let Ok(font_override) = std::env::var("EXOMIND_CELL_UI_FONT") {
        let trimmed = font_override.trim();
        if !trimmed.is_empty() {
            candidates.push(PathBuf::from(trimmed));
        }
    }

    #[cfg(target_os = "windows")]
    {
        for path in [
            r"C:\Windows\Fonts\NotoSansSC-VF.ttf",
            r"C:\Windows\Fonts\simhei.ttf",
            r"C:\Windows\Fonts\Deng.ttf",
            r"C:\Windows\Fonts\simsunb.ttf",
        ] {
            candidates.push(PathBuf::from(path));
        }
    }

    #[cfg(target_os = "linux")]
    {
        for path in [
            "/usr/share/fonts/opentype/noto/NotoSansCJK-Regular.ttc",
            "/usr/share/fonts/opentype/noto/NotoSerifCJK-Regular.ttc",
            "/usr/share/fonts/truetype/noto/NotoSansCJK-Regular.ttc",
        ] {
            candidates.push(PathBuf::from(path));
        }
    }

    #[cfg(target_os = "macos")]
    {
        for path in [
            "/System/Library/Fonts/PingFang.ttc",
            "/System/Library/Fonts/Hiragino Sans GB.ttc",
            "/Library/Fonts/Arial Unicode.ttf",
        ] {
            candidates.push(PathBuf::from(path));
        }
    }

    candidates
}

fn pick_first_existing_font_path(candidates: &[PathBuf]) -> Option<PathBuf> {
    candidates.iter().find(|path| path.is_file()).cloned()
}

fn install_font_as_fallback(
    fonts: &mut egui::FontDefinitions,
    font_name: &str,
    font_bytes: Vec<u8>,
) {
    let family_name: egui::FontFamily = egui::FontFamily::Name(font_name.into());
    fonts.font_data.insert(
        font_name.into(),
        egui::FontData::from_owned(font_bytes).into(),
    );
    fonts
        .families
        .entry(family_name)
        .or_default()
        .push(font_name.into());
    fonts
        .families
        .entry(egui::FontFamily::Proportional)
        .or_default()
        .push(font_name.into());
    fonts
        .families
        .entry(egui::FontFamily::Monospace)
        .or_default()
        .push(font_name.into());
}

struct CellGuiApp {
    base_config: CellConfig,
    world: CellWorld,
    layout: SoupLayout,
    seed: u64,
    paused: bool,
    ticks_per_frame: u32,
    selected_id: Option<u64>,
    renderer_mode: SoupRendererMode,
    population_history: Vec<f32>,
    energy_history: Vec<f32>,
    perf: PerfTracker,
    last_frame: Instant,
    smoke_test_frames: Option<u32>,
}

impl CellGuiApp {
    fn new(config: CellConfig, runtime: GuiRuntime) -> Self {
        let seed = 42;
        let mut app = Self {
            base_config: config.clone(),
            world: build_seeded_world(&config, seed),
            layout: SoupLayout::new(),
            seed,
            paused: false,
            ticks_per_frame: 16,
            selected_id: None,
            renderer_mode: SoupRendererMode::EguiPainterOnWgpu,
            population_history: Vec::new(),
            energy_history: Vec::new(),
            perf: PerfTracker::new(120),
            last_frame: Instant::now(),
            smoke_test_frames: runtime.smoke_test_frames,
        };
        app.layout.sync_from_world(&app.world);
        app.push_history();
        app
    }

    fn reset_world(&mut self) {
        self.world = build_seeded_world(&self.base_config, self.seed);
        self.layout = SoupLayout::new();
        self.layout.sync_from_world(&self.world);
        self.population_history.clear();
        self.energy_history.clear();
        self.selected_id = None;
        self.last_frame = Instant::now();
        self.push_history();
    }

    fn step_world_once(&mut self) {
        if self.world.tick < self.base_config.total_ticks {
            self.world.tick();
        }
        self.layout.sync_from_world(&self.world);
        self.push_history();
    }

    fn advance_with_dt(&mut self, dt: f32) -> FrameTimingSample {
        let mut sample = FrameTimingSample::default();
        let now = Instant::now();
        let mut advanced = false;
        let sim_start = now;
        if !self.paused {
            for _ in 0..self.ticks_per_frame {
                if self.world.tick >= self.base_config.total_ticks {
                    self.paused = true;
                    break;
                }
                self.world.tick();
                advanced = true;
            }
            sample.sim_ms = sim_start.elapsed().as_secs_f32() * 1000.0;

            let layout_start = Instant::now();
            self.layout.step(&self.world, dt);
            sample.layout_ms = layout_start.elapsed().as_secs_f32() * 1000.0;
        }

        if advanced {
            self.push_history();
        }

        if let Some(selected_id) = self.selected_id {
            let still_alive = self
                .world
                .organisms
                .iter()
                .any(|organism| organism.id == selected_id && organism.alive);
            if !still_alive {
                self.selected_id = None;
            }
        }

        sample
    }

    fn advance_frame(&mut self) -> FrameTimingSample {
        let now = Instant::now();
        let dt = now
            .saturating_duration_since(self.last_frame)
            .as_secs_f32()
            .clamp(1.0 / 240.0, 1.0 / 20.0);
        self.last_frame = now;
        self.advance_with_dt(dt)
    }

    fn push_history(&mut self) {
        let summary = summarize_world(&self.world);
        self.population_history.push(summary.population as f32);
        self.energy_history.push(summary.avg_energy);

        const MAX_HISTORY: usize = 240;
        if self.population_history.len() > MAX_HISTORY {
            let overflow = self.population_history.len() - MAX_HISTORY;
            self.population_history.drain(0..overflow);
            self.energy_history.drain(0..overflow);
        }
    }

    fn selected_organism(&self) -> Option<&crate::cell_vm::CellOrganism> {
        self.selected_id.and_then(|selected_id| {
            self.world
                .organisms
                .iter()
                .find(|organism| organism.id == selected_id && organism.alive)
        })
    }
}

impl eframe::App for CellGuiApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let perf_summary = self.perf.summary();
        let frame_start = Instant::now();
        let mut frame_timing = self.advance_frame();

        egui::TopBottomPanel::top("toolbar").show(ctx, |ui| {
            ui.horizontal_wrapped(|ui| {
                if ui
                    .button(if self.paused { "Resume" } else { "Pause" })
                    .clicked()
                {
                    self.paused = !self.paused;
                }
                if ui.button("Step").clicked() {
                    self.paused = true;
                    self.step_world_once();
                }
                if ui.button("Reset").clicked() {
                    self.reset_world();
                }

                ui.separator();
                ui.label("Seed");
                ui.add(egui::DragValue::new(&mut self.seed).range(1..=u64::MAX).speed(1.0));
                if ui.button("Apply Seed").clicked() {
                    self.reset_world();
                }

                ui.separator();
                ui.label("Ticks / frame");
                ui.add(egui::Slider::new(&mut self.ticks_per_frame, 1..=256));

                ui.separator();
                ui.label(format!("Tick {}", self.world.tick));
                ui.label(format!("Renderer {:?}", self.renderer_mode));
                ui.label("wgpu backend active");
            });
        });

        egui::SidePanel::right("stats_panel")
            .min_width(320.0)
            .default_width(360.0)
            .show(ctx, |ui| {
                let summary = summarize_world(&self.world);
                ui.heading("Cell v3 GUI Spike");
                ui.label("Positions are GUI-only layout state; VM semantics stay global.");
                ui.small("可视化坐标仅用于显示，不回写到 CellWorld。");
                ui.separator();

                labeled_value(ui, "Population", format!("{}", summary.population));
                labeled_value(ui, "Total Cells", format!("{}", summary.total_cells));
                labeled_value(ui, "Food Pool", format!("{}", summary.food_pool));
                labeled_value(ui, "Avg Energy", format!("{:.1}", summary.avg_energy));
                labeled_value(ui, "Avg Freshness", format!("{:.1}", summary.avg_freshness));
                labeled_value(ui, "Avg Cells", format!("{:.1}", summary.avg_cells));
                labeled_value(ui, "Max Generation", format!("{}", summary.max_generation));
                labeled_value(ui, "EAT / DIGEST", format!("{:.1}% / {:.1}%", summary.eat_ratio, summary.digest_ratio));
                labeled_value(
                    ui,
                    "REFRESH / DIVIDE",
                    format!("{:.1}% / {:.1}%", summary.refresh_ratio, summary.divide_ratio),
                );

                ui.separator();
                ui.heading("Performance / 性能");
                ui.small("CPU metrics only; GPU present time is not included.");
                ui.small("这里只统计 CPU 侧时间，不代表 GPU 实际提交/显示耗时。");
                labeled_value(ui, "Samples", perf_summary.samples.to_string());
                labeled_value(ui, "FPS", format!("{:.1}", perf_summary.fps));
                labeled_value(ui, "CPU Frame", format!("{:.2} ms", perf_summary.avg_frame_ms));
                labeled_value(ui, "CPU Sim", format!("{:.2} ms", perf_summary.avg_sim_ms));
                labeled_value(ui, "CPU Layout", format!("{:.2} ms", perf_summary.avg_layout_ms));
                labeled_value(ui, "CPU UI", format!("{:.2} ms", perf_summary.avg_ui_ms));

                ui.separator();
                ui.label("History");
                draw_sparkline(
                    ui,
                    "Population trend",
                    &self.population_history,
                    Color32::from_rgb(88, 200, 255),
                );
                draw_sparkline(
                    ui,
                    "Energy trend",
                    &self.energy_history,
                    Color32::from_rgb(255, 188, 56),
                );

                ui.separator();
                ui.horizontal(|ui| {
                    ui.label("Render path");
                    egui::ComboBox::from_id_salt("renderer_mode")
                        .selected_text(match self.renderer_mode {
                            SoupRendererMode::EguiPainterOnWgpu => "egui painter on wgpu",
                            SoupRendererMode::CustomWgpuPassReserved => "custom wgpu pass (reserved)",
                        })
                        .show_ui(ui, |ui| {
                            ui.selectable_value(
                                &mut self.renderer_mode,
                                SoupRendererMode::EguiPainterOnWgpu,
                                "egui painter on wgpu",
                            );
                            ui.add_enabled_ui(false, |ui| {
                                ui.selectable_value(
                                    &mut self.renderer_mode,
                                    SoupRendererMode::CustomWgpuPassReserved,
                                    "custom wgpu pass (reserved)",
                                );
                            });
                        });
                });

                ui.separator();
                ui.heading("How To Use / 使用说明");
                ui.collapsing("Controls / 控件", |ui| {
                    ui.label("Pause: stop VM ticks and freeze layout.");
                    ui.small("暂停：停止 VM tick，并冻结汤视图布局。");
                    ui.label("Step: advance one simulation step while paused.");
                    ui.small("单步：在暂停状态下推进一次模拟。");
                    ui.label("Reset / Apply Seed: rebuild the seeded world.");
                    ui.small("重置 / 应用种子：重新构建当前种子下的世界。");
                    ui.label("Ticks / frame: more ticks means faster evolution but heavier CPU load.");
                    ui.small("每帧 tick 数越大，进化越快，但 CPU 负载也越高。");
                });
                ui.collapsing("Visual Encoding / 视觉编码", |ui| {
                    ui.label("Color: red -> green means low -> high energy.");
                    ui.small("颜色：红到绿表示能量从低到高。");
                    ui.label("Alpha: transparent -> solid means low -> high freshness.");
                    ui.small("透明度：越透明 freshness 越低，越实心 freshness 越高。");
                    ui.label("Radius: larger circles contain more cells.");
                    ui.small("半径：越大表示该生命体包含的 cell 越多。");
                    ui.label("Edge drift is layout equilibrium, not real movement semantics.");
                    ui.small("跑到边缘只是布局平衡结果，不是 VM 里真实“移动”。");
                });

                ui.separator();
                ui.heading("Inspector");
                if let Some(organism) = self.selected_organism() {
                    let mut code_cells = 0;
                    let mut energy_cells = 0;
                    let mut stomach_cells = 0;
                    let mut data_cells = 0;
                    for cell in &organism.cells {
                        match cell.content {
                            CellType::Code(_) => code_cells += 1,
                            CellType::Energy(_) => energy_cells += 1,
                            CellType::Stomach(_) => stomach_cells += 1,
                            CellType::Data(_) => data_cells += 1,
                        }
                    }

                    labeled_value(ui, "Organism", format!("#{}", organism.id));
                    labeled_value(ui, "Generation", organism.generation.to_string());
                    labeled_value(ui, "Age", organism.age.to_string());
                    labeled_value(ui, "Instruction Pointer", organism.ip.to_string());
                    labeled_value(ui, "Total Energy", organism.total_energy().to_string());
                    labeled_value(ui, "Min Freshness", organism.min_freshness().to_string());
                    labeled_value(
                        ui,
                        "Cells",
                        format!(
                            "Code {} | Energy {} | Stomach {} | Data {}",
                            code_cells, energy_cells, stomach_cells, data_cells
                        ),
                    );
                    ui.separator();
                    ui.label("Registers");
                    ui.monospace(format!("{:?}", organism.registers));
                } else {
                    ui.label("Click a circle in the soup view to inspect an organism.");
                }
            });

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Soup View");
            ui.small("Force-directed layout rendered by egui on top of the wgpu backend.");

            let desired_size = ui.available_size();
            let (response, painter) = ui.allocate_painter(desired_size, Sense::click());
            let scene_rect = response.rect;
            let snapshot = self.layout.snapshot(&self.world);
            let summary = summarize_world(&self.world);

            draw_soup_scene(&painter, scene_rect, &snapshot, self.selected_id);
            draw_perf_overlay(
                &painter,
                scene_rect,
                perf_summary,
                &summary,
                snapshot.len(),
                self.world.tick,
                self.ticks_per_frame,
                self.paused,
            );

            if response.clicked() {
                if let Some(pointer_pos) = response.interact_pointer_pos() {
                    self.selected_id = pick_organism(&snapshot, scene_rect, pointer_pos);
                }
            }
        });

        frame_timing.frame_ms = frame_start.elapsed().as_secs_f32() * 1000.0;
        frame_timing.ui_ms = (frame_timing.frame_ms - frame_timing.sim_ms - frame_timing.layout_ms).max(0.0);
        self.perf.record(frame_timing);

        ctx.request_repaint();

        if let Some(frames_left) = &mut self.smoke_test_frames {
            if *frames_left == 0 {
                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            } else {
                *frames_left -= 1;
            }
        }
    }
}

fn build_seeded_world(config: &CellConfig, seed: u64) -> CellWorld {
    let mut world = CellWorld::new(config.clone(), seed);
    for _ in 0..10 {
        world.add_organism(cell_seed_a(config));
    }
    for _ in 0..10 {
        world.add_organism(cell_seed_b(config));
    }
    for _ in 0..10 {
        world.add_organism(cell_seed_g(config));
    }
    world
}

fn summarize_world(world: &CellWorld) -> GuiSummary {
    let alive: Vec<_> = world.organisms.iter().filter(|organism| organism.alive).collect();
    if alive.is_empty() {
        return GuiSummary {
            population: 0,
            total_cells: 0,
            avg_energy: 0.0,
            avg_freshness: 0.0,
            avg_cells: 0.0,
            max_generation: 0,
            food_pool: world.food_pool,
            eat_ratio: 0.0,
            digest_ratio: 0.0,
            refresh_ratio: 0.0,
            divide_ratio: 0.0,
        };
    }

    let total_instructions = alive
        .iter()
        .map(|organism| organism.total_instructions)
        .sum::<u64>()
        .max(1) as f32;

    GuiSummary {
        population: alive.len(),
        total_cells: alive.iter().map(|organism| organism.cells.len()).sum(),
        avg_energy: alive.iter().map(|organism| organism.total_energy() as f32).sum::<f32>()
            / alive.len() as f32,
        avg_freshness: alive
            .iter()
            .map(|organism| organism.min_freshness() as f32)
            .sum::<f32>()
            / alive.len() as f32,
        avg_cells: alive
            .iter()
            .map(|organism| organism.cells.len() as f32)
            .sum::<f32>()
            / alive.len() as f32,
        max_generation: alive
            .iter()
            .map(|organism| organism.generation)
            .max()
            .unwrap_or(0),
        food_pool: world.food_pool,
        eat_ratio: alive.iter().map(|organism| organism.eat_count as f32).sum::<f32>() * 100.0
            / total_instructions,
        digest_ratio: alive
            .iter()
            .map(|organism| organism.digest_count as f32)
            .sum::<f32>()
            * 100.0
            / total_instructions,
        refresh_ratio: alive
            .iter()
            .map(|organism| organism.refresh_count as f32)
            .sum::<f32>()
            * 100.0
            / total_instructions,
        divide_ratio: alive
            .iter()
            .map(|organism| organism.divide_count as f32)
            .sum::<f32>()
            * 100.0
            / total_instructions,
    }
}

fn labeled_value(ui: &mut egui::Ui, label: &str, value: String) {
    ui.horizontal(|ui| {
        ui.label(label);
        ui.monospace(value);
    });
}

fn draw_sparkline(ui: &mut egui::Ui, label: &str, values: &[f32], color: Color32) {
    ui.label(label);
    let desired_size = Vec2::new(ui.available_width(), 64.0);
    let (response, painter) = ui.allocate_painter(desired_size, Sense::hover());
    let rect = response.rect;

    painter.rect_filled(rect, 8.0, Color32::from_rgb(18, 23, 31));
    painter.rect_stroke(
        rect,
        8.0,
        Stroke::new(1.0, Color32::from_gray(48)),
        StrokeKind::Outside,
    );

    if values.len() < 2 {
        return;
    }

    let min_value = values.iter().copied().fold(f32::INFINITY, f32::min);
    let max_value = values.iter().copied().fold(f32::NEG_INFINITY, f32::max);
    let range = (max_value - min_value).max(1.0);

    let mut points = Vec::with_capacity(values.len());
    for (index, value) in values.iter().enumerate() {
        let t = index as f32 / (values.len().saturating_sub(1)) as f32;
        let x = egui::lerp(rect.left()..=rect.right(), t);
        let normalized = (*value - min_value) / range;
        let y = egui::lerp((rect.bottom() - 6.0)..=(rect.top() + 6.0), normalized);
        points.push(Pos2::new(x, y));
    }
    painter.add(Shape::line(points, Stroke::new(2.0, color)));
}

fn draw_soup_scene(
    painter: &egui::Painter,
    rect: Rect,
    items: &[SoupRenderItem],
    selected_id: Option<u64>,
) {
    painter.rect_filled(rect, 18.0, Color32::from_rgb(10, 14, 22));

    for index in 0..16 {
        let t = index as f32 / 15.0;
        let x = egui::lerp(rect.left()..=rect.right(), t);
        let y = egui::lerp(rect.top()..=rect.bottom(), t);
        let grid_color = Color32::from_rgba_unmultiplied(56, 82, 110, 18);
        painter.line_segment(
            [Pos2::new(x, rect.top()), Pos2::new(x, rect.bottom())],
            Stroke::new(1.0, grid_color),
        );
        painter.line_segment(
            [Pos2::new(rect.left(), y), Pos2::new(rect.right(), y)],
            Stroke::new(1.0, grid_color),
        );
    }

    if items.is_empty() {
        painter.text(
            rect.center(),
            egui::Align2::CENTER_CENTER,
            "No alive organisms",
            FontId::proportional(18.0),
            Color32::from_gray(180),
        );
        return;
    }

    for item in items {
        let center = world_to_screen(rect, item);
        let radius = item.radius.clamp(6.0, 24.0);
        let fill = soup_color(item.energy_ratio, item.freshness_ratio);
        painter.circle_filled(center, radius, fill);
        painter.circle_stroke(
            center,
            radius,
            Stroke::new(1.5, Color32::from_rgba_unmultiplied(255, 255, 255, 80)),
        );

        if selected_id == Some(item.organism_id) {
            painter.circle_stroke(center, radius + 5.0, Stroke::new(2.0, Color32::WHITE));
            painter.text(
                center + Vec2::new(radius + 8.0, -(radius + 8.0)),
                egui::Align2::LEFT_BOTTOM,
                format!("#{} gen {} cells {}", item.organism_id, item.generation, item.cell_count),
                FontId::monospace(13.0),
                Color32::WHITE,
            );
        }
    }
}

fn world_to_screen(rect: Rect, item: &SoupRenderItem) -> Pos2 {
    Pos2::new(
        egui::lerp(rect.left()..=rect.right(), item.x),
        egui::lerp(rect.top()..=rect.bottom(), item.y),
    )
}

fn draw_perf_overlay(
    painter: &egui::Painter,
    rect: Rect,
    perf: PerfSummary,
    summary: &GuiSummary,
    particles: usize,
    tick: u64,
    ticks_per_frame: u32,
    paused: bool,
) {
    let lines = [
        format!("Status      {}", if paused { "PAUSED" } else { "RUNNING" }),
        format!("Tick        {}", tick),
        format!("Ticks/frame {}", ticks_per_frame),
        format!("Population  {}", summary.population),
        format!("Cells       {}", summary.total_cells),
        format!("Particles   {}", particles),
        format!("CPU FPS     {:.1}", perf.fps),
        format!("CPU frame   {:.2} ms", perf.avg_frame_ms),
        format!("CPU sim     {:.2} ms", perf.avg_sim_ms),
        format!("CPU layout  {:.2} ms", perf.avg_layout_ms),
        format!("CPU ui      {:.2} ms", perf.avg_ui_ms),
    ];

    let panel_size = Vec2::new(232.0, 12.0 + lines.len() as f32 * 18.0);
    let panel_rect = Rect::from_min_size(
        Pos2::new(rect.right() - panel_size.x - 16.0, rect.top() + 16.0),
        panel_size,
    );

    painter.rect_filled(
        panel_rect,
        12.0,
        Color32::from_rgba_unmultiplied(7, 11, 18, 220),
    );
    painter.rect_stroke(
        panel_rect,
        12.0,
        Stroke::new(1.0, Color32::from_rgba_unmultiplied(180, 206, 255, 48)),
        StrokeKind::Outside,
    );

    for (index, line) in lines.iter().enumerate() {
        painter.text(
            Pos2::new(panel_rect.left() + 12.0, panel_rect.top() + 10.0 + index as f32 * 18.0),
            egui::Align2::LEFT_TOP,
            line,
            FontId::monospace(12.0),
            Color32::from_rgb(226, 232, 244),
        );
    }
}

fn pick_organism(items: &[SoupRenderItem], rect: Rect, pointer_pos: Pos2) -> Option<u64> {
    items
        .iter()
        .filter_map(|item| {
            let center = world_to_screen(rect, item);
            let distance = center.distance(pointer_pos);
            (distance <= item.radius + 8.0).then_some((item.organism_id, distance))
        })
        .min_by(|left, right| left.1.total_cmp(&right.1))
        .map(|(organism_id, _)| organism_id)
}

fn soup_color(energy_ratio: f32, freshness_ratio: f32) -> Color32 {
    let energy = energy_ratio.clamp(0.0, 1.0);
    let freshness = freshness_ratio.clamp(0.0, 1.0);
    let red = egui::lerp(225.0..=74.0, energy) as u8;
    let green = egui::lerp(92.0..=214.0, energy) as u8;
    let blue = egui::lerp(76.0..=141.0, energy) as u8;
    let alpha = egui::lerp(96.0..=255.0, freshness) as u8;
    Color32::from_rgba_unmultiplied(red, green, blue, alpha)
}

#[cfg(test)]
mod tests {
    use std::{
        fs,
        time::{Duration, Instant},
    };

    use super::*;

    fn test_config() -> CellConfig {
        let mut config = CellConfig::experimental();
        config.cell_energy_max = 50;
        config.food_per_tick = 500;
        config.max_organisms = 200;
        config.total_ticks = 2_000;
        config.snapshot_interval = 100;
        config.genome_dump_interval = 0;
        config
    }

    #[test]
    fn test_paused_gui_frame_keeps_layout_positions_stable() {
        let mut app = CellGuiApp::new(test_config(), GuiRuntime::default());
        app.paused = true;
        app.last_frame = Instant::now() - Duration::from_millis(16);

        let before = app
            .layout
            .snapshot(&app.world)
            .into_iter()
            .map(|item| (item.organism_id, item.x, item.y))
            .collect::<Vec<_>>();

        app.advance_frame();

        let after = app
            .layout
            .snapshot(&app.world)
            .into_iter()
            .map(|item| (item.organism_id, item.x, item.y))
            .collect::<Vec<_>>();

        assert_eq!(
            before, after,
            "Paused GUI frames should freeze soup layout positions",
        );
    }

    #[test]
    fn test_perf_tracker_reports_windowed_cpu_metrics() {
        let mut tracker = PerfTracker::new(3);

        tracker.record(FrameTimingSample {
            sim_ms: 4.0,
            layout_ms: 2.0,
            ui_ms: 3.0,
            frame_ms: 16.0,
        });
        tracker.record(FrameTimingSample {
            sim_ms: 6.0,
            layout_ms: 3.0,
            ui_ms: 5.0,
            frame_ms: 20.0,
        });
        tracker.record(FrameTimingSample {
            sim_ms: 8.0,
            layout_ms: 4.0,
            ui_ms: 6.0,
            frame_ms: 24.0,
        });
        tracker.record(FrameTimingSample {
            sim_ms: 10.0,
            layout_ms: 5.0,
            ui_ms: 7.0,
            frame_ms: 30.0,
        });

        let summary = tracker.summary();

        assert_eq!(summary.samples, 3);
        assert!((summary.avg_frame_ms - 24.666666).abs() < 0.001);
        assert!((summary.avg_sim_ms - 8.0).abs() < 0.001);
        assert!((summary.avg_layout_ms - 4.0).abs() < 0.001);
        assert!((summary.avg_ui_ms - 6.0).abs() < 0.001);
        assert!((summary.fps - (1000.0 / 24.666666)).abs() < 0.01);
    }

    #[test]
    fn test_pick_first_existing_font_path_prefers_earliest_existing_candidate() {
        let temp_root = std::env::temp_dir().join(format!(
            "exomind-cell-font-test-{}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&temp_root);
        fs::create_dir_all(&temp_root).expect("create temp font dir");

        let second = temp_root.join("second.ttf");
        let first = temp_root.join("first.ttf");
        fs::write(&second, b"dummy second").expect("write second");
        fs::write(&first, b"dummy first").expect("write first");

        let candidates = vec![
            temp_root.join("missing.ttf"),
            first.clone(),
            second,
        ];

        let selected = pick_first_existing_font_path(&candidates);
        assert_eq!(selected, Some(first));

        let _ = fs::remove_dir_all(&temp_root);
    }

    #[test]
    fn test_install_font_as_fallback_registers_both_default_families() {
        let mut fonts = egui::FontDefinitions::default();
        let font_name = "test-cjk-fallback";

        install_font_as_fallback(&mut fonts, font_name, vec![1, 2, 3, 4]);

        let proportional = fonts
            .families
            .get(&egui::FontFamily::Proportional)
            .expect("proportional family");
        let monospace = fonts
            .families
            .get(&egui::FontFamily::Monospace)
            .expect("monospace family");

        assert!(
            proportional.iter().any(|name| name == font_name),
            "Proportional family should include the CJK fallback font",
        );
        assert!(
            monospace.iter().any(|name| name == font_name),
            "Monospace family should include the CJK fallback font",
        );
        assert!(
            fonts.font_data.contains_key(font_name),
            "Font data should be registered under the same name",
        );
    }
}
