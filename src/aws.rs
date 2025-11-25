use anyhow::Result;
use aws_config::BehaviorVersion;
use aws_sdk_ecs::Client as EcsClient;
use aws_sdk_ec2::Client as Ec2Client;
use aws_sdk_ssm::Client as SsmClient;
use aws_sdk_rds::Client as RdsClient;

use crate::app::{Cluster, Container, Ec2Instance, RdsCluster, RdsInstance, Service, Task};

#[derive(Clone)]
pub struct AwsClient {
    config: aws_config::SdkConfig,
}

impl AwsClient {
    pub async fn new() -> Result<Self> {
        let config = aws_config::defaults(BehaviorVersion::latest())
            .load()
            .await;

        Ok(Self { config })
    }

    fn get_ecs_client(&self, region: &str) -> EcsClient {
        let region_provider = aws_sdk_ecs::config::Region::new(region.to_string());
        let ecs_config = aws_sdk_ecs::config::Builder::from(&self.config)
            .region(region_provider)
            .build();

        EcsClient::from_conf(ecs_config)
    }

    fn get_ec2_client(&self, region: &str) -> Ec2Client {
        let region_provider = aws_sdk_ec2::config::Region::new(region.to_string());
        let ec2_config = aws_sdk_ec2::config::Builder::from(&self.config)
            .region(region_provider)
            .build();

        Ec2Client::from_conf(ec2_config)
    }

    fn get_ssm_client(&self, region: &str) -> SsmClient {
        let region_provider = aws_sdk_ssm::config::Region::new(region.to_string());
        let ssm_config = aws_sdk_ssm::config::Builder::from(&self.config)
            .region(region_provider)
            .build();

        SsmClient::from_conf(ssm_config)
    }

    fn get_rds_client(&self, region: &str) -> RdsClient {
        let region_provider = aws_sdk_rds::config::Region::new(region.to_string());
        let rds_config = aws_sdk_rds::config::Builder::from(&self.config)
            .region(region_provider)
            .build();

        RdsClient::from_conf(rds_config)
    }

    pub async fn list_clusters(&self, region: &str) -> Result<Vec<Cluster>> {
        let client = self.get_ecs_client(region);

        let resp = client.list_clusters().send().await?;

        let cluster_arns = resp.cluster_arns();

        if cluster_arns.is_empty() {
            return Ok(Vec::new());
        }

        // Describe clusters to get more details
        let describe_resp = client
            .describe_clusters()
            .set_clusters(Some(cluster_arns.to_vec()))
            .send()
            .await?;

        let clusters = describe_resp
            .clusters()
            .iter()
            .filter_map(|c| {
                Some(Cluster {
                    arn: c.cluster_arn()?.to_string(),
                    name: c.cluster_name()?.to_string(),
                })
            })
            .collect();

        Ok(clusters)
    }

    pub async fn list_services(&self, region: &str, cluster_arn: &str) -> Result<Vec<Service>> {
        let client = self.get_ecs_client(region);

        let resp = client
            .list_services()
            .cluster(cluster_arn)
            .send()
            .await?;

        let service_arns = resp.service_arns();

        if service_arns.is_empty() {
            return Ok(Vec::new());
        }

        // Describe services to get more details
        let describe_resp = client
            .describe_services()
            .cluster(cluster_arn)
            .set_services(Some(service_arns.to_vec()))
            .send()
            .await?;

        let services = describe_resp
            .services()
            .iter()
            .filter_map(|s| {
                Some(Service {
                    arn: s.service_arn()?.to_string(),
                    name: s.service_name()?.to_string(),
                    status: s.status()?.to_string(),
                    desired_count: s.desired_count(),
                    running_count: s.running_count(),
                })
            })
            .collect();

        Ok(services)
    }

    pub async fn list_tasks(
        &self,
        region: &str,
        cluster_arn: &str,
        service_name: &str,
    ) -> Result<Vec<Task>> {
        let client = self.get_ecs_client(region);

        let resp = client
            .list_tasks()
            .cluster(cluster_arn)
            .service_name(service_name)
            .send()
            .await?;

        let task_arns = resp.task_arns();

        if task_arns.is_empty() {
            return Ok(Vec::new());
        }

        // Describe tasks to get more details
        let describe_resp = client
            .describe_tasks()
            .cluster(cluster_arn)
            .set_tasks(Some(task_arns.to_vec()))
            .send()
            .await?;

        let tasks = describe_resp
            .tasks()
            .iter()
            .filter_map(|t| {
                let arn = t.task_arn()?.to_string();
                let task_id = arn.split('/').last()?.to_string();

                Some(Task {
                    arn,
                    task_id,
                    status: t.last_status().unwrap_or("UNKNOWN").to_string(),
                    cpu: t.cpu().unwrap_or("N/A").to_string(),
                    memory: t.memory().unwrap_or("N/A").to_string(),
                })
            })
            .collect();

        Ok(tasks)
    }

    pub async fn list_containers(
        &self,
        region: &str,
        cluster_arn: &str,
        task_arn: &str,
    ) -> Result<Vec<Container>> {
        let client = self.get_ecs_client(region);

        let describe_resp = client
            .describe_tasks()
            .cluster(cluster_arn)
            .tasks(task_arn)
            .send()
            .await?;

        let containers = describe_resp
            .tasks()
            .first()
            .map(|task| {
                task.containers()
                    .iter()
                    .filter_map(|c| {
                        Some(Container {
                            name: c.name()?.to_string(),
                            image: c.image().unwrap_or("N/A").to_string(),
                            status: c.last_status().unwrap_or("UNKNOWN").to_string(),
                            runtime_id: c.runtime_id().map(|s| s.to_string()),
                        })
                    })
                    .collect()
            })
            .unwrap_or_default();

        Ok(containers)
    }

    pub async fn force_new_deployment(
        &self,
        region: &str,
        cluster_arn: &str,
        service_name: &str,
    ) -> Result<()> {
        let client = self.get_ecs_client(region);

        client
            .update_service()
            .cluster(cluster_arn)
            .service(service_name)
            .force_new_deployment(true)
            .send()
            .await?;

        Ok(())
    }

    pub async fn list_ec2_instances(&self, region: &str) -> Result<Vec<Ec2Instance>> {
        let client = self.get_ec2_client(region);

        let resp = client
            .describe_instances()
            .send()
            .await?;

        let mut instances = Vec::new();

        for reservation in resp.reservations().iter() {
            for instance in reservation.instances().iter() {
                let instance_id = instance.instance_id().unwrap_or("N/A").to_string();

                // Get name from tags
                let name = instance
                    .tags()
                    .iter()
                    .find(|tag| tag.key() == Some("Name"))
                    .and_then(|tag| tag.value())
                    .unwrap_or(&instance_id)
                    .to_string();

                let instance_type = instance
                    .instance_type()
                    .map(|t| t.as_str().to_string())
                    .unwrap_or("N/A".to_string());

                let state = instance
                    .state()
                    .and_then(|s| s.name())
                    .map(|n| n.as_str().to_string())
                    .unwrap_or("UNKNOWN".to_string());

                let public_ip = instance.public_ip_address().map(|s| s.to_string());
                let private_ip = instance.private_ip_address().map(|s| s.to_string());

                let availability_zone = instance
                    .placement()
                    .and_then(|p| p.availability_zone())
                    .unwrap_or("N/A")
                    .to_string();

                let key_name = instance.key_name().map(|s| s.to_string());

                let iam_instance_profile = instance
                    .iam_instance_profile()
                    .and_then(|p| p.arn())
                    .map(|s| s.to_string());

                instances.push(Ec2Instance {
                    instance_id: instance_id.clone(),
                    name,
                    instance_type,
                    state,
                    public_ip,
                    private_ip,
                    availability_zone,
                    key_name,
                    iam_instance_profile,
                    ssm_managed: false, // Will be checked separately
                });
            }
        }

        // Check SSM availability for all instances
        if !instances.is_empty() {
            let ssm_client = self.get_ssm_client(region);

            // Check which instances are managed by SSM
            if let Ok(resp) = ssm_client
                .describe_instance_information()
                .send()
                .await
            {
                let managed_instance_ids: std::collections::HashSet<String> = resp
                    .instance_information_list()
                    .iter()
                    .filter_map(|info| info.instance_id())
                    .map(|id| id.to_string())
                    .collect();

                for instance in &mut instances {
                    instance.ssm_managed = managed_instance_ids.contains(&instance.instance_id);
                }
            }
        }

        Ok(instances)
    }

    pub async fn list_rds_clusters(&self, region: &str) -> Result<Vec<RdsCluster>> {
        let client = self.get_rds_client(region);

        let resp = client
            .describe_db_clusters()
            .send()
            .await?;

        let clusters = resp
            .db_clusters()
            .iter()
            .filter_map(|c| {
                Some(RdsCluster {
                    arn: c.db_cluster_arn()?.to_string(),
                    identifier: c.db_cluster_identifier()?.to_string(),
                    engine: c.engine().unwrap_or("unknown").to_string(),
                    engine_version: c.engine_version().unwrap_or("unknown").to_string(),
                    status: c.status().unwrap_or("unknown").to_string(),
                    endpoint: c.endpoint().map(|s| s.to_string()),
                    reader_endpoint: c.reader_endpoint().map(|s| s.to_string()),
                    port: c.port().unwrap_or(0),
                    master_username: c.master_username().unwrap_or("N/A").to_string(),
                    database_name: c.database_name().map(|s| s.to_string()),
                    multi_az: c.multi_az().unwrap_or(false),
                    storage_encrypted: c.storage_encrypted().unwrap_or(false),
                })
            })
            .collect();

        Ok(clusters)
    }

    pub async fn list_rds_instances(&self, region: &str) -> Result<Vec<RdsInstance>> {
        let client = self.get_rds_client(region);

        let resp = client
            .describe_db_instances()
            .send()
            .await?;

        let instances = resp
            .db_instances()
            .iter()
            .filter_map(|i| {
                Some(RdsInstance {
                    arn: i.db_instance_arn()?.to_string(),
                    identifier: i.db_instance_identifier()?.to_string(),
                    cluster_identifier: i.db_cluster_identifier().map(|s| s.to_string()),
                    engine: i.engine().unwrap_or("unknown").to_string(),
                    engine_version: i.engine_version().unwrap_or("unknown").to_string(),
                    instance_class: i.db_instance_class().unwrap_or("unknown").to_string(),
                    status: i.db_instance_status().unwrap_or("unknown").to_string(),
                    endpoint: i.endpoint().and_then(|e| e.address()).map(|s| s.to_string()),
                    port: i.endpoint().and_then(|e| e.port()).unwrap_or(0),
                    availability_zone: i.availability_zone().unwrap_or("N/A").to_string(),
                    multi_az: i.multi_az().unwrap_or(false),
                    storage_type: i.storage_type().unwrap_or("N/A").to_string(),
                    allocated_storage: i.allocated_storage().unwrap_or(0),
                })
            })
            .collect();

        Ok(instances)
    }

    pub async fn list_rds_instances_for_cluster(
        &self,
        region: &str,
        cluster_identifier: &str,
    ) -> Result<Vec<RdsInstance>> {
        let client = self.get_rds_client(region);

        let resp = client
            .describe_db_instances()
            .send()
            .await?;

        let instances = resp
            .db_instances()
            .iter()
            .filter(|i| {
                i.db_cluster_identifier()
                    .map(|id| id == cluster_identifier)
                    .unwrap_or(false)
            })
            .filter_map(|i| {
                Some(RdsInstance {
                    arn: i.db_instance_arn()?.to_string(),
                    identifier: i.db_instance_identifier()?.to_string(),
                    cluster_identifier: i.db_cluster_identifier().map(|s| s.to_string()),
                    engine: i.engine().unwrap_or("unknown").to_string(),
                    engine_version: i.engine_version().unwrap_or("unknown").to_string(),
                    instance_class: i.db_instance_class().unwrap_or("unknown").to_string(),
                    status: i.db_instance_status().unwrap_or("unknown").to_string(),
                    endpoint: i.endpoint().and_then(|e| e.address()).map(|s| s.to_string()),
                    port: i.endpoint().and_then(|e| e.port()).unwrap_or(0),
                    availability_zone: i.availability_zone().unwrap_or("N/A").to_string(),
                    multi_az: i.multi_az().unwrap_or(false),
                    storage_type: i.storage_type().unwrap_or("N/A").to_string(),
                    allocated_storage: i.allocated_storage().unwrap_or(0),
                })
            })
            .collect();

        Ok(instances)
    }
}
