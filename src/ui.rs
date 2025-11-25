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

    // Draw info popup on top if enabled
    if app.show_info_popup {
        draw_info_popup(f, app);
    }
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

    // Region is always first
    if let Some(region) = &app.navigation.selected_region {
        parts.push(region.name.clone());
    } else {
        parts.push("Select Region".to_string());
        return parts.join(" > ");
    }

    // Service type
    if let Some(service_type) = &app.navigation.service_type {
        match service_type {
            ServiceType::ECS => parts.push("ECS".to_string()),
            ServiceType::EC2 => parts.push("EC2".to_string()),
            ServiceType::RDS => parts.push("RDS".to_string()),
        }
    } else if app.navigation.level == NavigationLevel::ServiceType {
        parts.push("Select Service".to_string());
        return parts.join(" > ");
    } else {
        return parts.join(" > ");
    }

    // Handle ECS-specific navigation
    if app.navigation.service_type == Some(ServiceType::ECS) {
        if let Some(cluster) = &app.navigation.selected_cluster {
            parts.push(cluster.name.clone());
        } else if app.navigation.level == NavigationLevel::Cluster
            || app.navigation.level == NavigationLevel::Service
            || app.navigation.level == NavigationLevel::Task
            || app.navigation.level == NavigationLevel::Container
        {
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
    }

    // Handle EC2-specific navigation
    if app.navigation.service_type == Some(ServiceType::EC2) {
        if let Some(instance) = &app.navigation.selected_ec2_instance {
            parts.push(instance.name.clone());
        } else if app.navigation.level == NavigationLevel::Ec2Instance {
            parts.push("Select Instance".to_string());
        }
    }

    // Handle RDS-specific navigation
    if app.navigation.service_type == Some(ServiceType::RDS) {
        if let Some(cluster) = &app.navigation.selected_rds_cluster {
            parts.push(cluster.identifier.clone());
        } else if app.navigation.level == NavigationLevel::RdsCluster
            || app.navigation.level == NavigationLevel::RdsInstance
        {
            parts.push("Select Cluster".to_string());
            return parts.join(" > ");
        }

        if let Some(instance) = &app.navigation.selected_rds_instance {
            parts.push(instance.identifier.clone());
        } else if app.navigation.level == NavigationLevel::RdsInstance {
            parts.push("Select Instance".to_string());
        }
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
        NavigationLevel::RdsCluster => draw_rds_cluster_list(f, app, area),
        NavigationLevel::RdsInstance => draw_rds_instance_list(f, app, area),
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
                ServiceType::RDS => ("RDS - Relational Database Service", "🗄️"),
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

fn draw_rds_cluster_list(f: &mut Frame, app: &App, area: Rect) {
    if app.rds_clusters.is_empty() {
        let msg = Paragraph::new("No RDS clusters found in this region")
            .block(Block::default().borders(Borders::ALL).title(" RDS Clusters "))
            .style(Style::default().fg(Color::Yellow));
        f.render_widget(msg, area);
        return;
    }

    let items: Vec<ListItem> = app
        .rds_clusters
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

            let status_color = match cluster.status.as_str() {
                "available" => Color::Green,
                "creating" | "modifying" | "backing-up" => Color::Yellow,
                "stopped" | "stopping" => Color::Red,
                _ => Color::Gray,
            };

            let endpoint_display = cluster.endpoint
                .as_ref()
                .map(|e| format!(" - {}", e))
                .unwrap_or_default();

            let lines = vec![
                Line::from(vec![
                    Span::styled("  ", style),
                    Span::styled(&cluster.identifier, style.add_modifier(Modifier::BOLD)),
                    Span::styled(" [", style),
                    Span::styled(&cluster.status, Style::default().fg(status_color).bg(if i == app.selected_index { Color::Cyan } else { Color::Reset })),
                    Span::styled("]", style),
                ]),
                Line::from(vec![
                    Span::styled("    Engine: ", style),
                    Span::styled(format!("{} {}", cluster.engine, cluster.engine_version), style),
                    Span::styled(" | Port: ", style),
                    Span::styled(cluster.port.to_string(), style),
                    Span::styled(endpoint_display, style),
                ]),
            ];

            ListItem::new(lines)
        })
        .collect();

    let list = List::new(items).block(
        Block::default()
            .borders(Borders::ALL)
            .title(" RDS Clusters (↑/↓ to navigate, Enter to view instances, Esc to go back) "),
    );

    f.render_widget(list, area);
}

fn draw_rds_instance_list(f: &mut Frame, app: &App, area: Rect) {
    if app.rds_instances.is_empty() {
        let msg = Paragraph::new("No RDS instances found in this cluster")
            .block(Block::default().borders(Borders::ALL).title(" RDS Instances "))
            .style(Style::default().fg(Color::Yellow));
        f.render_widget(msg, area);
        return;
    }

    let items: Vec<ListItem> = app
        .rds_instances
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

            let status_color = match instance.status.as_str() {
                "available" => Color::Green,
                "creating" | "modifying" | "backing-up" | "rebooting" => Color::Yellow,
                "stopped" | "stopping" | "failed" => Color::Red,
                _ => Color::Gray,
            };

            let endpoint_display = instance.endpoint
                .as_ref()
                .map(|e| format!(" - {}:{}", e, instance.port))
                .unwrap_or_default();

            let lines = vec![
                Line::from(vec![
                    Span::styled("  ", style),
                    Span::styled(&instance.identifier, style.add_modifier(Modifier::BOLD)),
                    Span::styled(" [", style),
                    Span::styled(&instance.status, Style::default().fg(status_color).bg(if i == app.selected_index { Color::Cyan } else { Color::Reset })),
                    Span::styled("]", style),
                ]),
                Line::from(vec![
                    Span::styled("    Class: ", style),
                    Span::styled(&instance.instance_class, style),
                    Span::styled(" | AZ: ", style),
                    Span::styled(&instance.availability_zone, style),
                    Span::styled(" | Storage: ", style),
                    Span::styled(format!("{}GB {}", instance.allocated_storage, instance.storage_type), style),
                    Span::styled(endpoint_display, style),
                ]),
            ];

            ListItem::new(lines)
        })
        .collect();

    let list = List::new(items).block(
        Block::default()
            .borders(Borders::ALL)
            .title(" RDS Instances (↑/↓ to navigate, 'i' for info, Esc to go back) "),
    );

    f.render_widget(list, area);
}

fn draw_footer(f: &mut Frame, app: &App, area: Rect) {
    let mut footer_text = vec![
        Span::raw(" q: quit | "),
        Span::raw("↑/↓: navigate | "),
        Span::raw("Enter: select | "),
        Span::raw("Esc: back | "),
        Span::raw("r: refresh | "),
        Span::styled("i: info", Style::default().fg(Color::Yellow)),
    ];

    if app.navigation.level == NavigationLevel::Service {
        footer_text.push(Span::raw(" | "));
        footer_text.push(Span::styled("f: deploy", Style::default().fg(Color::Green)));
    }

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

fn draw_info_popup(f: &mut Frame, app: &App) {
    let area = f.size();

    // Create centered popup (80% width, 80% height)
    let popup_width = (area.width * 80) / 100;
    let popup_height = (area.height * 80) / 100;
    let popup_x = (area.width - popup_width) / 2;
    let popup_y = (area.height - popup_height) / 2;

    let popup_area = Rect {
        x: popup_x,
        y: popup_y,
        width: popup_width,
        height: popup_height,
    };

    // Clear the popup area
    let clear_block = Block::default()
        .style(Style::default().bg(Color::Reset));
    f.render_widget(clear_block, popup_area);

    let info_text = get_info_text(app);

    let paragraph = Paragraph::new(info_text)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Info (Press 'i' or 'Esc' to close) ")
                .style(Style::default().fg(Color::Cyan))
        )
        .wrap(ratatui::widgets::Wrap { trim: true });

    f.render_widget(paragraph, popup_area);
}

fn get_info_text(app: &App) -> String {
    match app.navigation.level {
        NavigationLevel::Region => {
            if let Some(region) = app.regions.get(app.selected_index) {
                format!(
                    "Region Information\n\
                    ═══════════════════\n\n\
                    Name: {}\n\
                    ",
                    region.name
                )
            } else {
                "No region selected".to_string()
            }
        }
        NavigationLevel::ServiceType => {
            if let Some(service_type) = app.service_types.get(app.selected_index) {
                match service_type {
                    ServiceType::ECS => {
                        "ECS (Elastic Container Service)\n\
                        ════════════════════════════════\n\n\
                        AWS ECS is a fully managed container orchestration service.\n\n\
                        Features:\n\
                        • Run Docker containers at scale\n\
                        • Integrate with other AWS services\n\
                        • Support for Fargate (serverless) and EC2 launch types\n\
                        ".to_string()
                    }
                    ServiceType::EC2 => {
                        "EC2 (Elastic Compute Cloud)\n\
                        ════════════════════════════\n\n\
                        AWS EC2 provides resizable compute capacity in the cloud.\n\n\
                        Features:\n\
                        • Virtual servers (instances) in various configurations\n\
                        • Multiple instance types optimized for different use cases\n\
                        • Flexible pricing models (On-Demand, Reserved, Spot)\n\
                        ".to_string()
                    }
                    ServiceType::RDS => {
                        "RDS (Relational Database Service)\n\
                        ══════════════════════════════════\n\n\
                        AWS RDS makes it easy to set up, operate, and scale relational databases.\n\n\
                        Features:\n\
                        • Managed database service for MySQL, PostgreSQL, MariaDB, Oracle, SQL Server\n\
                        • Aurora for high-performance MySQL and PostgreSQL compatible databases\n\
                        • Automated backups, patching, and Multi-AZ deployments\n\
                        ".to_string()
                    }
                }
            } else {
                "No service type selected".to_string()
            }
        }
        NavigationLevel::Cluster => {
            if let Some(cluster) = app.clusters.get(app.selected_index) {
                format!(
                    "ECS Cluster Information\n\
                    ═══════════════════════\n\n\
                    Name: {}\n\n\
                    ARN:\n{}\n\
                    ",
                    cluster.name,
                    cluster.arn
                )
            } else {
                "No cluster selected".to_string()
            }
        }
        NavigationLevel::Service => {
            if let Some(service) = app.services.get(app.selected_index) {
                format!(
                    "ECS Service Information\n\
                    ═══════════════════════\n\n\
                    Name: {}\n\
                    Status: {}\n\
                    Desired Count: {}\n\
                    Running Count: {}\n\n\
                    ARN:\n{}\n\
                    ",
                    service.name,
                    service.status,
                    service.desired_count,
                    service.running_count,
                    service.arn
                )
            } else {
                "No service selected".to_string()
            }
        }
        NavigationLevel::Task => {
            if let Some(task) = app.tasks.get(app.selected_index) {
                format!(
                    "ECS Task Information\n\
                    ════════════════════\n\n\
                    Task ID: {}\n\
                    Status: {}\n\
                    CPU: {}\n\
                    Memory: {}\n\n\
                    ARN:\n{}\n\
                    ",
                    task.task_id,
                    task.status,
                    task.cpu,
                    task.memory,
                    task.arn
                )
            } else {
                "No task selected".to_string()
            }
        }
        NavigationLevel::Container => {
            if let Some(container) = app.containers.get(app.selected_index) {
                let runtime_info = container.runtime_id
                    .as_ref()
                    .map(|id| format!("Runtime ID: {}\n", id))
                    .unwrap_or_else(|| "Runtime ID: N/A\n".to_string());

                format!(
                    "Container Information\n\
                    ═════════════════════\n\n\
                    Name: {}\n\
                    Status: {}\n\
                    {}\n\
                    Image:\n{}\n\
                    ",
                    container.name,
                    container.status,
                    runtime_info,
                    container.image
                )
            } else {
                "No container selected".to_string()
            }
        }
        NavigationLevel::Ec2Instance => {
            if let Some(instance) = app.ec2_instances.get(app.selected_index) {
                let public_ip = instance.public_ip
                    .as_ref()
                    .map(|ip| format!("Public IP: {}\n", ip))
                    .unwrap_or_else(|| "Public IP: None\n".to_string());

                let private_ip = instance.private_ip
                    .as_ref()
                    .map(|ip| format!("Private IP: {}\n", ip))
                    .unwrap_or_else(|| "Private IP: None\n".to_string());

                let key_name = instance.key_name
                    .as_ref()
                    .map(|key| format!("SSH Key Pair: {}\n", key))
                    .unwrap_or_else(|| "SSH Key Pair: None\n".to_string());

                let ssm_status = if instance.ssm_managed {
                    "SSM Managed: ✓ Yes (SSM Session Manager available)\n"
                } else {
                    "SSM Managed: ✗ No (SSM not available, use traditional SSH)\n"
                };

                let iam_profile = instance.iam_instance_profile
                    .as_ref()
                    .map(|arn| format!("IAM Instance Profile:\n{}\n", arn))
                    .unwrap_or_else(|| "IAM Instance Profile: None\n".to_string());

                format!(
                    "EC2 Instance Information\n\
                    ════════════════════════\n\n\
                    Name: {}\n\
                    Instance ID: {}\n\
                    Type: {}\n\
                    State: {}\n\
                    Availability Zone: {}\n\
                    {}\n\
                    Network:\n\
                    {}{}\n\
                    Access:\n\
                    {}{}\n\
                    ",
                    instance.name,
                    instance.instance_id,
                    instance.instance_type,
                    instance.state,
                    instance.availability_zone,
                    iam_profile,
                    public_ip,
                    private_ip,
                    key_name,
                    ssm_status
                )
            } else {
                "No instance selected".to_string()
            }
        }
        NavigationLevel::RdsCluster => {
            if let Some(cluster) = app.rds_clusters.get(app.selected_index) {
                let endpoint = cluster.endpoint
                    .as_ref()
                    .map(|e| format!("Endpoint: {}:{}\n", e, cluster.port))
                    .unwrap_or_else(|| "Endpoint: Not available\n".to_string());

                let reader_endpoint = cluster.reader_endpoint
                    .as_ref()
                    .map(|e| format!("Reader Endpoint: {}\n", e))
                    .unwrap_or_else(|| "Reader Endpoint: Not available\n".to_string());

                let database_name = cluster.database_name
                    .as_ref()
                    .map(|db| format!("Database Name: {}\n", db))
                    .unwrap_or_else(|| "Database Name: None\n".to_string());

                let multi_az = if cluster.multi_az { "Yes" } else { "No" };
                let encrypted = if cluster.storage_encrypted { "Yes" } else { "No" };

                format!(
                    "RDS Cluster Information\n\
                    ═══════════════════════\n\n\
                    Identifier: {}\n\
                    Status: {}\n\
                    Engine: {} {}\n\n\
                    Connection:\n\
                    {}{}\n\
                    Configuration:\n\
                    {}Master Username: {}\n\
                    Multi-AZ: {}\n\
                    Storage Encrypted: {}\n\n\
                    ARN:\n{}\n\
                    ",
                    cluster.identifier,
                    cluster.status,
                    cluster.engine,
                    cluster.engine_version,
                    endpoint,
                    reader_endpoint,
                    database_name,
                    cluster.master_username,
                    multi_az,
                    encrypted,
                    cluster.arn
                )
            } else {
                "No RDS cluster selected".to_string()
            }
        }
        NavigationLevel::RdsInstance => {
            if let Some(instance) = app.rds_instances.get(app.selected_index) {
                let endpoint = instance.endpoint
                    .as_ref()
                    .map(|e| format!("Endpoint: {}:{}\n", e, instance.port))
                    .unwrap_or_else(|| "Endpoint: Not available\n".to_string());

                let cluster_info = instance.cluster_identifier
                    .as_ref()
                    .map(|id| format!("Cluster: {}\n", id))
                    .unwrap_or_else(|| "Cluster: Standalone instance\n".to_string());

                let multi_az = if instance.multi_az { "Yes" } else { "No" };

                format!(
                    "RDS Instance Information\n\
                    ════════════════════════\n\n\
                    Identifier: {}\n\
                    Status: {}\n\
                    {}\n\
                    Engine: {} {}\n\
                    Instance Class: {}\n\n\
                    Connection:\n\
                    {}\n\
                    Configuration:\n\
                    Availability Zone: {}\n\
                    Multi-AZ: {}\n\
                    Storage: {} GB ({})\n\n\
                    ARN:\n{}\n\
                    ",
                    instance.identifier,
                    instance.status,
                    cluster_info,
                    instance.engine,
                    instance.engine_version,
                    instance.instance_class,
                    endpoint,
                    instance.availability_zone,
                    multi_az,
                    instance.allocated_storage,
                    instance.storage_type,
                    instance.arn
                )
            } else {
                "No RDS instance selected".to_string()
            }
        }
    }
}
