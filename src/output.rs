use crate::api::{DaySummary, UserSummaryData};
use crate::i18n::Locale;
use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::Span;
use ratatui::widgets::{Block, BorderType, Row, Table, Widget};
use unicode_width::UnicodeWidthStr;

#[derive(Clone, Copy, PartialEq)]
pub enum RenderMode {
    Ascii,
    Unicode,
}

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
            Self::Unicode => BorderType::Rounded,
        }
    }

    fn color(&self, c: Color) -> Color {
        match self {
            Self::Ascii => Color::Reset,
            Self::Unicode => c,
        }
    }
}

pub fn print_summary(
    summary: &UserSummaryData,
    requests: u64,
    json: bool,
    locale: Locale,
    render_mode: RenderMode,
) {
    if json {
        output_json_summary(summary, requests);
        return;
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

    let bal_val = format!("{:.2} {}", balance.0.parse::<f64>().unwrap_or(0.0), balance.1);
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
    let value_w = [&bal_val, &cost_val, &req_val, &tok_val]
        .iter()
        .map(|v| UnicodeWidthStr::width(v.as_str()))
        .max()
        .unwrap_or(10);
    let title_w =
        UnicodeWidthStr::width(format!(" {} ", locale.t("header")).as_str());
    let content_w = label_w + value_w + 10;
    let card_w = content_w.max(title_w + 8);

    let bold = Style::new().add_modifier(Modifier::BOLD);
    let bal_style = bold.fg(render_mode.color(Color::Green));
    let cost_style = bold.fg(render_mode.color(Color::Yellow));
    let req_style = bold.fg(render_mode.color(Color::Cyan));
    let tok_style = bold.fg(render_mode.color(Color::White));

    let rows = [
        Row::new([
            Span::raw(labels[0].clone()),
            Span::styled(bal_val, bal_style),
        ]),
        Row::new([
            Span::raw(labels[1].clone()),
            Span::styled(cost_val, cost_style),
        ]),
        Row::new([
            Span::raw(labels[2].clone()),
            Span::styled(req_val, req_style),
        ]),
        Row::new([
            Span::raw(labels[3].clone()),

            Span::styled(tok_val, tok_style),
        ]),
    ];

    let block = Block::bordered()
        .border_type(render_mode.border_type())
        .title(format!(" {} ", locale.t("header")))
        .title_style(bold)
        .border_style(bold.fg(render_mode.color(Color::Cyan)));

    let table = Table::new(
        rows,
        [
            Constraint::Length(label_w as u16),
            Constraint::Length(value_w as u16),
        ],
    )
    .block(block)
    .column_spacing(2);

    render_inline(table, 6, card_w);
}

pub fn print_usage(
    days: &[DaySummary],
    model_filter: Option<&str>,
    json: bool,
    locale: Locale,
    render_mode: RenderMode,
) {
    let filtered: Vec<&DaySummary> = if let Some(model) = model_filter {
        days.iter()
            .filter(|d| model_matches(&d.model, model))
            .collect()
    } else {
        days.iter().collect()
    };

    if filtered.is_empty() {
        println!("{}", locale.t("no_data"));
        return;
    }

    if json {
        output_json_usage(&filtered);
        return;
    }

    let bold = Style::new().add_modifier(Modifier::BOLD);
    let header_style = bold.bg(render_mode.color(Color::Rgb(40, 40, 40)));
    let total_style = bold.fg(render_mode.color(Color::Yellow));

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
            .map(|h| Span::styled(h.clone(), header_style)),
    );

    let mut total_prompt: u64 = 0;
    let mut total_cache_hit: u64 = 0;
    let mut total_cache_miss: u64 = 0;
    let mut total_response: u64 = 0;
    let mut total_requests: u64 = 0;
    let mut total_cost: f64 = 0.0;

    let data_rows: Vec<Row> = filtered
        .iter()
        .map(|d| {
            total_prompt += d.prompt_tokens;
            total_cache_hit += d.cache_hit_tokens;
            total_cache_miss += d.cache_miss_tokens;
            total_response += d.response_tokens;
            total_requests += d.requests;
            total_cost += d.cost;

            Row::new(vec![
                d.date.clone(),
                format_num(d.prompt_tokens),
                format_num(d.cache_hit_tokens),
                format_num(d.cache_miss_tokens),
                format_num(d.response_tokens),
                format_num(d.requests),
                format!("{:.2}", d.cost),
            ])
        })
        .collect();

    let total_row = Row::new(vec![
        Span::styled(locale.t("total"), total_style),
        Span::styled(format_num(total_prompt), total_style),
        Span::styled(format_num(total_cache_hit), total_style),
        Span::styled(format_num(total_cache_miss), total_style),
        Span::styled(format_num(total_response), total_style),
        Span::styled(format_num(total_requests), total_style),
        Span::styled(format!("{:.2}", total_cost), total_style),
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
            Constraint::Min(10),
            Constraint::Min(8),
            Constraint::Min(10),
            Constraint::Min(10),
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
            .title_style(bold)
            .border_style(bold.fg(render_mode.color(Color::Cyan))),
    )
    .column_spacing(1);

    render_inline(table, filtered.len() + 4, 72);
}

fn render_inline(widget: impl Widget, height: usize, width: usize) {
    let area = Rect::new(0, 0, width as u16, height as u16);
    let mut buffer = Buffer::empty(area);
    widget.render(area, &mut buffer);

    let mut lines: Vec<String> = Vec::with_capacity(height);
    for y in 0..height {
        let mut line = String::with_capacity(width + 4);
        let mut skip = 0usize;
        for x in 0..width {
            if skip > 0 {
                skip -= 1;
                continue;
            }
            if let Some(cell) = buffer.cell((x as u16, y as u16)) {
                let sym = cell.symbol();
                let sym_width = UnicodeWidthStr::width(sym);
                if sym_width > 1 {
                    skip = sym_width - 1;
                }
                line.push_str(sym);
            }
        }
        let trimmed = line.trim_end().to_string();
        lines.push(trimmed);
    }
    while lines.last().map_or(false, |l| l.is_empty()) {
        lines.pop();
    }
    for line in &lines {
        println!("{}", line);
    }
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
    a == f || a.contains(&f) || f.contains(&a)
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
