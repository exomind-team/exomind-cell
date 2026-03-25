//! Terminal UI for real-time D0 VM visualization using ratatui.

use std::io;
use std::time::{Duration, Instant};

use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use ratatui::{
    prelude::*,
    widgets::*,
};

use crate::organism::{seed_a, seed_b, seed_c, Config};
use crate::world::World;

/// Live statistics collected from the world each frame.
struct LiveStats {
    tick: u64,
    population: usize,
    avg_energy: f64,
    avg_freshness: f64,
    avg_code_len: f64,
    max_generation: u32,
    food_pool: i32,
    // Instruction ratios (from interval counters)
    eat_pct: f64,
    refresh_pct: f64,
    divide_pct: f64,
    emit_pct: f64,
    sample_pct: f64,
    other_pct: f64,
    // Medium heat (sum of all medium values)
    medium_heat: u64,
    medium_max: u8,
    // History for sparkline
    pop_history: Vec<u64>,
    energy_history: Vec<u64>,
}

impl LiveStats {
    fn from_world(world: &World, pop_history: &[u64], energy_history: &[u64]) -> Self {
        let alive: Vec<&crate::organism::Organism> =
            world.organisms.iter().filter(|o| o.alive).collect();
        let n = alive.len();

        let (avg_energy, avg_freshness, avg_code_len, max_gen) = if n > 0 {
            (
                alive.iter().map(|o| o.energy as f64).sum::<f64>() / n as f64,
                alive.iter().map(|o| o.freshness as f64).sum::<f64>() / n as f64,
                alive.iter().map(|o| o.code.len() as f64).sum::<f64>() / n as f64,
                alive.iter().map(|o| o.generation).max().unwrap_or(0),
            )
        } else {
            (0.0, 0.0, 0.0, 0)
        };

        // Instruction ratios from interval counters
        let total = world.interval_instructions().max(1) as f64;
        let eat = world.interval_eat() as f64 / total;
        let refresh = world.interval_refresh() as f64 / total;
        let divide = world.interval_divide() as f64 / total;
        let emit = world.interval_emit() as f64 / total;
        let sample = world.interval_sample() as f64 / total;
        let other = 1.0 - eat - refresh - divide - emit - sample;

        let medium_heat: u64 = world.medium.iter().map(|&v| v as u64).sum();
        let medium_max = world.medium.iter().copied().max().unwrap_or(0);

        LiveStats {
            tick: world.tick,
            population: n,
            avg_energy,
            avg_freshness,
            avg_code_len,
            max_generation: max_gen,
            food_pool: world.food_pool,
            eat_pct: eat * 100.0,
            refresh_pct: refresh * 100.0,
            divide_pct: divide * 100.0,
            emit_pct: emit * 100.0,
            sample_pct: sample * 100.0,
            other_pct: other.max(0.0) * 100.0,
            medium_heat,
            medium_max,
            pop_history: pop_history.to_vec(),
            energy_history: energy_history.to_vec(),
        }
    }
}

/// Run the TUI visualization mode.
pub fn run_tui(config: Config) -> io::Result<()> {
    // Setup terminal
    enable_raw_mode()?;
    io::stdout().execute(EnterAlternateScreen)?;
    let mut terminal = Terminal::new(CrosstermBackend::new(io::stdout()))?;

    let mut world = World::new(config.clone(), 42);

    // Seed organisms
    for _ in 0..5 { world.add_organism(seed_a(&config)); }
    for _ in 0..5 { world.add_organism(seed_b(&config)); }
    if config.medium_size > 0 {
        for _ in 0..10 { world.add_organism(seed_c(&config)); }
    } else {
        for _ in 0..10 { world.add_organism(seed_b(&config)); }
    }

    let mut pop_history: Vec<u64> = Vec::new();
    let mut energy_history: Vec<u64> = Vec::new();
    let mut ticks_per_frame: u64 = 50;
    let mut paused = false;
    let frame_duration = Duration::from_millis(100); // 10 FPS (stable, no flicker)

    loop {
        let frame_start = Instant::now();

        // Handle input
        if event::poll(Duration::from_millis(0))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    match key.code {
                        KeyCode::Char('q') => break,
                        KeyCode::Char('p') => paused = !paused,
                        KeyCode::Char('s') if paused => { world.tick(); },
                        KeyCode::Char('+') | KeyCode::Char('=') => {
                            ticks_per_frame = (ticks_per_frame * 2).min(1000);
                        },
                        KeyCode::Char('-') => {
                            ticks_per_frame = (ticks_per_frame / 2).max(1);
                        },
                        _ => {}
                    }
                }
            }
        }

        // Run simulation ticks (unless paused)
        if !paused {
            for _ in 0..ticks_per_frame {
                if world.tick >= config.total_ticks { break; }
                world.tick();
            }
        }

        // Collect history (every frame = every 50 ticks)
        let alive_count = world.organisms.iter().filter(|o| o.alive).count();
        pop_history.push(alive_count as u64);
        let avg_e = if alive_count > 0 {
            world.organisms.iter().filter(|o| o.alive)
                .map(|o| o.energy as u64).sum::<u64>() / alive_count as u64
        } else { 0 };
        energy_history.push(avg_e);

        // Keep history bounded
        let max_history = 200;
        if pop_history.len() > max_history {
            pop_history.drain(0..pop_history.len() - max_history);
            energy_history.drain(0..energy_history.len() - max_history);
        }

        let stats = LiveStats::from_world(&world, &pop_history, &energy_history);

        // Draw
        terminal.draw(|frame| draw_ui(frame, &stats, &config))?;

        // Check if simulation ended
        if world.tick >= config.total_ticks {
            // Wait for 'q' to quit
            loop {
                if let Event::Key(key) = event::read()? {
                    if key.kind == KeyEventKind::Press && key.code == KeyCode::Char('q') {
                        break;
                    }
                }
            }
            break;
        }

        // Frame rate limiting
        let elapsed = frame_start.elapsed();
        if elapsed < frame_duration {
            std::thread::sleep(frame_duration - elapsed);
        }
    }

    // Restore terminal
    disable_raw_mode()?;
    io::stdout().execute(LeaveAlternateScreen)?;
    Ok(())
}

fn draw_ui(frame: &mut Frame, stats: &LiveStats, config: &Config) {
    let area = frame.area();

    // Main layout: header + body
    let main_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),  // Header
            Constraint::Min(0),    // Body
            Constraint::Length(1), // Footer
        ])
        .split(area);

    // Header
    let header_text = format!(
        " D0 VM | Tick: {}/{} ({:.0}%) | Pop: {} | Food: {} | Gen: {} ",
        stats.tick, config.total_ticks,
        (stats.tick as f64 / config.total_ticks as f64) * 100.0,
        stats.population, stats.food_pool, stats.max_generation,
    );
    let header = Paragraph::new(header_text)
        .block(Block::default().borders(Borders::ALL).title(" D0 Virtual Machine "))
        .style(Style::default().fg(Color::Cyan).bold());
    frame.render_widget(header, main_layout[0]);

    // Body: split into left (stats + bars) and right (sparklines)
    let body_layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(50),
            Constraint::Percentage(50),
        ])
        .split(main_layout[1]);

    // Left panel: stats + instruction bars
    let left_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(8),  // Stats
            Constraint::Length(5),  // Instruction bars
            Constraint::Min(0),    // Medium heat (if enabled)
        ])
        .split(body_layout[0]);

    // Stats block
    let stats_text = vec![
        Line::from(vec![
            Span::raw("  Avg Energy:    "),
            Span::styled(format!("{:.0}", stats.avg_energy), Style::default().fg(Color::Yellow)),
            Span::raw(format!(" / {}", config.e_max)),
        ]),
        Line::from(vec![
            Span::raw("  Avg Freshness: "),
            Span::styled(format!("{:.0}", stats.avg_freshness), Style::default().fg(
                if stats.avg_freshness > 200.0 { Color::Green }
                else if stats.avg_freshness > 100.0 { Color::Yellow }
                else { Color::Red }
            )),
            Span::raw(format!(" / {}", config.freshness_max)),
        ]),
        Line::from(vec![
            Span::raw("  Avg Code Len:  "),
            Span::styled(format!("{:.1}", stats.avg_code_len), Style::default().fg(Color::Magenta)),
        ]),
        Line::from(vec![
            Span::raw("  Freshness Decay: "),
            Span::styled(
                if config.freshness_decay { "ON" } else { "OFF" },
                Style::default().fg(if config.freshness_decay { Color::Red } else { Color::Gray }),
            ),
        ]),
    ];
    let stats_widget = Paragraph::new(stats_text)
        .block(Block::default().borders(Borders::ALL).title(" Statistics "));
    frame.render_widget(stats_widget, left_layout[0]);

    // Instruction ratio bars
    let bar_data: Vec<(&str, u64)> = vec![
        ("EAT", (stats.eat_pct * 10.0) as u64),
        ("REF", (stats.refresh_pct * 10.0) as u64),
        ("DIV", (stats.divide_pct * 10.0) as u64),
        ("EMT", (stats.emit_pct * 10.0) as u64),
        ("SMP", (stats.sample_pct * 10.0) as u64),
        ("OTH", (stats.other_pct * 10.0) as u64),
    ];
    let barchart = BarChart::default()
        .block(Block::default().borders(Borders::ALL).title(format!(
            " Instructions (EAT {:.0}% | REF {:.0}% | DIV {:.0}%) ",
            stats.eat_pct, stats.refresh_pct, stats.divide_pct
        )))
        .data(&bar_data)
        .bar_width(5)
        .bar_gap(1)
        .bar_style(Style::default().fg(Color::Green))
        .value_style(Style::default().fg(Color::White).bold());
    frame.render_widget(barchart, left_layout[1]);

    // Medium heat (if stigmergy enabled)
    if config.medium_size > 0 {
        let medium_text = format!(
            "  Total heat: {}  |  Max channel: {}  |  Channels: {}",
            stats.medium_heat, stats.medium_max, config.medium_size,
        );
        let medium_widget = Paragraph::new(medium_text)
            .block(Block::default().borders(Borders::ALL).title(" Stigmergy Medium "));
        frame.render_widget(medium_widget, left_layout[2]);
    }

    // Right panel: sparklines
    let right_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(50),
            Constraint::Percentage(50),
        ])
        .split(body_layout[1]);

    // Population sparkline
    let pop_sparkline = Sparkline::default()
        .block(Block::default().borders(Borders::ALL).title(format!(
            " Population (current: {}) ", stats.population
        )))
        .data(&stats.pop_history)
        .max(config.max_organisms as u64)
        .style(Style::default().fg(Color::Cyan));
    frame.render_widget(pop_sparkline, right_layout[0]);

    // Energy sparkline
    let energy_sparkline = Sparkline::default()
        .block(Block::default().borders(Borders::ALL).title(format!(
            " Avg Energy (current: {:.0}) ", stats.avg_energy
        )))
        .data(&stats.energy_history)
        .max(config.e_max as u64)
        .style(Style::default().fg(Color::Yellow));
    frame.render_widget(energy_sparkline, right_layout[1]);

    // Footer
    let footer = Paragraph::new(" Press 'q' to quit ")
        .style(Style::default().fg(Color::DarkGray));
    frame.render_widget(footer, main_layout[2]);
}

// ============================================================================
// Cell TUI mode (interactive with pause/step/inspect/speed control)
// ============================================================================

use crate::cell_vm::{CellConfig, CellWorld, CellOrganism, CellType, cell_seed_a, cell_seed_b};

/// Run the Cell v3 TUI with interactive controls.
pub fn run_cell_tui(config: CellConfig) -> io::Result<()> {
    enable_raw_mode()?;
    io::stdout().execute(EnterAlternateScreen)?;
    let mut terminal = Terminal::new(CrosstermBackend::new(io::stdout()))?;

    let mut world = CellWorld::new(config.clone(), 42);
    for _ in 0..10 { world.add_organism(cell_seed_a(&config)); }
    for _ in 0..10 { world.add_organism(cell_seed_b(&config)); }

    let mut pop_history: Vec<u64> = Vec::new();
    let mut energy_history: Vec<u64> = Vec::new();
    let mut paused = false;
    let mut ticks_per_frame: u64 = 50;
    let mut show_help = false;
    let mut inspect_idx: Option<usize> = None; // organism index to inspect
    let frame_duration = Duration::from_millis(100); // 10 FPS

    loop {
        let frame_start = Instant::now();

        // Handle input
        if event::poll(Duration::from_millis(0))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    match key.code {
                        KeyCode::Char('q') => break,
                        KeyCode::Char('p') => { paused = !paused; inspect_idx = None; },
                        KeyCode::Char('h') => show_help = !show_help,
                        KeyCode::Char('s') if paused => { world.tick(); }, // single step
                        KeyCode::Char('+') | KeyCode::Char('=') => {
                            ticks_per_frame = (ticks_per_frame * 2).min(1000);
                        },
                        KeyCode::Char('-') => {
                            ticks_per_frame = (ticks_per_frame / 2).max(1);
                        },
                        KeyCode::Char('i') => {
                            // Toggle inspect: pick first alive organism
                            if inspect_idx.is_some() {
                                inspect_idx = None;
                            } else if let Some((i, _)) = world.organisms.iter().enumerate()
                                .find(|(_, o)| o.alive)
                            {
                                inspect_idx = Some(i);
                                paused = true;
                            }
                        },
                        KeyCode::Right if inspect_idx.is_some() => {
                            // Next organism
                            let cur = inspect_idx.unwrap();
                            inspect_idx = world.organisms.iter().enumerate()
                                .skip(cur + 1)
                                .find(|(_, o)| o.alive)
                                .map(|(i, _)| i)
                                .or_else(|| world.organisms.iter().enumerate()
                                    .find(|(_, o)| o.alive)
                                    .map(|(i, _)| i));
                        },
                        KeyCode::Left if inspect_idx.is_some() => {
                            // Previous organism
                            let cur = inspect_idx.unwrap();
                            inspect_idx = world.organisms.iter().enumerate()
                                .rev()
                                .skip(world.organisms.len().saturating_sub(cur))
                                .find(|(_, o)| o.alive)
                                .map(|(i, _)| i)
                                .or_else(|| world.organisms.iter().enumerate()
                                    .rev()
                                    .find(|(_, o)| o.alive)
                                    .map(|(i, _)| i));
                        },
                        _ => {}
                    }
                }
            }
        }

        // Run ticks (unless paused)
        if !paused {
            for _ in 0..ticks_per_frame {
                if world.tick >= config.total_ticks { break; }
                world.tick();
            }
        }

        // Collect stats
        let alive: Vec<&CellOrganism> = world.organisms.iter().filter(|o| o.alive).collect();
        let n = alive.len();
        pop_history.push(n as u64);
        let avg_e = if n > 0 {
            alive.iter().map(|o| o.total_energy() as u64).sum::<u64>() / n.max(1) as u64
        } else { 0 };
        energy_history.push(avg_e);
        if pop_history.len() > 200 {
            pop_history.drain(0..pop_history.len() - 200);
            energy_history.drain(0..energy_history.len() - 200);
        }

        let (mut cc, mut ec, mut sc, mut dc) = (0u64, 0u64, 0u64, 0u64);
        let mut fresh_sum: f64 = 0.0;
        for org in &alive {
            for c in &org.cells {
                match c.content {
                    CellType::Code(_) => cc += 1,
                    CellType::Energy(_) => ec += 1,
                    CellType::Stomach(_) => sc += 1,
                    CellType::Data(_) => dc += 1,
                }
            }
            fresh_sum += org.min_freshness() as f64;
        }
        let avg_fresh = if n > 0 { fresh_sum / n as f64 } else { 0.0 };
        let avg_energy_f = if n > 0 { alive.iter().map(|o| o.total_energy() as f64).sum::<f64>() / n as f64 } else { 0.0 };
        let max_gen = alive.iter().map(|o| o.generation).max().unwrap_or(0);
        let tick = world.tick;
        let food = world.food_pool;
        let cem = config.cell_energy_max;
        let decay = config.freshness_decay;
        let total_ticks = config.total_ticks;
        let max_org = config.max_organisms;
        let spd = ticks_per_frame;

        // Capture inspect data before draw closure
        let inspect_data: Option<Vec<String>> = inspect_idx.and_then(|i| {
            world.organisms.get(i).filter(|o| o.alive).map(|org| {
                let mut lines = vec![
                    format!(" Organism #{} | Gen {} | Age {} | IP {}", i, org.generation, org.age, org.ip),
                    format!(" Energy: {} | Code cells: {} | Total cells: {}", org.total_energy(), org.code_count(), org.cells.len()),
                    String::new(),
                ];
                for (ci, cell) in org.cells.iter().enumerate() {
                    let marker = if cell.is_code() {
                        let mut code_idx = 0;
                        for c in &org.cells[..ci] { if c.is_code() { code_idx += 1; } }
                        if code_idx == org.ip { " <IP" } else { "" }
                    } else { "" };
                    lines.push(format!(" [{:2}] {} {}", ci, cell, marker));
                }
                lines
            })
        });

        terminal.draw(|frame| {
            let area = frame.area();

            // Help overlay
            if show_help {
                let help_text = vec![
                    Line::from("  p   Pause / Resume"),
                    Line::from("  q   Quit"),
                    Line::from("  h   Toggle this help"),
                    Line::from("  s   Single step (paused)"),
                    Line::from("  i   Inspect organism"),
                    Line::from("  < > Navigate organisms"),
                    Line::from("  +/- Speed up / down"),
                ];
                let help = Paragraph::new(help_text)
                    .block(Block::default().borders(Borders::ALL).title(" Help "))
                    .style(Style::default().fg(Color::White));
                frame.render_widget(help, area);
                return;
            }

            // Inspect overlay
            if let Some(ref lines) = inspect_data {
                let text: Vec<Line> = lines.iter().map(|s| Line::from(s.as_str())).collect();
                let inspect_w = Paragraph::new(text)
                    .block(Block::default().borders(Borders::ALL)
                        .title(" Organism Inspector (i=close, <>=nav) "))
                    .style(Style::default().fg(Color::Yellow));
                frame.render_widget(inspect_w, area);
                return;
            }

            let main_layout = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Length(3), Constraint::Min(0), Constraint::Length(1)])
                .split(area);

            let status = if paused { "PAUSED" } else { "RUNNING" };
            let header = Paragraph::new(format!(
                " Cell v3 | {} | Tick {}/{} ({:.0}%) | Pop {} | Gen {} | x{} ",
                status, tick, total_ticks, (tick as f64 / total_ticks as f64) * 100.0,
                n, max_gen, spd,
            ))
            .block(Block::default().borders(Borders::ALL).title(" ExoMind Cell "))
            .style(Style::default().fg(if paused { Color::Yellow } else { Color::Cyan }).bold());
            frame.render_widget(header, main_layout[0]);

            let body = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(45), Constraint::Percentage(55)])
                .split(main_layout[1]);

            let left = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Length(8), Constraint::Length(5), Constraint::Min(0)])
                .split(body[0]);

            let stats_text = vec![
                Line::from(vec![
                    Span::raw("  Energy:  "),
                    Span::styled(format!("{:.0}", avg_energy_f), Style::default().fg(Color::Yellow)),
                    Span::raw(format!("  CEM={}", cem)),
                ]),
                Line::from(vec![
                    Span::raw("  Fresh:   "),
                    Span::styled(format!("{:.0}", avg_fresh), Style::default().fg(
                        if avg_fresh > 200.0 { Color::Green }
                        else if avg_fresh > 100.0 { Color::Yellow }
                        else { Color::Red }
                    )),
                ]),
                Line::from(format!("  Food:    {}", food)),
                Line::from(format!("  Cells:   {}C {}E {}S {}D", cc, ec, sc, dc)),
                Line::from(format!("  Decay:   {}", if decay { "ON" } else { "OFF" })),
            ];
            frame.render_widget(
                Paragraph::new(stats_text)
                    .block(Block::default().borders(Borders::ALL).title(" Stats ")),
                left[0],
            );

            let bar_data: Vec<(&str, u64)> = vec![("Code", cc), ("Enrg", ec), ("Stom", sc), ("Data", dc)];
            frame.render_widget(
                BarChart::default()
                    .block(Block::default().borders(Borders::ALL).title(" Cell Types "))
                    .data(&bar_data).bar_width(5).bar_gap(1)
                    .bar_style(Style::default().fg(Color::Green))
                    .value_style(Style::default().fg(Color::White).bold()),
                left[1],
            );

            let right = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
                .split(body[1]);

            frame.render_widget(
                Sparkline::default()
                    .block(Block::default().borders(Borders::ALL).title(format!(" Pop ({}) ", n)))
                    .data(&pop_history).max(max_org as u64)
                    .style(Style::default().fg(Color::Cyan)),
                right[0],
            );
            frame.render_widget(
                Sparkline::default()
                    .block(Block::default().borders(Borders::ALL).title(format!(" Energy ({:.0}) ", avg_energy_f)))
                    .data(&energy_history).max((cem as u64) * 5)
                    .style(Style::default().fg(Color::Yellow)),
                right[1],
            );

            frame.render_widget(
                Paragraph::new(" q:quit  p:pause  h:help  i:inspect  s:step  +/-:speed ")
                    .style(Style::default().fg(Color::DarkGray)),
                main_layout[2],
            );
        })?;

        if world.tick >= config.total_ticks && !paused {
            paused = true; // Auto-pause at end
        }

        let elapsed = frame_start.elapsed();
        if elapsed < frame_duration { std::thread::sleep(frame_duration - elapsed); }
    }

    disable_raw_mode()?;
    io::stdout().execute(LeaveAlternateScreen)?;
    Ok(())
}
