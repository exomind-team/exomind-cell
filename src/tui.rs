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
    let ticks_per_frame = 50; // Run 50 ticks per frame for smooth animation
    let target_fps = 30;
    let frame_duration = Duration::from_millis(1000 / target_fps);

    loop {
        let frame_start = Instant::now();

        // Check for quit
        if event::poll(Duration::from_millis(0))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press && key.code == KeyCode::Char('q') {
                    break;
                }
            }
        }

        // Run simulation ticks
        for _ in 0..ticks_per_frame {
            if world.tick >= config.total_ticks {
                break;
            }
            world.tick();
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
