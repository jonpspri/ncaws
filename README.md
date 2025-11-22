# ncaws - Terminal-based AWS Console

A rich terminal user interface (TUI) for navigating and interacting with AWS services including ECS (Elastic Container Service) and EC2 (Elastic Compute Cloud). Built with Rust and Ratatui.

## Features

- 📊 **Hierarchical Navigation**: Navigate through AWS regions → service type (ECS/EC2) → resources
  - **ECS Path**: Clusters → Services → Tasks → Containers
  - **EC2 Path**: EC2 Instances
- 🔍 **Real-time Information**: View live status of ECS services, tasks, containers, and EC2 instances
- 💻 **Interactive Terminals**:
  - Execute commands directly in ECS containers using ECS Exec
  - SSH into EC2 instances (SSM Session Manager or traditional SSH)
- ⚡ **Fast & Responsive**: Built with async Rust for quick navigation
- 🎨 **Clean UI**: Intuitive terminal interface with vim-style navigation

## Prerequisites

1. **Rust** (latest stable version)
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   ```

2. **AWS CLI v2**
   ```bash
   # macOS
   brew install awscli

   # Linux
   curl "https://awscli.amazonaws.com/awscli-exe-linux-x86_64.zip" -o "awscliv2.zip"
   unzip awscliv2.zip
   sudo ./aws/install
   ```

3. **Session Manager Plugin** (for ECS Exec and EC2 SSM)
   ```bash
   # macOS
   brew install --cask session-manager-plugin

   # Linux
   curl "https://s3.amazonaws.com/session-manager-downloads/plugin/latest/ubuntu_64bit/session-manager-plugin.deb" -o "session-manager-plugin.deb"
   sudo dpkg -i session-manager-plugin.deb
   ```

4. **SSH Client** (for EC2 SSH connections, usually pre-installed on macOS/Linux)

5. **AWS Credentials**
   Configure your AWS credentials using one of these methods:
   - Environment variables (`AWS_ACCESS_KEY_ID`, `AWS_SECRET_ACCESS_KEY`)
   - AWS CLI configuration (`~/.aws/credentials`)
   - IAM role (if running on EC2/ECS)

## Installation

```bash
# Clone and build
git clone <repository-url>
cd ncaws
cargo build --release

# Run
cargo run --release
```

## Usage

### Navigation

- **↑/↓** or **j/k**: Navigate up/down in lists
- **Enter**: Select item and drill down
- **Esc** or **Backspace**: Go back to previous level
- **r**: Refresh current view
- **q**: Quit application

### ECS Exec

When viewing containers:
- Press **e** to start an interactive terminal session with the selected container

**Requirements for ECS Exec:**
- Task must be started with `enableExecuteCommand=true`
- Task role must have these permissions:
  ```json
  {
    "Version": "2012-10-17",
    "Statement": [
      {
        "Effect": "Allow",
        "Action": [
          "ssmmessages:CreateControlChannel",
          "ssmmessages:CreateDataChannel",
          "ssmmessages:OpenControlChannel",
          "ssmmessages:OpenDataChannel"
        ],
        "Resource": "*"
      }
    ]
  }
  ```

### EC2 SSH

When viewing EC2 instances:
- Press **s** to connect to the selected EC2 instance

**Connection options:**
1. **AWS Systems Manager (SSM)** - Recommended, no SSH keys required
   - Requires SSM agent running on the instance
   - Instance role must have `AmazonSSMManagedInstanceCore` policy
   - No security group configuration needed for SSH port

2. **Traditional SSH** - Direct SSH connection
   - Requires SSH key pair
   - Security group must allow port 22
   - Instance must have public IP or be accessible from your network

## Navigation Flow

### ECS Path
```
Regions
  ↓ (select)
Service Type
  ↓ (select ECS)
Clusters (in selected region)
  ↓ (select)
Services (in selected cluster)
  ↓ (select)
Tasks (in selected service)
  ↓ (select)
Containers (in selected task)
  ↓ (press 'e')
Interactive ECS Exec Session
```

### EC2 Path
```
Regions
  ↓ (select)
Service Type
  ↓ (select EC2)
EC2 Instances (in selected region)
  ↓ (press 's')
SSH Connection (SSM or traditional)
```

## Architecture

- **src/main.rs**: Application entry point and event loop
- **src/app.rs**: Application state and navigation logic
- **src/aws.rs**: AWS SDK integration for ECS operations
- **src/ui.rs**: Ratatui UI rendering components
- **src/terminal.rs**: ECS Exec and EC2 SSH integration

## Troubleshooting

### "No clusters found"
- Verify you have ECS clusters in the selected region
- Check your AWS credentials have permission to call `ecs:ListClusters`

### "No EC2 instances found"
- Verify you have EC2 instances in the selected region
- Check your AWS credentials have permission to call `ec2:DescribeInstances`

### ECS Exec fails
- Ensure the task was started with `enableExecuteCommand=true`
- Verify the task role has SSM permissions
- Confirm Session Manager plugin is installed: `session-manager-plugin --version`
- Check container has `/bin/sh` or modify the command in `terminal.rs`

### EC2 SSH/SSM fails
- **SSM Issues:**
  - Verify SSM agent is running on the instance
  - Check instance role has `AmazonSSMManagedInstanceCore` policy
  - Ensure Session Manager plugin is installed: `session-manager-plugin --version`
- **SSH Issues:**
  - Verify you have the correct SSH key
  - Check security group allows port 22 from your IP
  - Ensure instance has a public IP or is accessible from your network
  - Try specifying key: `ssh -i /path/to/key.pem username@ip`

### Permission errors
Make sure your AWS credentials have these permissions:
- **ECS:**
  - `ecs:ListClusters`
  - `ecs:DescribeClusters`
  - `ecs:ListServices`
  - `ecs:DescribeServices`
  - `ecs:ListTasks`
  - `ecs:DescribeTasks`
  - `ecs:ExecuteCommand` (for ECS Exec)
- **EC2:**
  - `ec2:DescribeInstances`
  - `ssm:StartSession` (for SSM connections)

## Future Enhancements

- [ ] Support for Fargate and EC2 launch types in ECS
- [ ] Custom command input for ECS Exec
- [ ] Task logs viewer
- [ ] Service metrics and health checks
- [ ] Search/filter functionality
- [ ] Multi-region view
- [ ] Task definition viewer
- [ ] Support for other shells (bash, zsh)
- [ ] EC2 instance start/stop controls
- [ ] EC2 instance filtering by tags, state, etc.
- [ ] Support for additional AWS services (RDS, Lambda, etc.)

## License

MIT

## Contributing

Contributions welcome! Please open an issue or submit a pull request.
