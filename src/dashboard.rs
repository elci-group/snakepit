use crate::resolver::DependencyResolver;
use crate::dependency::ProjectDependencies;
use serde::Serialize;
use anyhow::Result;
use std::collections::{HashSet, VecDeque};
use axum::{
    routing::get,
    response::{Html, Json},
    Router,
};
use std::net::SocketAddr;

#[derive(Serialize, Clone)]
pub struct Node {
    pub id: String,
    pub label: String,
    pub version: String,
    pub r#type: String, // "root", "direct", "transitive"
    pub description: Option<String>,
}

#[derive(Serialize, Clone)]
pub struct Edge {
    pub from: String,
    pub to: String,
}

#[derive(Serialize, Clone)]
pub struct GraphData {
    pub project_name: String,
    pub nodes: Vec<Node>,
    pub edges: Vec<Edge>,
}

pub struct DashboardServer {
    project_name: String,
    project_deps: ProjectDependencies,
}

impl DashboardServer {
    pub fn new(project_name: String, project_deps: ProjectDependencies) -> Self {
        Self { project_name, project_deps }
    }

    pub async fn start(self, port: u16) -> Result<()> {
        let addr = SocketAddr::from(([127, 0, 0, 1], port));
        println!("🚀 Snakepit Dashboard launching on http://{}", addr);

        let project_name_capture = self.project_name.clone();
        let project_deps_capture = self.project_deps.clone();

        let app = Router::new()
            .route("/", get(|| async {
                Html(include_str!("dashboard.html"))
            }))
            .route("/api/graph", get(move || {
                let p_name = project_name_capture.clone();
                let p_deps = project_deps_capture.clone();
                async move {
                    match generate_graph_data(&p_name, &p_deps).await {
                        Ok(data) => Json(data),
                        Err(_) => Json(GraphData {
                            project_name: p_name,
                            nodes: vec![],
                            edges: vec![],
                        }),
                    }
                }
            }));

        let listener = tokio::net::TcpListener::bind(addr).await?;
        axum::serve(listener, app).await?;
        
        Ok(())
    }
}

async fn generate_graph_data(project_name: &str, project_deps: &ProjectDependencies) -> Result<GraphData> {
    let mut resolver = DependencyResolver::new();
    let resolved = resolver.resolve_dependencies(project_deps).await?;

    let mut nodes = Vec::new();
    let mut edges = Vec::new();
    let mut visited = HashSet::new();

    // Add root node
    nodes.push(Node {
        id: "root".to_string(),
        label: project_name.to_string(),
        version: "0.1.0".to_string(),
        r#type: "root".to_string(),
        description: Some("Current project root".to_string()),
    });

    let mut queue = VecDeque::new();

    // Add direct dependencies
    for dep in &resolved.dependencies {
        nodes.push(Node {
            id: dep.name.clone(),
            label: dep.name.clone(),
            version: dep.version.clone(),
            r#type: "direct".to_string(),
            description: None,
        });
        edges.push(Edge { from: "root".to_string(), to: dep.name.clone() });
        queue.push_back(dep.clone());
        visited.insert(dep.name.clone());
    }

    while let Some(current) = queue.pop_front() {
        for sub_dep in &current.dependencies {
            if !visited.contains(&sub_dep.name) {
                nodes.push(Node {
                    id: sub_dep.name.clone(),
                    label: sub_dep.name.clone(),
                    version: sub_dep.version.clone(),
                    r#type: "transitive".to_string(),
                    description: None,
                });
                visited.insert(sub_dep.name.clone());
                queue.push_back(sub_dep.clone());
            }
            edges.push(Edge { from: current.name.clone(), to: sub_dep.name.clone() });
        }
    }

    Ok(GraphData {
        project_name: project_name.to_string(),
        nodes,
        edges,
    })
}
