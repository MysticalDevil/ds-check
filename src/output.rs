use crate::api::{DaySummary, UserSummaryData};
use crate::i18n::Locale;
use crossterm::{queue, style::*};
use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::Span;
use ratatui::widgets::{Block, BorderType, Cell, Padding, Row, Table, Widget};
use std::io::{Write, stdout};
use unicode_width::UnicodeWidthStr;

#[derive(Clone, Copy, PartialEq)]
pub enum RenderMode {
    Ascii,
    Unicode,
}

// ── Color palette ──────────────────────────────────────────────

const C_BORDER: Color = Color::Cyan;
const C_TITLE: Color = Color::Yellow;
const C_LABEL: Color = Color::Gray;
const C_DIM: Color = Color::DarkGray;
const C_BALANCE: Color = Color::Green;
const C_COST: Color = Color::Yellow;
const C_REQUESTS: Color = Color::Cyan;
const C_TOKENS: Color = Color::White;
const C_WHITE: Color = Color::White;

// Rgb colors only where builtin palette is insufficient
const C_HEADER_BG: Color = Color::Rgb(50, 50, 55);
const C_ROW_EVEN_BG: Color = Color::Rgb(26, 26, 32);
const C_COST_DIM: Color = Color::Rgb(0xCC, 0xAA, 0x00);

impl RenderMode {
    pub fn from_env() -> Self {
        std::env::var("DSCHECK_RENDER")
            .map(|v| v.to_lowercase())
            .map(|v| match v.as_str() {
                "ascii" => Self::Ascii,
                _ => Self::Unicode,
            })
            .unwrap_or(Self::Unicode)
    }

    fn border_type(&self) -> BorderType {
        match self {
            Self::Ascii => BorderType::Plain,
            Self::Unicode => BorderType::Thick,
        }
    }

    fn color(&self, c: Color) -> Color {
        match self {
            Self::Ascii => Color::Reset,
            Self::Unicode => c,
        }
    }
}

// ── Crossterm color bridge ─────────────────────────────────────

fn rt_to_ct(c: Color) -> crossterm::style::Color {
    match c {
        Color::Reset => crossterm::style::Color::Reset,
        Color::Black => crossterm::style::Color::Black,
        Color::Red => crossterm::style::Color::DarkRed,
        Color::Green => crossterm::style::Color::DarkGreen,
        Color::Yellow => crossterm::style::Color::DarkYellow,
        Color::Blue => crossterm::style::Color::DarkBlue,
        Color::Magenta => crossterm::style::Color::DarkMagenta,
        Color::Cyan => crossterm::style::Color::DarkCyan,
        Color::Gray => crossterm::style::Color::Grey,
        Color::DarkGray => crossterm::style::Color::DarkGrey,
        Color::LightRed => crossterm::style::Color::Red,
        Color::LightGreen => crossterm::style::Color::Green,
        Color::LightYellow => crossterm::style::Color::Yellow,
        Color::LightBlue => crossterm::style::Color::Blue,
        Color::LightMagenta => crossterm::style::Color::Magenta,
        Color::LightCyan => crossterm::style::Color::Cyan,
        Color::White => crossterm::style::Color::White,
        Color::Indexed(i) => crossterm::style::Color::AnsiValue(i),
        Color::Rgb(r, g, b) => crossterm::style::Color::Rgb { r, g, b },
    }
}

fn apply_style(stdout: &mut std::io::Stdout, style: Style) -> anyhow::Result<()> {
    queue!(stdout, ResetColor)?;
    if let Some(fg) = style.fg {
        queue!(stdout, SetForegroundColor(rt_to_ct(fg)))?;
    }
    if let Some(bg) = style.bg {
        queue!(stdout, SetBackgroundColor(rt_to_ct(bg)))?;
    }
    if style.add_modifier.contains(Modifier::BOLD) {
        queue!(stdout, SetAttribute(Attribute::Bold))?;
    }
    if style.add_modifier.contains(Modifier::DIM) {
        queue!(stdout, SetAttribute(Attribute::Dim))?;
    }
    if style.add_modifier.contains(Modifier::ITALIC) {
        queue!(stdout, SetAttribute(Attribute::Italic))?;
    }
    if style.add_modifier.contains(Modifier::UNDERLINED) {
        queue!(stdout, SetAttribute(Attribute::Underlined))?;
    }
    if style.add_modifier.contains(Modifier::REVERSED) {
        queue!(stdout, SetAttribute(Attribute::Reverse))?;
    }
    Ok(())
}

// ── Summary card ───────────────────────────────────────────────

pub fn print_summary(
    summary: &UserSummaryData,
    requests: u64,
    json: bool,
    locale: Locale,
    render_mode: RenderMode,
) -> anyhow::Result<()> {
    if json {
        output_json_summary(summary, requests);
        return Ok(());
    }

    let balance = summary
        .normal_wallets
        .first()
        .map(|w| (w.balance.clone(), w.currency.clone()))
        .unwrap_or(("0".into(), "CNY".into()));
    let cost = summary
        .monthly_costs
        .first()
        .map(|c| (c.amount.clone(), c.currency.clone()))
        .unwrap_or(("0".into(), "CNY".into()));
    let tokens = &summary.monthly_token_usage;

    let bal_val = format!(
        "{:.2} {}",
        balance.0.parse::<f64>().unwrap_or(0.0),
        balance.1
    );
    let cost_val = format!("{:.2} {}", cost.0.parse::<f64>().unwrap_or(0.0), cost.1);
    let req_val = format_num(requests);
    let tok_val = format_num(tokens.parse().unwrap_or(0));

    let labels = [
        locale.t("balance"),
        locale.t("monthly_cost"),
        locale.t("api_requests"),
        locale.t("tokens"),
    ];
    let label_w = labels
        .iter()
        .map(|l| UnicodeWidthStr::width(l.as_str()))
        .max()
        .unwrap_or(10);

    let bold = Style::new().add_modifier(Modifier::BOLD);
    let gold = render_mode.color(C_TITLE);
    let cyan = render_mode.color(C_BORDER);
    let label_style = Style::new().fg(render_mode.color(C_LABEL));

    let value_styles = [
        Style::new()
            .fg(render_mode.color(C_BALANCE))
            .add_modifier(Modifier::BOLD),
        Style::new()
            .fg(render_mode.color(C_COST))
            .add_modifier(Modifier::BOLD),
        Style::new()
            .fg(render_mode.color(C_REQUESTS))
            .add_modifier(Modifier::BOLD),
        Style::new()
            .fg(render_mode.color(C_TOKENS))
            .add_modifier(Modifier::BOLD),
    ];
    let values = [bal_val, cost_val, req_val, tok_val];
    let value_w = values
        .iter()
        .map(|v| UnicodeWidthStr::width(v.as_str()))
        .max()
        .unwrap_or(10);

    if render_mode == RenderMode::Ascii {
        let title = locale.t("header");
        let title_w = UnicodeWidthStr::width(title.as_str());
        println!("{}", title);
        println!("{}", "=".repeat(title_w));
        for (label, value) in labels.iter().zip(values.iter()) {
            println!("{:>w$}: {}", label, value, w = label_w);
        }
        return Ok(());
    }

    let title_w = UnicodeWidthStr::width(format!(" {} ", locale.t("header")).as_str());
    let card_w = (label_w + value_w + 10).max(title_w + 4);

    let rows: Vec<Row> = labels
        .iter()
        .zip(values.iter())
        .zip(value_styles.iter())
        .map(|((label, value), vstyle)| {
            Row::new([
                Span::styled(format!("{}  ", label), label_style),
                Span::styled(value.clone(), *vstyle),
            ])
        })
        .collect();

    let block = Block::bordered()
        .border_type(render_mode.border_type())
        .title(format!(" {} ", locale.t("header")))
        .title_style(bold.fg(gold))
        .border_style(Style::new().fg(cyan))
        .padding(Padding::symmetric(3, 1));

    let table = Table::new(
        rows,
        [
            Constraint::Length(label_w as u16 + 2),
            Constraint::Length(value_w as u16),
        ],
    )
    .block(block)
    .column_spacing(0);

    render_inline(table, 8, card_w)?;
    Ok(())
}

// ── Usage card ─────────────────────────────────────────────────

pub fn print_usage(
    days: &[DaySummary],
    model_filter: Option<&str>,
    json: bool,
    locale: Locale,
    render_mode: RenderMode,
) -> anyhow::Result<()> {
    let filtered: Vec<&DaySummary> = if let Some(model) = model_filter {
        days.iter()
            .filter(|d| model_matches(&d.model, model))
            .collect()
    } else {
        days.iter().collect()
    };

    if filtered.is_empty() {
        println!("{}", locale.t("no_data"));
        return Ok(());
    }

    if json {
        output_json_usage(&filtered);
        return Ok(());
    }

    if render_mode == RenderMode::Ascii {
        let headers = [
            locale.t("date"),
            locale.t("prompt_tokens"),
            locale.t("cache_hit_tokens"),
            locale.t("cache_miss_tokens"),
            locale.t("response_tokens"),
            locale.t("requests"),
            locale.t("cost"),
        ];
        let mut widths: Vec<usize> = headers.iter().map(|h| h.len()).collect();

        let mut total_prompt = 0u64;
        let mut total_cache_hit = 0u64;
        let mut total_cache_miss = 0u64;
        let mut total_response = 0u64;
        let mut total_requests = 0u64;
        let mut total_cost = 0.0f64;

        let rows: Vec<Vec<String>> = filtered
            .iter()
            .map(|d| {
                total_prompt += d.prompt_tokens;
                total_cache_hit += d.cache_hit_tokens;
                total_cache_miss += d.cache_miss_tokens;
                total_response += d.response_tokens;
                total_requests += d.requests;
                total_cost += d.cost;
                vec![
                    d.date.clone(),
                    format_num(d.prompt_tokens),
                    format_num(d.cache_hit_tokens),
                    format_num(d.cache_miss_tokens),
                    format_num(d.response_tokens),
                    format_num(d.requests),
                    format!("{:.2}", d.cost),
                ]
            })
            .collect();

        for row in &rows {
            for (i, cell) in row.iter().enumerate() {
                widths[i] = widths[i].max(cell.len());
            }
        }

        let total_row = [
            locale.t("total"),
            format_num(total_prompt),
            format_num(total_cache_hit),
            format_num(total_cache_miss),
            format_num(total_response),
            format_num(total_requests),
            format!("{:.2}", total_cost),
        ];
        for (i, cell) in total_row.iter().enumerate() {
            widths[i] = widths[i].max(cell.len());
        }

        // model name header
        if let Some(d) = filtered.first() {
            println!("{}", d.model);
        }

        for (i, h) in headers.iter().enumerate() {
            if i > 0 {
                print!(" | ");
            }
            print!("{:>w$}", h, w = widths[i]);
        }
        println!();
        for (i, w) in widths.iter().enumerate() {
            if i > 0 {
                print!("-+-");
            }
            print!("{:-<width$}", "", width = w);
        }
        println!();
        for row in &rows {
            for (i, cell) in row.iter().enumerate() {
                if i > 0 {
                    print!(" | ");
                }
                print!("{:>w$}", cell, w = widths[i]);
            }
            println!();
        }
        for (i, cell) in total_row.iter().enumerate() {
            if i > 0 {
                print!(" | ");
            }
            print!("{:>w$}", cell, w = widths[i]);
        }
        println!();

        return Ok(());
    }

    let bold = Style::new().add_modifier(Modifier::BOLD);
    let gold = render_mode.color(C_TITLE);
    let cyan = render_mode.color(C_BORDER);
    let dim = Style::new().fg(render_mode.color(C_DIM));

    let header_bg = render_mode.color(C_HEADER_BG);
    let row_even_bg = render_mode.color(C_ROW_EVEN_BG);

    let header_style = bold.fg(render_mode.color(C_WHITE)).bg(header_bg);
    let total_style = bold.fg(gold);
    let row_even = Style::new().bg(row_even_bg);
    let row_odd = Style::new();

    let headers = [
        locale.t("date"),
        locale.t("prompt_tokens"),
        locale.t("cache_hit_tokens"),
        locale.t("cache_miss_tokens"),
        locale.t("response_tokens"),
        locale.t("requests"),
        locale.t("cost"),
    ];
    let header = Row::new(
        headers
            .iter()
            .map(|h| Cell::from(Span::styled(h.clone(), header_style))),
    )
    .style(header_style);

    let mut total_prompt: u64 = 0;
    let mut total_cache_hit: u64 = 0;
    let mut total_cache_miss: u64 = 0;
    let mut total_response: u64 = 0;
    let mut total_requests: u64 = 0;
    let mut total_cost: f64 = 0.0;

    let data_rows: Vec<Row> = filtered
        .iter()
        .enumerate()
        .map(|(i, d)| {
            total_prompt += d.prompt_tokens;
            total_cache_hit += d.cache_hit_tokens;
            total_cache_miss += d.cache_miss_tokens;
            total_response += d.response_tokens;
            total_requests += d.requests;
            total_cost += d.cost;

            let cost_s = format!("{:.2}", d.cost);
            let style = if i % 2 == 0 { row_even } else { row_odd };
            Row::new(vec![
                Cell::from(Span::styled(d.date.clone(), dim)),
                Cell::from(Span::styled(
                    format_num(d.prompt_tokens),
                    Style::new().fg(render_mode.color(C_WHITE)),
                )),
                Cell::from(Span::styled(
                    format_num(d.cache_hit_tokens),
                    Style::new().fg(render_mode.color(C_WHITE)),
                )),
                Cell::from(Span::styled(
                    format_num(d.cache_miss_tokens),
                    Style::new().fg(render_mode.color(C_WHITE)),
                )),
                Cell::from(Span::styled(
                    format_num(d.response_tokens),
                    Style::new().fg(render_mode.color(C_WHITE)),
                )),
                Cell::from(Span::styled(
                    format_num(d.requests),
                    Style::new().fg(render_mode.color(C_WHITE)),
                )),
                Cell::from(Span::styled(
                    cost_s,
                    Style::new().fg(render_mode.color(C_COST_DIM)),
                )),
            ])
            .style(style)
        })
        .collect();

    let total_row = Row::new(vec![
        Cell::from(Span::styled(locale.t("total"), total_style)),
        Cell::from(Span::styled(format_num(total_prompt), total_style)),
        Cell::from(Span::styled(format_num(total_cache_hit), total_style)),
        Cell::from(Span::styled(format_num(total_cache_miss), total_style)),
        Cell::from(Span::styled(format_num(total_response), total_style)),
        Cell::from(Span::styled(format_num(total_requests), total_style)),
        Cell::from(Span::styled(
            format!("{:.2}", total_cost),
            Style::new()
                .fg(render_mode.color(C_TITLE))
                .add_modifier(Modifier::BOLD),
        )),
    ]);

    let title = if let Some(d) = filtered.first() {
        format!(" {} ({}) ", d.model, filtered.len())
    } else {
        locale.t("no_data")
    };

    let table = Table::new(
        data_rows,
        [
            Constraint::Min(10),
            Constraint::Min(11),
            Constraint::Min(8),
            Constraint::Min(8),
            Constraint::Min(11),
            Constraint::Min(8),
            Constraint::Min(6),
        ],
    )
    .header(header)
    .footer(total_row)
    .block(
        Block::bordered()
            .border_type(render_mode.border_type())
            .title(title)
            .title_style(bold.fg(gold))
            .border_style(Style::new().fg(cyan))
            .padding(Padding::symmetric(1, 1)),
    )
    .column_spacing(1);

    render_inline(table, filtered.len() + 6, 86)?;
    Ok(())
}

// ── Inline render via crossterm ────────────────────────────────

fn render_inline(widget: impl Widget, height: usize, width: usize) -> anyhow::Result<()> {
    let area = Rect::new(0, 0, width as u16, height as u16);
    let mut buffer = Buffer::empty(area);
    widget.render(area, &mut buffer);

    let mut lines: Vec<Vec<(Style, String)>> = Vec::with_capacity(height);

    for y in 0..height {
        let mut runs: Vec<(Style, String)> = Vec::new();
        let mut skip = 0usize;
        let mut current_style = Style::default();
        let mut current_text = String::new();

        for x in 0..width {
            if skip > 0 {
                skip -= 1;
                continue;
            }
            if let Some(cell) = buffer.cell((x as u16, y as u16)) {
                let sym = cell.symbol();
                let w = UnicodeWidthStr::width(sym);
                if w > 1 {
                    skip = w - 1;
                }
                let style = cell.style();
                if style == current_style {
                    current_text.push_str(sym);
                } else {
                    if !current_text.is_empty() {
                        runs.push((current_style, current_text));
                    }
                    current_style = style;
                    current_text = sym.to_string();
                }
            }
        }
        if !current_text.is_empty() {
            runs.push((current_style, current_text));
        }
        lines.push(runs);
    }

    while lines.last().is_some_and(|runs| {
        runs.is_empty()
            || runs
                .iter()
                .map(|(_, t)| t.as_str())
                .collect::<String>()
                .trim()
                .is_empty()
    }) {
        lines.pop();
    }

    let mut stdout = stdout();
    for runs in &lines {
        for (style, text) in runs {
            apply_style(&mut stdout, *style)?;
            queue!(stdout, Print(text))?;
        }
        queue!(stdout, ResetColor, Print("\n"))?;
    }
    stdout.flush()?;
    Ok(())
}

fn output_json_summary(summary: &UserSummaryData, requests: u64) {
    let balance = summary
        .normal_wallets
        .first()
        .map(|w| (w.balance.as_str(), w.currency.as_str()))
        .unwrap_or(("0", "CNY"));
    let cost = summary
        .monthly_costs
        .first()
        .map(|c| (c.amount.as_str(), c.currency.as_str()))
        .unwrap_or(("0", "CNY"));

    let output = serde_json::json!({
        "balance": {
            "amount": balance.0.parse::<f64>().unwrap_or(0.0),
            "currency": balance.1,
        },
        "monthly_cost": {
            "amount": cost.0.parse::<f64>().unwrap_or(0.0),
            "currency": cost.1,
        },
        "api_requests": requests,
        "tokens": summary.monthly_token_usage.parse::<u64>().unwrap_or(0),
    });
    if let Ok(s) = serde_json::to_string_pretty(&output) {
        println!("{}", s);
    }
}

fn output_json_usage(days: &[&DaySummary]) {
    let output: Vec<serde_json::Value> = days
        .iter()
        .map(|d| {
            serde_json::json!({
                "date": d.date,
                "model": d.model,
                "prompt_tokens": d.prompt_tokens,
                "cache_hit_tokens": d.cache_hit_tokens,
                "cache_miss_tokens": d.cache_miss_tokens,
                "response_tokens": d.response_tokens,
                "requests": d.requests,
                "cost": d.cost,
            })
        })
        .collect();
    if let Ok(s) = serde_json::to_string_pretty(&output) {
        println!("{}", s);
    }
}

fn model_matches(actual: &str, filter: &str) -> bool {
    let a = actual.to_lowercase();
    let f = filter.to_lowercase();
    a == f || a.contains(&f)
}

fn format_num(n: u64) -> String {
    if n >= 1_000_000_000 {
        format!("{:.2}B", n as f64 / 1_000_000_000.0)
    } else if n >= 1_000_000 {
        format!("{:.2}M", n as f64 / 1_000_000.0)
    } else if n >= 1_000 {
        format!("{:.2}K", n as f64 / 1_000.0)
    } else {
        n.to_string()
    }
}
