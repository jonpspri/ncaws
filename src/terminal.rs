use anyhow::{Context, Result};
use std::process::{Command, Stdio};

use crate::app::Ec2Instance;

/// Start an ECS Exec session using the AWS CLI
///
/// This function spawns the AWS CLI command to start an interactive session
/// with a container running in ECS using ECS Exec (which uses SSM Session Manager).
///
/// Prerequisites:
/// - AWS CLI v2 must be installed
/// - Session Manager plugin must be installed
/// - The ECS task must have been started with enableExecuteCommand=true
/// - The task role must have the necessary SSM permissions
pub async fn start_ecs_exec(
    region: &str,
    cluster_arn: &str,
    task_arn: &str,
    container_name: &str,
) -> Result<()> {
    // Extract cluster name from ARN
    let cluster_name = cluster_arn
        .split('/')
        .last()
        .context("Invalid cluster ARN")?;

    // Extract task ID from ARN
    let task_id = task_arn
        .split('/')
        .last()
        .context("Invalid task ARN")?;

    // Temporarily exit the TUI to run the interactive session
    crossterm::terminal::disable_raw_mode()?;
    crossterm::execute!(
        std::io::stdout(),
        crossterm::terminal::LeaveAlternateScreen
    )?;

    println!("\n╔════════════════════════════════════════════════════════════════╗");
    println!("║          Starting ECS Exec Session                            ║");
    println!("╟────────────────────────────────────────────────────────────────╢");
    println!("║ Region:    {:<51} ║", region);
    println!("║ Cluster:   {:<51} ║", cluster_name);
    println!("║ Task:      {:<51} ║", task_id);
    println!("║ Container: {:<51} ║", container_name);
    println!("╟────────────────────────────────────────────────────────────────╢");
    println!("║ Type 'exit' or press Ctrl+D to return to the console          ║");
    println!("╚════════════════════════════════════════════════════════════════╝\n");

    // Build the AWS ECS execute-command
    let status = Command::new("aws")
        .arg("ecs")
        .arg("execute-command")
        .arg("--region")
        .arg(region)
        .arg("--cluster")
        .arg(cluster_name)
        .arg("--task")
        .arg(task_id)
        .arg("--container")
        .arg(container_name)
        .arg("--interactive")
        .arg("--command")
        .arg("/bin/sh")
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
        .context("Failed to execute AWS CLI command. Make sure AWS CLI v2 and Session Manager plugin are installed.")?;

    if !status.success() {
        eprintln!("\n❌ ECS Exec session failed with status: {}", status);
        eprintln!("\nCommon issues:");
        eprintln!("  1. The task was not started with enableExecuteCommand=true");
        eprintln!("  2. The task role lacks necessary SSM permissions");
        eprintln!("  3. Session Manager plugin is not installed");
        eprintln!("  4. The container is not running or doesn't have /bin/sh");
        eprintln!("\nFor more details, visit:");
        eprintln!("  https://docs.aws.amazon.com/AmazonECS/latest/developerguide/ecs-exec.html\n");
    } else {
        println!("\n✓ ECS Exec session ended successfully\n");
    }

    println!("Press Enter to return to the console...");
    let mut input = String::new();
    std::io::stdin().read_line(&mut input)?;

    // Re-enter the TUI
    crossterm::terminal::enable_raw_mode()?;
    crossterm::execute!(
        std::io::stdout(),
        crossterm::terminal::EnterAlternateScreen
    )?;

    Ok(())
}

/// Check if ECS Exec is enabled for a task
///
/// This function can be used to verify if a task has ECS Exec enabled
/// before attempting to start a session.
#[allow(dead_code)]
pub async fn check_exec_enabled(
    region: &str,
    cluster_name: &str,
    task_id: &str,
) -> Result<bool> {
    let output = Command::new("aws")
        .arg("ecs")
        .arg("describe-tasks")
        .arg("--region")
        .arg(region)
        .arg("--cluster")
        .arg(cluster_name)
        .arg("--tasks")
        .arg(task_id)
        .arg("--query")
        .arg("tasks[0].enableExecuteCommand")
        .arg("--output")
        .arg("text")
        .output()
        .context("Failed to check if ECS Exec is enabled")?;

    let result = String::from_utf8(output.stdout)?;
    Ok(result.trim() == "True")
}

/// Start an SSH session to an EC2 instance
///
/// This function spawns an SSH command to connect to an EC2 instance.
/// It will attempt to use AWS SSM Session Manager first (recommended),
/// and fall back to traditional SSH if SSM is not available.
///
/// Prerequisites:
/// - For SSM: AWS CLI v2 and Session Manager plugin must be installed
/// - For SSH: SSH client must be installed and SSH key must be configured
pub async fn start_ssh_session(instance: &Ec2Instance) -> Result<()> {
    // Temporarily exit the TUI to run the interactive session
    crossterm::terminal::disable_raw_mode()?;
    crossterm::execute!(
        std::io::stdout(),
        crossterm::terminal::LeaveAlternateScreen
    )?;

    println!("\n╔════════════════════════════════════════════════════════════════╗");
    println!("║          EC2 SSH Connection                                    ║");
    println!("╟────────────────────────────────────────────────────────────────╢");
    println!("║ Instance:  {:<51} ║", instance.instance_id);
    println!("║ Name:      {:<51} ║", instance.name);
    println!("║ State:     {:<51} ║", instance.state);
    if let Some(public_ip) = &instance.public_ip {
        println!("║ Public IP: {:<51} ║", public_ip);
    }
    if let Some(private_ip) = &instance.private_ip {
        println!("║ Private IP:{:<51} ║", private_ip);
    }
    println!("╟────────────────────────────────────────────────────────────────╢");
    println!("║ Choose connection method:                                      ║");
    println!("║   1) AWS Systems Manager (SSM) - Recommended                   ║");
    println!("║   2) Traditional SSH                                           ║");
    println!("║   3) Cancel                                                    ║");
    println!("╚════════════════════════════════════════════════════════════════╝\n");

    print!("Enter choice (1-3): ");
    use std::io::Write;
    std::io::stdout().flush()?;

    let mut choice = String::new();
    std::io::stdin().read_line(&mut choice)?;

    let choice = choice.trim();

    match choice {
        "1" => {
            println!("\nStarting SSM session...\n");
            let status = Command::new("aws")
                .arg("ssm")
                .arg("start-session")
                .arg("--target")
                .arg(&instance.instance_id)
                .stdin(Stdio::inherit())
                .stdout(Stdio::inherit())
                .stderr(Stdio::inherit())
                .status()
                .context("Failed to execute AWS SSM command. Make sure AWS CLI v2 and Session Manager plugin are installed.")?;

            if !status.success() {
                eprintln!("\n❌ SSM session failed with status: {}", status);
                eprintln!("\nCommon issues:");
                eprintln!("  1. Session Manager plugin is not installed");
                eprintln!("  2. Instance doesn't have SSM agent installed/running");
                eprintln!("  3. Instance role lacks necessary SSM permissions");
                eprintln!("  4. Security group/network doesn't allow SSM connection");
                eprintln!("\nFor more details, visit:");
                eprintln!("  https://docs.aws.amazon.com/systems-manager/latest/userguide/session-manager.html\n");
            } else {
                println!("\n✓ SSM session ended successfully\n");
            }
        }
        "2" => {
            if instance.state != "running" {
                eprintln!("\n❌ Instance is not in running state (current: {})", instance.state);
            } else if let Some(ip) = instance.public_ip.as_ref().or(instance.private_ip.as_ref()) {
                println!("\n╔════════════════════════════════════════════════════════════════╗");
                println!("║ SSH Connection Options                                        ║");
                println!("╟────────────────────────────────────────────────────────────────╢");
                println!("║ Enter SSH username (e.g., ec2-user, ubuntu, admin):           ║");
                println!("╚════════════════════════════════════════════════════════════════╝\n");

                print!("Username [ec2-user]: ");
                std::io::stdout().flush()?;

                let mut username = String::new();
                std::io::stdin().read_line(&mut username)?;
                let username = username.trim();
                let username = if username.is_empty() { "ec2-user" } else { username };

                println!("\nConnecting via SSH to {}@{}...\n", username, ip);
                println!("Note: You may need to specify your SSH key with -i flag if the default key doesn't work.\n");

                let status = Command::new("ssh")
                    .arg(format!("{}@{}", username, ip))
                    .stdin(Stdio::inherit())
                    .stdout(Stdio::inherit())
                    .stderr(Stdio::inherit())
                    .status()
                    .context("Failed to execute SSH command. Make sure SSH client is installed.")?;

                if !status.success() {
                    eprintln!("\n❌ SSH connection failed");
                    eprintln!("\nTroubleshooting:");
                    eprintln!("  1. Ensure you have the correct SSH key");
                    eprintln!("  2. Try: ssh -i /path/to/key.pem {}@{}", username, ip);
                    eprintln!("  3. Check security group allows SSH (port 22)");
                    eprintln!("  4. Verify network connectivity\n");
                } else {
                    println!("\n✓ SSH session ended successfully\n");
                }
            } else {
                eprintln!("\n❌ No IP address available for this instance\n");
            }
        }
        "3" => {
            println!("\nConnection cancelled.\n");
        }
        _ => {
            eprintln!("\nInvalid choice. Connection cancelled.\n");
        }
    }

    println!("Press Enter to return to the console...");
    let mut input = String::new();
    std::io::stdin().read_line(&mut input)?;

    // Re-enter the TUI
    crossterm::terminal::enable_raw_mode()?;
    crossterm::execute!(
        std::io::stdout(),
        crossterm::terminal::EnterAlternateScreen
    )?;

    Ok(())
}
