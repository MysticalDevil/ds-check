use crate::api::{DaySummary, UserSummaryData};
use crate::i18n::Locale;
use crate::util::dw;
use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::{Block, BorderType, Paragraph, Row, Table, Widget};

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
}

pub fn print_summary(
    summary: &UserSummaryData,
    requests: u64,
    nickname: &str,
    json: bool,
    locale: Locale,
    render_mode: RenderMode,
) {
    if json {
        output_json_summary(summary, requests, nickname);
        return;
    }

    let text = build_summary_text(summary, requests, nickname, &locale);
    let block_title = format!(" {} ", locale.t("header"));
    let p = Paragraph::new(text).block(
        Block::bordered()
            .border_type(render_mode.border_type())
            .title(block_title)
            .title_style(Style::new().add_modifier(Modifier::BOLD)),
    );

    render_widget(p);
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

    let table = build_usage_table(&filtered, &locale, render_mode);
    render_widget(table);
}

fn render_widget(widget: impl Widget) {
    use unicode_width::UnicodeWidthStr;

    let (tw, th) = crossterm::terminal::size().unwrap_or((100, 30));
    let w = (tw as usize).clamp(40, 200);
    let h = (th as usize).clamp(4, 80);
    let area = Rect::new(0, 0, w as u16, h as u16);
    let mut buffer = Buffer::empty(area);
    widget.render(area, &mut buffer);

    let mut last_content_y: usize = 0;
    let mut rows: Vec<String> = Vec::with_capacity(h);

    for y in 0..h {
        let mut line = String::with_capacity(w + 2);
        let mut skip_next = 0usize;
        for x in 0..w {
            if skip_next > 0 {
                skip_next -= 1;
                continue;
            }
            let cell = buffer.cell((x as u16, y as u16)).unwrap();
            let sym = cell.symbol();
            let sym_width = UnicodeWidthStr::width(sym);
            if sym_width > 1 {
                skip_next = sym_width - 1;
            }
            line.push_str(sym);
        }
        let trimmed_line = line.trim_end().to_string();
        let significant = trimmed_line.chars().any(|c| {
            c != ' '
                && c != '│'
                && c != '╰'
                && c != '╯'
                && c != '╭'
                && c != '╮'
                && c != '─'
                && c != '┌'
                && c != '┐'
                && c != '└'
                && c != '┘'
                && c != '├'
                && c != '┤'
                && c != '┬'
                && c != '┴'
                && c != '┼'
                && c != '+'
                && c != '-'
                && c != '|'
        });
        if significant {
            last_content_y = rows.len();
        }
        rows.push(trimmed_line);
    }

    for (i, line) in rows.iter().enumerate() {
        if i > last_content_y {
            break;
        }
        println!("{}", line);
    }
}

fn build_summary_text(
    summary: &UserSummaryData,
    requests: u64,
    nickname: &str,
    locale: &Locale,
) -> Text<'static> {
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

    let tokens = summary.monthly_token_usage.clone();

    let labels = [
        locale.t("user"),
        locale.t("balance"),
        locale.t("monthly_cost"),
        locale.t("api_requests"),
        locale.t("tokens"),
    ];
    let max_w = labels.iter().map(|l| dw(l)).max().unwrap_or(10);

    let bal_str = format!(
        "{:.2} {}",
        balance.0.parse::<f64>().unwrap_or(0.0),
        balance.1
    );
    let cost_str = format!("{:.2} {}", cost.0.parse::<f64>().unwrap_or(0.0), cost.1);
    let req_str = format_num(requests);
    let tok_str = format_num(tokens.parse().unwrap_or(0));

    let lines: Vec<Line<'static>> = vec![
        kv_line(&labels[0], nickname, max_w),
        kv_line(&labels[1], &bal_str, max_w),
        kv_line(&labels[2], &cost_str, max_w),
        kv_line(&labels[3], &req_str, max_w),
        kv_line(&labels[4], &tok_str, max_w),
    ];

    Text::from(lines)
}

fn kv_line(label: &str, value: &str, max_w: usize) -> Line<'static> {
    let padded = pad_end(label, max_w + 2);
    Line::from(vec![Span::raw(format!(" {}: {}", padded, value))])
}

fn build_usage_table(
    days: &[&DaySummary],
    locale: &Locale,
    render_mode: RenderMode,
) -> Table<'static> {
    let headers: Vec<String> = vec![
        locale.t("date"),
        locale.t("prompt_tokens"),
        locale.t("cache_hit_tokens"),
        locale.t("cache_miss_tokens"),
        locale.t("response_tokens"),
        locale.t("requests"),
        locale.t("cost"),
    ];

    let header_row = Row::new(
        headers
            .iter()
            .map(|h| Span::styled(h.clone(), Style::new().add_modifier(Modifier::BOLD))),
    );

    let mut total_prompt: u64 = 0;
    let mut total_cache_hit: u64 = 0;
    let mut total_cache_miss: u64 = 0;
    let mut total_response: u64 = 0;
    let mut total_requests: u64 = 0;
    let mut total_cost: f64 = 0.0;

    let rows: Vec<Row<'static>> = days
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

    let total_label = locale.t("total");
    let total_row = Row::new(vec![
        total_label,
        format_num(total_prompt),
        format_num(total_cache_hit),
        format_num(total_cache_miss),
        format_num(total_response),
        format_num(total_requests),
        format!("{:.2}", total_cost),
    ])
    .style(Style::new().add_modifier(Modifier::BOLD));

    let all_rows: Vec<Row<'static>> = {
        let mut r = vec![header_row];
        r.extend(rows);
        r.push(total_row);
        r
    };

    let title = if let Some(d) = days.first() {
        format!(" {} ({}) ", d.model, days.len())
    } else {
        format!(" {} ", locale.t("no_data"))
    };

    Table::new(
        all_rows,
        [
            Constraint::Min(12),
            Constraint::Min(10),
            Constraint::Min(10),
            Constraint::Min(10),
            Constraint::Min(10),
            Constraint::Min(8),
            Constraint::Min(8),
        ],
    )
    .block(
        Block::bordered()
            .border_type(render_mode.border_type())
            .title(title)
            .title_style(Style::new().add_modifier(Modifier::BOLD)),
    )
    .column_spacing(1)
}

fn output_json_summary(summary: &UserSummaryData, requests: u64, nickname: &str) {
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
    let tokens = &summary.monthly_token_usage;

    let output = serde_json::json!({
        "user": nickname,
        "balance": {
            "amount": balance.0.parse::<f64>().unwrap_or(0.0),
            "currency": balance.1,
        },
        "monthly_cost": {
            "amount": cost.0.parse::<f64>().unwrap_or(0.0),
            "currency": cost.1,
        },
        "api_requests": requests,
        "tokens": tokens.parse::<u64>().unwrap_or(0),
    });
    println!("{}", serde_json::to_string_pretty(&output).unwrap());
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
    println!("{}", serde_json::to_string_pretty(&output).unwrap());
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

fn pad_end(s: &str, width: usize) -> String {
    let cur = dw(s);
    if cur >= width {
        s.to_string()
    } else {
        format!("{}{}", s, " ".repeat(width - cur))
    }
}
