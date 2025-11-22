use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame,
};

use crate::app::{App, NavigationLevel, ServiceType};

pub fn draw(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),  // Header
            Constraint::Min(0),     // Main content
            Constraint::Length(3),  // Footer
        ])
        .split(f.size());

    draw_header(f, app, chunks[0]);
    draw_main_content(f, app, chunks[1]);
    draw_footer(f, app, chunks[2]);
}

fn draw_header(f: &mut Frame, app: &App, area: Rect) {
    let breadcrumb = build_breadcrumb(app);
    let title = Paragraph::new(breadcrumb)
        .block(Block::default().borders(Borders::ALL).title(" AWS ECS Console "))
        .style(Style::default().fg(Color::Cyan));

    f.render_widget(title, area);
}

fn build_breadcrumb(app: &App) -> String {
    let mut parts = vec![];

    if let Some(region) = &app.navigation.selected_region {
        parts.push(region.name.clone());
    } else {
        parts.push("Select Region".to_string());
        return parts.join(" > ");
    }

    if let Some(cluster) = &app.navigation.selected_cluster {
        parts.push(cluster.name.clone());
    } else if app.navigation.level != NavigationLevel::Region {
        parts.push("Select Cluster".to_string());
        return parts.join(" > ");
    }

    if let Some(service) = &app.navigation.selected_service {
        parts.push(service.name.clone());
    } else if app.navigation.level == NavigationLevel::Service
        || app.navigation.level == NavigationLevel::Task
        || app.navigation.level == NavigationLevel::Container
    {
        parts.push("Select Service".to_string());
        return parts.join(" > ");
    }

    if let Some(task) = &app.navigation.selected_task {
        parts.push(task.task_id.clone());
    } else if app.navigation.level == NavigationLevel::Task
        || app.navigation.level == NavigationLevel::Container
    {
        parts.push("Select Task".to_string());
        return parts.join(" > ");
    }

    if let Some(container) = &app.navigation.selected_container {
        parts.push(container.name.clone());
    } else if app.navigation.level == NavigationLevel::Container {
        parts.push("Select Container".to_string());
    }

    parts.join(" > ")
}

fn draw_main_content(f: &mut Frame, app: &App, area: Rect) {
    if app.loading {
        let msg = Paragraph::new("Loading...")
            .block(Block::default().borders(Borders::ALL))
            .style(Style::default().fg(Color::Yellow));
        f.render_widget(msg, area);
        return;
    }

    match app.navigation.level {
        NavigationLevel::Region => draw_region_list(f, app, area),
        NavigationLevel::ServiceType => draw_service_type_list(f, app, area),
        NavigationLevel::Cluster => draw_cluster_list(f, app, area),
        NavigationLevel::Service => draw_service_list(f, app, area),
        NavigationLevel::Task => draw_task_list(f, app, area),
        NavigationLevel::Container => draw_container_list(f, app, area),
        NavigationLevel::Ec2Instance => draw_ec2_instance_list(f, app, area),
    }
}

fn draw_region_list(f: &mut Frame, app: &App, area: Rect) {
    let items: Vec<ListItem> = app
        .regions
        .iter()
        .enumerate()
        .map(|(i, region)| {
            let style = if i == app.selected_index {
                Style::default()
                    .fg(Color::Black)
                    .bg(Color::Cyan)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            };

            let content = Line::from(vec![
                Span::styled("  ", style),
                Span::styled(&region.name, style),
            ]);

            ListItem::new(content)
        })
        .collect();

    let list = List::new(items).block(
        Block::default()
            .borders(Borders::ALL)
            .title(" Regions (↑/↓ to navigate, Enter to select) "),
    );

    f.render_widget(list, area);
}

fn draw_service_type_list(f: &mut Frame, app: &App, area: Rect) {
    let items: Vec<ListItem> = app
        .service_types
        .iter()
        .enumerate()
        .map(|(i, service_type)| {
            let style = if i == app.selected_index {
                Style::default()
                    .fg(Color::Black)
                    .bg(Color::Cyan)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            };

            let (name, icon) = match service_type {
                ServiceType::ECS => ("ECS - Elastic Container Service", "🐳"),
                ServiceType::EC2 => ("EC2 - Elastic Compute Cloud", "💻"),
            };

            let content = Line::from(vec![
                Span::styled("  ", style),
                Span::styled(icon, style),
                Span::styled(" ", style),
                Span::styled(name, style),
            ]);

            ListItem::new(content)
        })
        .collect();

    let list = List::new(items).block(
        Block::default()
            .borders(Borders::ALL)
            .title(" Service Type (↑/↓ to navigate, Enter to select, Esc to go back) "),
    );

    f.render_widget(list, area);
}

fn draw_cluster_list(f: &mut Frame, app: &App, area: Rect) {
    if app.clusters.is_empty() {
        let msg = Paragraph::new("No clusters found in this region")
            .block(Block::default().borders(Borders::ALL).title(" Clusters "))
            .style(Style::default().fg(Color::Yellow));
        f.render_widget(msg, area);
        return;
    }

    let items: Vec<ListItem> = app
        .clusters
        .iter()
        .enumerate()
        .map(|(i, cluster)| {
            let style = if i == app.selected_index {
                Style::default()
                    .fg(Color::Black)
                    .bg(Color::Cyan)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            };

            let content = Line::from(vec![
                Span::styled("  ", style),
                Span::styled(&cluster.name, style),
            ]);

            ListItem::new(content)
        })
        .collect();

    let list = List::new(items).block(
        Block::default()
            .borders(Borders::ALL)
            .title(" Clusters (↑/↓ to navigate, Enter to select, Esc to go back) "),
    );

    f.render_widget(list, area);
}

fn draw_service_list(f: &mut Frame, app: &App, area: Rect) {
    if app.services.is_empty() {
        let msg = Paragraph::new("No services found in this cluster")
            .block(Block::default().borders(Borders::ALL).title(" Services "))
            .style(Style::default().fg(Color::Yellow));
        f.render_widget(msg, area);
        return;
    }

    let items: Vec<ListItem> = app
        .services
        .iter()
        .enumerate()
        .map(|(i, service)| {
            let style = if i == app.selected_index {
                Style::default()
                    .fg(Color::Black)
                    .bg(Color::Cyan)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            };

            let status_color = match service.status.as_str() {
                "ACTIVE" => Color::Green,
                "DRAINING" => Color::Yellow,
                _ => Color::Red,
            };

            let content = Line::from(vec![
                Span::styled("  ", style),
                Span::styled(&service.name, style),
                Span::styled(" [", style),
                Span::styled(&service.status, Style::default().fg(status_color).bg(if i == app.selected_index { Color::Cyan } else { Color::Reset })),
                Span::styled("] ", style),
                Span::styled(
                    format!("{}/{} tasks", service.running_count, service.desired_count),
                    style,
                ),
            ]);

            ListItem::new(content)
        })
        .collect();

    let list = List::new(items).block(
        Block::default()
            .borders(Borders::ALL)
            .title(" Services (↑/↓ to navigate, Enter to select, Esc to go back) "),
    );

    f.render_widget(list, area);
}

fn draw_task_list(f: &mut Frame, app: &App, area: Rect) {
    if app.tasks.is_empty() {
        let msg = Paragraph::new("No tasks found for this service")
            .block(Block::default().borders(Borders::ALL).title(" Tasks "))
            .style(Style::default().fg(Color::Yellow));
        f.render_widget(msg, area);
        return;
    }

    let items: Vec<ListItem> = app
        .tasks
        .iter()
        .enumerate()
        .map(|(i, task)| {
            let style = if i == app.selected_index {
                Style::default()
                    .fg(Color::Black)
                    .bg(Color::Cyan)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            };

            let status_color = match task.status.as_str() {
                "RUNNING" => Color::Green,
                "PENDING" => Color::Yellow,
                "STOPPED" => Color::Red,
                _ => Color::Gray,
            };

            let content = Line::from(vec![
                Span::styled("  ", style),
                Span::styled(&task.task_id, style),
                Span::styled(" [", style),
                Span::styled(&task.status, Style::default().fg(status_color).bg(if i == app.selected_index { Color::Cyan } else { Color::Reset })),
                Span::styled("] CPU: ", style),
                Span::styled(&task.cpu, style),
                Span::styled(" MEM: ", style),
                Span::styled(&task.memory, style),
            ]);

            ListItem::new(content)
        })
        .collect();

    let list = List::new(items).block(
        Block::default()
            .borders(Borders::ALL)
            .title(" Tasks (↑/↓ to navigate, Enter to select, Esc to go back) "),
    );

    f.render_widget(list, area);
}

fn draw_container_list(f: &mut Frame, app: &App, area: Rect) {
    if app.containers.is_empty() {
        let msg = Paragraph::new("No containers found for this task")
            .block(Block::default().borders(Borders::ALL).title(" Containers "))
            .style(Style::default().fg(Color::Yellow));
        f.render_widget(msg, area);
        return;
    }

    let items: Vec<ListItem> = app
        .containers
        .iter()
        .enumerate()
        .map(|(i, container)| {
            let style = if i == app.selected_index {
                Style::default()
                    .fg(Color::Black)
                    .bg(Color::Cyan)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            };

            let status_color = match container.status.as_str() {
                "RUNNING" => Color::Green,
                "PENDING" => Color::Yellow,
                "STOPPED" => Color::Red,
                _ => Color::Gray,
            };

            let lines = vec![
                Line::from(vec![
                    Span::styled("  ", style),
                    Span::styled(&container.name, style.add_modifier(Modifier::BOLD)),
                    Span::styled(" [", style),
                    Span::styled(&container.status, Style::default().fg(status_color).bg(if i == app.selected_index { Color::Cyan } else { Color::Reset })),
                    Span::styled("]", style),
                ]),
                Line::from(vec![
                    Span::styled("    Image: ", style),
                    Span::styled(&container.image, style),
                ]),
            ];

            ListItem::new(lines)
        })
        .collect();

    let list = List::new(items).block(
        Block::default()
            .borders(Borders::ALL)
            .title(" Containers (↑/↓ to navigate, 'e' to exec, Esc to go back) "),
    );

    f.render_widget(list, area);
}

fn draw_ec2_instance_list(f: &mut Frame, app: &App, area: Rect) {
    if app.ec2_instances.is_empty() {
        let msg = Paragraph::new("No EC2 instances found in this region")
            .block(Block::default().borders(Borders::ALL).title(" EC2 Instances "))
            .style(Style::default().fg(Color::Yellow));
        f.render_widget(msg, area);
        return;
    }

    let items: Vec<ListItem> = app
        .ec2_instances
        .iter()
        .enumerate()
        .map(|(i, instance)| {
            let style = if i == app.selected_index {
                Style::default()
                    .fg(Color::Black)
                    .bg(Color::Cyan)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            };

            let state_color = match instance.state.as_str() {
                "running" => Color::Green,
                "stopped" => Color::Red,
                "pending" => Color::Yellow,
                "stopping" => Color::Yellow,
                "terminated" => Color::DarkGray,
                _ => Color::Gray,
            };

            let mut line2_spans = vec![
                Span::styled("    Type: ", style),
                Span::styled(instance.instance_type.clone(), style),
                Span::styled(" | State: ", style),
                Span::styled(instance.state.clone(), Style::default().fg(state_color).bg(if i == app.selected_index { Color::Cyan } else { Color::Reset })),
                Span::styled(" | ", style),
            ];

            // Add IP display
            if let Some(public_ip) = &instance.public_ip {
                line2_spans.push(Span::styled("Public: ".to_string(), style));
                line2_spans.push(Span::styled(public_ip.clone(), style));
            } else if let Some(private_ip) = &instance.private_ip {
                line2_spans.push(Span::styled("Private: ".to_string(), style));
                line2_spans.push(Span::styled(private_ip.clone(), style));
            } else {
                line2_spans.push(Span::styled("No IP".to_string(), style));
            }

            let lines = vec![
                Line::from(vec![
                    Span::styled("  ", style),
                    Span::styled(instance.name.clone(), style.add_modifier(Modifier::BOLD)),
                    Span::styled(" (", style),
                    Span::styled(instance.instance_id.clone(), style),
                    Span::styled(")", style),
                ]),
                Line::from(line2_spans),
            ];

            ListItem::new(lines)
        })
        .collect();

    let list = List::new(items).block(
        Block::default()
            .borders(Borders::ALL)
            .title(" EC2 Instances (↑/↓ to navigate, 's' to SSH, Esc to go back) "),
    );

    f.render_widget(list, area);
}

fn draw_footer(f: &mut Frame, app: &App, area: Rect) {
    let mut footer_text = vec![
        Span::raw(" q: quit | "),
        Span::raw("↑/↓: navigate | "),
        Span::raw("Enter: select | "),
        Span::raw("Esc: back | "),
        Span::raw("r: refresh"),
    ];

    if app.navigation.level == NavigationLevel::Container {
        footer_text.push(Span::raw(" | "));
        footer_text.push(Span::styled("e: exec", Style::default().fg(Color::Green)));
    }

    if app.navigation.level == NavigationLevel::Ec2Instance {
        footer_text.push(Span::raw(" | "));
        footer_text.push(Span::styled("s: SSH", Style::default().fg(Color::Green)));
    }

    footer_text.push(Span::raw(" | "));
    footer_text.push(Span::raw(&app.status_message));

    if let Some(error) = &app.error_message {
        footer_text.push(Span::raw(" | "));
        footer_text.push(Span::styled(
            format!("ERROR: {}", error),
            Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
        ));
    }

    let footer = Paragraph::new(Line::from(footer_text))
        .block(Block::default().borders(Borders::ALL))
        .style(Style::default().fg(Color::White));

    f.render_widget(footer, area);
}
