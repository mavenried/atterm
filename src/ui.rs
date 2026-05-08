use crate::prelude::*;

pub fn ui(frame: &mut Frame, app: &App) {
    let area = frame.size();

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(1),
            Constraint::Length(3),
        ])
        .split(area);

    let (status_text, status_color) = match &app.status {
        ConnectionStatus::Connected => ("● CONNECTED", Color::Green),
        ConnectionStatus::Disconnected => ("○ DISCONNECTED", Color::Yellow),
        ConnectionStatus::Error(e) => (e.as_str(), Color::Red),
    };

    let scroll_span = if app.auto_scroll {
        Span::styled("  AUTO-SCROLL", Style::default().fg(Color::DarkGray))
    } else {
        Span::styled(
            format!("  ↕ -{}", app.scroll_offset),
            Style::default().fg(Color::Cyan),
        )
    };

    let pending_span = if !app.pending.is_empty() {
        Span::styled(
            format!("  ⧗ {} queued", app.pending.len()),
            Style::default().fg(Color::Magenta),
        )
    } else {
        Span::raw("")
    };

    let header = Paragraph::new(Line::from(vec![
        Span::styled(
            format!(" {} @ {}baud  ", app.port_path, app.baud_rate),
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            status_text,
            Style::default()
                .fg(status_color)
                .add_modifier(Modifier::BOLD),
        ),
        scroll_span,
        pending_span,
        Span::styled(
            "  [^C] quit  [↑↓] history  [PgUp/Dn] scroll  [^U] clear input",
            Style::default().fg(Color::DarkGray),
        ),
    ]))
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Blue))
            .border_type(BorderType::Rounded)
            .title(Span::styled(
                " AT Terminal ",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            )),
    );
    frame.render_widget(header, chunks[0]);

    let output_height = chunks[1]
        .inner(&Margin {
            horizontal: 1,
            vertical: 1,
        })
        .height as usize;

    let items: Vec<ListItem> = app
        .output
        .iter()
        .map(|line| {
            let style = if line.starts_with("▶ ") {
                Style::default().fg(Color::Yellow)
            } else if line.starts_with("✖") || line.to_ascii_uppercase().starts_with("ERROR") {
                Style::default().fg(Color::Red)
            } else if line == "OK" || line.starts_with("✔") {
                Style::default().fg(Color::Green)
            } else if is_terminal_response(line) {
                Style::default().fg(Color::Green)
            } else {
                Style::default().fg(Color::White)
            };
            ListItem::new(Line::from(Span::styled(line.clone(), style)))
        })
        .collect();

    let total = items.len();
    let visible = output_height.min(total);
    let end = total.saturating_sub(app.scroll_offset);
    let start = end.saturating_sub(visible);
    let visible_items: Vec<ListItem> = items.into_iter().skip(start).take(visible).collect();

    let output_list = List::new(visible_items).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::DarkGray))
            .border_type(BorderType::Rounded)
            .title(Span::styled(
                format!(" Output ({} lines) ", total),
                Style::default().fg(Color::DarkGray),
            )),
    );
    frame.render_widget(output_list, chunks[1]);

    let input_border_color = if matches!(app.read_state, ReadState::WaitingReply) {
        Color::Magenta
    } else {
        Color::Green
    };

    let input_widget = Paragraph::new(app.input.as_str())
        .style(Style::default().fg(Color::White))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(input_border_color))
                .border_type(BorderType::Rounded)
                .title(Span::styled(
                    " Command (Enter → send \\n\\r) ",
                    Style::default().fg(Color::Cyan),
                )),
        );
    frame.render_widget(input_widget, chunks[2]);

    let input_inner = chunks[2].inner(&Margin {
        horizontal: 1,
        vertical: 1,
    });
    frame.set_cursor(input_inner.x + app.cursor_pos as u16, input_inner.y);
}
