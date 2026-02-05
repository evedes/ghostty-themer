pub mod widgets;

use std::io::{self, stdout};
use std::path::PathBuf;
use std::time::Duration;

use anyhow::Result;
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use crossterm::ExecutableCommand;
use ratatui::prelude::*;
use ratatui::widgets::{Block, Clear, Paragraph};

use crate::cli::ThemeMode;
use crate::pipeline::assign::AnsiPalette;
use crate::pipeline::extract::ExtractedColor;

use self::widgets::PaletteWidget;

/// State for the interactive TUI application.
pub struct TuiApp {
    pub palette: AnsiPalette,
    pub extracted_colors: Vec<ExtractedColor>,
    pub image_path: PathBuf,
    pub mode: ThemeMode,
    pub selected_slot: Option<usize>,
    pub theme_name: String,
    pub show_help: bool,
}

impl TuiApp {
    pub fn new(
        palette: AnsiPalette,
        extracted_colors: Vec<ExtractedColor>,
        image_path: PathBuf,
        mode: ThemeMode,
        theme_name: String,
    ) -> Self {
        Self {
            palette,
            extracted_colors,
            image_path,
            mode,
            selected_slot: None,
            theme_name,
            show_help: false,
        }
    }
}

/// Launch the TUI application.
pub fn run(mut app: TuiApp) -> Result<()> {
    enable_raw_mode()?;
    stdout().execute(EnterAlternateScreen)?;

    let backend = CrosstermBackend::new(stdout());
    let mut terminal = Terminal::new(backend)?;

    let result = run_event_loop(&mut terminal, &mut app);

    // Always restore terminal, even on error
    disable_raw_mode()?;
    stdout().execute(LeaveAlternateScreen)?;

    result
}

fn run_event_loop(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    app: &mut TuiApp,
) -> Result<()> {
    loop {
        terminal.draw(|f| draw(f, app))?;

        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    match key.code {
                        KeyCode::Char('q') => break,
                        KeyCode::Char('?') => app.show_help = !app.show_help,
                        KeyCode::Tab => cycle_slot(app),
                        KeyCode::BackTab => cycle_slot_reverse(app),
                        KeyCode::Char(c @ '1'..='6') => {
                            app.selected_slot = Some((c as u8 - b'0') as usize);
                        }
                        KeyCode::Esc => {
                            if app.show_help {
                                app.show_help = false;
                            } else {
                                app.selected_slot = None;
                            }
                        }
                        _ => {}
                    }
                }
            }
        }
    }
    Ok(())
}

fn cycle_slot(app: &mut TuiApp) {
    app.selected_slot = Some(match app.selected_slot {
        None | Some(15) => 0,
        Some(n) => n + 1,
    });
}

fn cycle_slot_reverse(app: &mut TuiApp) {
    app.selected_slot = Some(match app.selected_slot {
        None | Some(0) => 15,
        Some(n) => n - 1,
    });
}

fn draw(f: &mut Frame, app: &TuiApp) {
    // Main layout: top section, preview, status bar
    let main_layout = Layout::vertical([
        Constraint::Min(10),
        Constraint::Percentage(40),
        Constraint::Length(1),
    ])
    .split(f.area());

    // Top: image (30%) | palette (70%)
    let top_layout = Layout::horizontal([Constraint::Percentage(30), Constraint::Percentage(70)])
        .split(main_layout[0]);

    draw_image_pane(f, app, top_layout[0]);
    draw_palette_pane(f, app, top_layout[1]);
    draw_preview_pane(f, app, main_layout[1]);
    draw_status_bar(f, main_layout[2]);

    if app.show_help {
        draw_help_overlay(f);
    }
}

fn draw_image_pane(f: &mut Frame, app: &TuiApp, area: Rect) {
    let block = Block::bordered().title("Image");
    let inner = block.inner(area);
    f.render_widget(block, area);

    let mut lines = vec![
        Line::from(""),
        Line::from(format!("  {}", app.image_path.display())),
        Line::from(""),
        Line::from(format!("  Mode: {:?}", app.mode)),
        Line::from(format!("  Theme: {}", app.theme_name)),
        Line::from(format!("  Colors: {}", app.extracted_colors.len())),
        Line::from(""),
    ];

    // Show extracted color swatches
    let mut swatch_spans = vec![Span::raw("  ")];
    for ec in app.extracted_colors.iter().take(12) {
        let c = &ec.color;
        let bg = Color::Rgb(c.r, c.g, c.b);
        swatch_spans.push(Span::styled("  ", Style::default().bg(bg)));
    }
    lines.push(Line::from(swatch_spans));

    f.render_widget(Paragraph::new(lines), inner);
}

fn draw_palette_pane(f: &mut Frame, app: &TuiApp, area: Rect) {
    let widget = PaletteWidget::new(&app.palette, app.selected_slot);
    f.render_widget(widget, area);
}

fn draw_preview_pane(f: &mut Frame, app: &TuiApp, area: Rect) {
    let block = Block::bordered().title("Preview");
    let inner = block.inner(area);
    f.render_widget(block, area);

    let bg = &app.palette.background;
    let fg = &app.palette.foreground;
    let base_style = Style::default()
        .bg(Color::Rgb(bg.r, bg.g, bg.b))
        .fg(Color::Rgb(fg.r, fg.g, fg.b));

    let red = &app.palette.slots[1];
    let green = &app.palette.slots[2];
    let yellow = &app.palette.slots[3];
    let blue = &app.palette.slots[4];
    let cyan = &app.palette.slots[6];
    let bright_black = &app.palette.slots[8];

    let lines = vec![
        Line::from(Span::styled(" ".repeat(inner.width as usize), base_style)),
        Line::from(vec![
            Span::styled("  ", base_style),
            Span::styled(
                "user@host",
                base_style.fg(Color::Rgb(green.r, green.g, green.b)),
            ),
            Span::styled(":", base_style),
            Span::styled(
                "~/projects",
                base_style.fg(Color::Rgb(blue.r, blue.g, blue.b)),
            ),
            Span::styled("$ ls", base_style),
            pad_line(inner.width, 28, base_style),
        ]),
        Line::from(vec![
            Span::styled("  ", base_style),
            Span::styled("src/", base_style.fg(Color::Rgb(blue.r, blue.g, blue.b))),
            Span::styled("  ", base_style),
            Span::styled("README.md", base_style.fg(Color::Rgb(fg.r, fg.g, fg.b))),
            Span::styled("  ", base_style),
            Span::styled(
                "Cargo.toml",
                base_style.fg(Color::Rgb(yellow.r, yellow.g, yellow.b)),
            ),
            Span::styled("  ", base_style),
            Span::styled(
                "run.sh",
                base_style.fg(Color::Rgb(green.r, green.g, green.b)),
            ),
            pad_line(inner.width, 39, base_style),
        ]),
        Line::from(vec![
            Span::styled("  ", base_style),
            Span::styled(
                "user@host",
                base_style.fg(Color::Rgb(green.r, green.g, green.b)),
            ),
            Span::styled(":", base_style),
            Span::styled(
                "~/projects",
                base_style.fg(Color::Rgb(blue.r, blue.g, blue.b)),
            ),
            Span::styled("$ git diff", base_style),
            pad_line(inner.width, 34, base_style),
        ]),
        Line::from(vec![
            Span::styled(
                "  - old line removed",
                base_style.fg(Color::Rgb(red.r, red.g, red.b)),
            ),
            pad_line(inner.width, 20, base_style),
        ]),
        Line::from(vec![
            Span::styled(
                "  + new line added",
                base_style.fg(Color::Rgb(green.r, green.g, green.b)),
            ),
            pad_line(inner.width, 18, base_style),
        ]),
        Line::from(vec![
            Span::styled(
                "  # comment in code",
                base_style.fg(Color::Rgb(bright_black.r, bright_black.g, bright_black.b)),
            ),
            pad_line(inner.width, 19, base_style),
        ]),
        Line::from(vec![
            Span::styled("  ", base_style),
            Span::styled("let", base_style.fg(Color::Rgb(cyan.r, cyan.g, cyan.b))),
            Span::styled(" x = ", base_style),
            Span::styled(
                "42",
                base_style.fg(Color::Rgb(yellow.r, yellow.g, yellow.b)),
            ),
            Span::styled(";", base_style),
            pad_line(inner.width, 14, base_style),
        ]),
    ];

    f.render_widget(Paragraph::new(lines), inner);
}

/// Create padding to fill the rest of a line with the base style.
fn pad_line(total_width: u16, used: u16, style: Style) -> Span<'static> {
    let remaining = total_width.saturating_sub(used) as usize;
    Span::styled(" ".repeat(remaining), style)
}

fn draw_status_bar(f: &mut Frame, area: Rect) {
    let status = " q: Quit | ?: Help | Tab/Shift+Tab: Cycle | 1-6: Select accent | Esc: Deselect";
    let bar = Paragraph::new(status).style(
        Style::default()
            .fg(Color::DarkGray)
            .bg(Color::Rgb(20, 20, 20)),
    );
    f.render_widget(bar, area);
}

fn draw_help_overlay(f: &mut Frame) {
    let area = centered_rect(50, 50, f.area());
    let lines = vec![
        Line::from(""),
        Line::from("  Keybindings:"),
        Line::from(""),
        Line::from("  q             Quit"),
        Line::from("  ?             Toggle this help"),
        Line::from("  Tab           Next slot"),
        Line::from("  Shift+Tab     Previous slot"),
        Line::from("  1-6           Select accent slot"),
        Line::from("  Esc           Deselect / close help"),
        Line::from(""),
        Line::from("  Press ? or Esc to close"),
    ];
    let popup = Paragraph::new(lines)
        .block(Block::bordered().title(" Help "))
        .style(Style::default().bg(Color::Black).fg(Color::White));
    f.render_widget(Clear, area);
    f.render_widget(popup, area);
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let v = Layout::vertical([
        Constraint::Percentage((100 - percent_y) / 2),
        Constraint::Percentage(percent_y),
        Constraint::Percentage((100 - percent_y) / 2),
    ])
    .split(r);
    Layout::horizontal([
        Constraint::Percentage((100 - percent_x) / 2),
        Constraint::Percentage(percent_x),
        Constraint::Percentage((100 - percent_x) / 2),
    ])
    .split(v[1])[1]
}
