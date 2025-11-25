use anyhow::Result;
use tokio::sync::mpsc;

use crate::aws::AwsClient;

#[derive(Debug, Clone)]
pub enum AppEvent {
    ClustersLoaded(Vec<Cluster>),
    ServicesLoaded(Vec<Service>),
    TasksLoaded(Vec<Task>),
    ContainersLoaded(Vec<Container>),
    Ec2InstancesLoaded(Vec<Ec2Instance>),
    DeploymentTriggered(String),
    Error(String),
}

#[derive(Debug, Clone)]
pub struct Region {
    pub name: String,
}

#[derive(Debug, Clone)]
pub struct Cluster {
    pub arn: String,
    pub name: String,
}

#[derive(Debug, Clone)]
pub struct Service {
    #[allow(dead_code)]
    pub arn: String,
    pub name: String,
    pub status: String,
    pub desired_count: i32,
    pub running_count: i32,
}

#[derive(Debug, Clone)]
pub struct Task {
    pub arn: String,
    pub task_id: String,
    pub status: String,
    pub cpu: String,
    pub memory: String,
}

#[derive(Debug, Clone)]
pub struct Container {
    pub name: String,
    pub image: String,
    pub status: String,
    #[allow(dead_code)]
    pub runtime_id: Option<String>,
}

#[derive(Debug, Clone)]
pub struct Ec2Instance {
    pub instance_id: String,
    pub name: String,
    pub instance_type: String,
    pub state: String,
    pub public_ip: Option<String>,
    pub private_ip: Option<String>,
    #[allow(dead_code)]
    pub availability_zone: String,
    pub key_name: Option<String>,
    pub iam_instance_profile: Option<String>,
    pub ssm_managed: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub enum NavigationLevel {
    Region,
    ServiceType,  // Choose between ECS or EC2
    // ECS path
    Cluster,
    Service,
    Task,
    Container,
    // EC2 path
    Ec2Instance,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ServiceType {
    ECS,
    EC2,
}

pub struct NavigationState {
    pub level: NavigationLevel,
    pub service_type: Option<ServiceType>,
    pub selected_region: Option<Region>,
    // ECS fields
    pub selected_cluster: Option<Cluster>,
    pub selected_service: Option<Service>,
    pub selected_task: Option<Task>,
    pub selected_container: Option<Container>,
    // EC2 fields
    pub selected_ec2_instance: Option<Ec2Instance>,
}

pub struct App {
    pub aws_client: AwsClient,
    pub navigation: NavigationState,
    pub regions: Vec<Region>,
    pub service_types: Vec<ServiceType>,
    // ECS data
    pub clusters: Vec<Cluster>,
    pub services: Vec<Service>,
    pub tasks: Vec<Task>,
    pub containers: Vec<Container>,
    // EC2 data
    pub ec2_instances: Vec<Ec2Instance>,
    pub selected_index: usize,
    pub loading: bool,
    pub error_message: Option<String>,
    pub status_message: String,
    pub show_info_popup: bool,
    quit: bool,
}

impl App {
    pub async fn new() -> Result<Self> {
        let aws_client = AwsClient::new().await?;

        let regions = vec![
            Region { name: "us-east-1".to_string() },
            Region { name: "us-west-2".to_string() },
            Region { name: "eu-west-1".to_string() },
            Region { name: "ap-southeast-1".to_string() },
            Region { name: "ap-northeast-1".to_string() },
        ];

        Ok(Self {
            aws_client,
            navigation: NavigationState {
                level: NavigationLevel::Region,
                service_type: None,
                selected_region: None,
                selected_cluster: None,
                selected_service: None,
                selected_task: None,
                selected_container: None,
                selected_ec2_instance: None,
            },
            regions,
            service_types: vec![ServiceType::ECS, ServiceType::EC2],
            clusters: Vec::new(),
            services: Vec::new(),
            tasks: Vec::new(),
            containers: Vec::new(),
            ec2_instances: Vec::new(),
            selected_index: 0,
            loading: false,
            error_message: None,
            status_message: "Select a region to begin".to_string(),
            show_info_popup: false,
            quit: false,
        })
    }

    pub fn toggle_info_popup(&mut self) {
        self.show_info_popup = !self.show_info_popup;
    }

    pub fn close_info_popup(&mut self) {
        self.show_info_popup = false;
    }

    pub fn current_items_count(&self) -> usize {
        match self.navigation.level {
            NavigationLevel::Region => self.regions.len(),
            NavigationLevel::ServiceType => self.service_types.len(),
            NavigationLevel::Cluster => self.clusters.len(),
            NavigationLevel::Service => self.services.len(),
            NavigationLevel::Task => self.tasks.len(),
            NavigationLevel::Container => self.containers.len(),
            NavigationLevel::Ec2Instance => self.ec2_instances.len(),
        }
    }

    pub fn next_item(&mut self) {
        let count = self.current_items_count();
        if count > 0 {
            self.selected_index = (self.selected_index + 1) % count;
        }
    }

    pub fn previous_item(&mut self) {
        let count = self.current_items_count();
        if count > 0 {
            if self.selected_index > 0 {
                self.selected_index -= 1;
            } else {
                self.selected_index = count - 1;
            }
        }
    }

    pub async fn select_item(&mut self, tx: mpsc::Sender<AppEvent>) -> Result<()> {
        match self.navigation.level {
            NavigationLevel::Region => {
                if let Some(region) = self.regions.get(self.selected_index) {
                    self.navigation.selected_region = Some(region.clone());
                    self.navigation.level = NavigationLevel::ServiceType;
                    self.status_message = "Select service type".to_string();
                    self.selected_index = 0;
                }
            }
            NavigationLevel::ServiceType => {
                if let Some(service_type) = self.service_types.get(self.selected_index) {
                    self.navigation.service_type = Some(service_type.clone());

                    match service_type {
                        ServiceType::ECS => {
                            self.loading = true;
                            let region = self.navigation.selected_region.as_ref().unwrap().name.clone();
                            self.status_message = format!("Loading ECS clusters in {}...", region);

                            let client = self.aws_client.clone();
                            tokio::spawn(async move {
                                match client.list_clusters(&region).await {
                                    Ok(clusters) => {
                                        let _ = tx.send(AppEvent::ClustersLoaded(clusters)).await;
                                    }
                                    Err(e) => {
                                        let _ = tx.send(AppEvent::Error(format!("Failed to load clusters: {}", e))).await;
                                    }
                                }
                            });
                        }
                        ServiceType::EC2 => {
                            self.loading = true;
                            let region = self.navigation.selected_region.as_ref().unwrap().name.clone();
                            self.status_message = format!("Loading EC2 instances in {}...", region);

                            let client = self.aws_client.clone();
                            tokio::spawn(async move {
                                match client.list_ec2_instances(&region).await {
                                    Ok(instances) => {
                                        let _ = tx.send(AppEvent::Ec2InstancesLoaded(instances)).await;
                                    }
                                    Err(e) => {
                                        let _ = tx.send(AppEvent::Error(format!("Failed to load EC2 instances: {}", e))).await;
                                    }
                                }
                            });
                        }
                    }
                }
            }
            NavigationLevel::Cluster => {
                if let Some(cluster) = self.clusters.get(self.selected_index) {
                    self.navigation.selected_cluster = Some(cluster.clone());
                    self.loading = true;
                    self.status_message = format!("Loading services in {}...", cluster.name);

                    let client = self.aws_client.clone();
                    let region = self.navigation.selected_region.as_ref().unwrap().name.clone();
                    let cluster_arn = cluster.arn.clone();
                    tokio::spawn(async move {
                        match client.list_services(&region, &cluster_arn).await {
                            Ok(services) => {
                                let _ = tx.send(AppEvent::ServicesLoaded(services)).await;
                            }
                            Err(e) => {
                                let _ = tx.send(AppEvent::Error(format!("Failed to load services: {}", e))).await;
                            }
                        }
                    });
                }
            }
            NavigationLevel::Service => {
                if let Some(service) = self.services.get(self.selected_index) {
                    self.navigation.selected_service = Some(service.clone());
                    self.loading = true;
                    self.status_message = format!("Loading tasks for {}...", service.name);

                    let client = self.aws_client.clone();
                    let region = self.navigation.selected_region.as_ref().unwrap().name.clone();
                    let cluster_arn = self.navigation.selected_cluster.as_ref().unwrap().arn.clone();
                    let service_name = service.name.clone();
                    tokio::spawn(async move {
                        match client.list_tasks(&region, &cluster_arn, &service_name).await {
                            Ok(tasks) => {
                                let _ = tx.send(AppEvent::TasksLoaded(tasks)).await;
                            }
                            Err(e) => {
                                let _ = tx.send(AppEvent::Error(format!("Failed to load tasks: {}", e))).await;
                            }
                        }
                    });
                }
            }
            NavigationLevel::Task => {
                if let Some(task) = self.tasks.get(self.selected_index) {
                    self.navigation.selected_task = Some(task.clone());
                    self.loading = true;
                    self.status_message = format!("Loading containers for task {}...", task.task_id);

                    let client = self.aws_client.clone();
                    let region = self.navigation.selected_region.as_ref().unwrap().name.clone();
                    let cluster_arn = self.navigation.selected_cluster.as_ref().unwrap().arn.clone();
                    let task_arn = task.arn.clone();
                    tokio::spawn(async move {
                        match client.list_containers(&region, &cluster_arn, &task_arn).await {
                            Ok(containers) => {
                                let _ = tx.send(AppEvent::ContainersLoaded(containers)).await;
                            }
                            Err(e) => {
                                let _ = tx.send(AppEvent::Error(format!("Failed to load containers: {}", e))).await;
                            }
                        }
                    });
                }
            }
            NavigationLevel::Container => {
                // Already at deepest level for ECS
            }
            NavigationLevel::Ec2Instance => {
                // Already at deepest level for EC2
            }
        }
        Ok(())
    }

    pub fn go_back(&mut self) {
        self.selected_index = 0;
        self.error_message = None;

        match self.navigation.level {
            NavigationLevel::Region => {
                // Already at top level
            }
            NavigationLevel::ServiceType => {
                self.navigation.level = NavigationLevel::Region;
                self.navigation.service_type = None;
                self.status_message = "Select a region".to_string();
            }
            NavigationLevel::Cluster => {
                self.navigation.level = NavigationLevel::ServiceType;
                self.navigation.selected_cluster = None;
                self.clusters.clear();
                self.status_message = "Select a service type".to_string();
            }
            NavigationLevel::Service => {
                self.navigation.level = NavigationLevel::Cluster;
                self.navigation.selected_service = None;
                self.services.clear();
                self.status_message = "Select a cluster".to_string();
            }
            NavigationLevel::Task => {
                self.navigation.level = NavigationLevel::Service;
                self.navigation.selected_task = None;
                self.tasks.clear();
                self.status_message = "Select a service".to_string();
            }
            NavigationLevel::Container => {
                self.navigation.level = NavigationLevel::Task;
                self.navigation.selected_container = None;
                self.containers.clear();
                self.status_message = "Select a task".to_string();
            }
            NavigationLevel::Ec2Instance => {
                self.navigation.level = NavigationLevel::ServiceType;
                self.navigation.selected_ec2_instance = None;
                self.ec2_instances.clear();
                self.status_message = "Select a service type".to_string();
            }
        }
    }

    pub async fn refresh(&mut self, tx: mpsc::Sender<AppEvent>) -> Result<()> {
        self.selected_index = 0;

        match self.navigation.level {
            NavigationLevel::Region | NavigationLevel::ServiceType => {
                // Nothing to refresh at region or service type level
            }
            NavigationLevel::Cluster => {
                if let Some(region) = &self.navigation.selected_region {
                    self.loading = true;
                    let client = self.aws_client.clone();
                    let region_name = region.name.clone();
                    tokio::spawn(async move {
                        match client.list_clusters(&region_name).await {
                            Ok(clusters) => {
                                let _ = tx.send(AppEvent::ClustersLoaded(clusters)).await;
                            }
                            Err(e) => {
                                let _ = tx.send(AppEvent::Error(format!("Failed to refresh: {}", e))).await;
                            }
                        }
                    });
                }
            }
            NavigationLevel::Service => {
                if let (Some(region), Some(cluster)) =
                    (&self.navigation.selected_region, &self.navigation.selected_cluster) {
                    self.loading = true;
                    let client = self.aws_client.clone();
                    let region_name = region.name.clone();
                    let cluster_arn = cluster.arn.clone();
                    tokio::spawn(async move {
                        match client.list_services(&region_name, &cluster_arn).await {
                            Ok(services) => {
                                let _ = tx.send(AppEvent::ServicesLoaded(services)).await;
                            }
                            Err(e) => {
                                let _ = tx.send(AppEvent::Error(format!("Failed to refresh: {}", e))).await;
                            }
                        }
                    });
                }
            }
            NavigationLevel::Task | NavigationLevel::Container | NavigationLevel::Ec2Instance => {
                // Similar refresh logic for tasks, containers, and EC2 instances
            }
        }
        Ok(())
    }

    pub async fn handle_event(&mut self, event: AppEvent) -> Result<()> {
        self.loading = false;

        match event {
            AppEvent::ClustersLoaded(clusters) => {
                self.clusters = clusters;
                self.navigation.level = NavigationLevel::Cluster;
                self.selected_index = 0;
                self.status_message = format!("Found {} clusters", self.clusters.len());
            }
            AppEvent::ServicesLoaded(services) => {
                self.services = services;
                self.navigation.level = NavigationLevel::Service;
                self.selected_index = 0;
                self.status_message = format!("Found {} services", self.services.len());
            }
            AppEvent::TasksLoaded(tasks) => {
                self.tasks = tasks;
                self.navigation.level = NavigationLevel::Task;
                self.selected_index = 0;
                self.status_message = format!("Found {} tasks", self.tasks.len());
            }
            AppEvent::ContainersLoaded(containers) => {
                self.containers = containers;
                self.navigation.level = NavigationLevel::Container;
                self.selected_index = 0;
                self.status_message = format!("Found {} containers", self.containers.len());
            }
            AppEvent::Ec2InstancesLoaded(instances) => {
                self.ec2_instances = instances;
                self.navigation.level = NavigationLevel::Ec2Instance;
                self.selected_index = 0;
                self.status_message = format!("Found {} EC2 instances", self.ec2_instances.len());
            }
            AppEvent::DeploymentTriggered(service_name) => {
                self.status_message = format!("Deployment triggered for {}", service_name);
            }
            AppEvent::Error(msg) => {
                self.error_message = Some(msg);
                self.status_message = "Error occurred".to_string();
            }
        }

        Ok(())
    }

    pub async fn execute_command(&mut self) -> Result<()> {
        match self.navigation.level {
            NavigationLevel::Container => {
                if let Some(container) = self.containers.get(self.selected_index) {
                    if let (Some(region), Some(cluster), Some(task)) = (
                        &self.navigation.selected_region,
                        &self.navigation.selected_cluster,
                        &self.navigation.selected_task,
                    ) {
                        self.status_message = format!("Starting ECS Exec session for {}...", container.name);

                        crate::terminal::start_ecs_exec(
                            &region.name,
                            &cluster.arn,
                            &task.arn,
                            &container.name,
                        ).await?;
                    }
                }
            }
            NavigationLevel::Ec2Instance => {
                if let Some(instance) = self.ec2_instances.get(self.selected_index) {
                    self.status_message = format!("Starting SSH session to {}...", instance.instance_id);

                    crate::terminal::start_ssh_session(instance).await?;
                }
            }
            _ => {}
        }
        Ok(())
    }

    pub async fn force_deployment(&mut self, tx: mpsc::Sender<AppEvent>) -> Result<()> {
        if self.navigation.level != NavigationLevel::Service {
            return Ok(());
        }

        if let Some(service) = self.services.get(self.selected_index) {
            if let (Some(region), Some(cluster)) = (
                &self.navigation.selected_region,
                &self.navigation.selected_cluster,
            ) {
                self.loading = true;
                self.status_message = format!("Triggering deployment for {}...", service.name);

                let client = self.aws_client.clone();
                let region_name = region.name.clone();
                let cluster_arn = cluster.arn.clone();
                let service_name = service.name.clone();

                tokio::spawn(async move {
                    match client
                        .force_new_deployment(&region_name, &cluster_arn, &service_name)
                        .await
                    {
                        Ok(()) => {
                            let _ = tx.send(AppEvent::DeploymentTriggered(service_name)).await;
                        }
                        Err(e) => {
                            let _ = tx
                                .send(AppEvent::Error(format!("Failed to trigger deployment: {}", e)))
                                .await;
                        }
                    }
                });
            }
        }
        Ok(())
    }

    pub fn can_quit(&self) -> bool {
        true
    }

    pub fn should_quit(&self) -> bool {
        self.quit
    }
}
