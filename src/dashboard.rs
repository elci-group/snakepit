use crate::dependency::ProjectDependencies;
use crate::resolver::DependencyResolver;
use anyhow::Result;
use serde::Serialize;
use std::collections::{HashSet, VecDeque};
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
        Self {
            project_name,
            project_deps,
        }
    }

    pub async fn start(self, port: u16) -> Result<()> {
        let addr = SocketAddr::from(([127, 0, 0, 1], port));
        println!("🚀 Snakepit Dashboard launching on http://{}", addr);

        let listener = tokio::net::TcpListener::bind(addr).await?;
        let project_name = self.project_name;
        let project_deps = self.project_deps;

        loop {
            let (mut stream, _) = listener.accept().await?;
            let project_name = project_name.clone();
            let project_deps = project_deps.clone();
            tokio::spawn(async move {
                if let Err(e) = handle_connection(&mut stream, &project_name, &project_deps).await {
                    eprintln!("Dashboard connection error: {}", e);
                }
            });
        }
    }
}

/// Minimal HTTP/1.1 handler for the two dashboard routes (`/` and `/api/graph`).
/// Reads a single request head — these routes take no request body — and closes
/// the connection after each response, so no keep-alive bookkeeping is needed.
async fn handle_connection(
    stream: &mut tokio::net::TcpStream,
    project_name: &str,
    project_deps: &ProjectDependencies,
) -> Result<()> {
    use tokio::io::{AsyncReadExt, AsyncWriteExt};

    let mut buf = [0u8; 8192];
    let n = stream.read(&mut buf).await?;
    let request = String::from_utf8_lossy(&buf[..n]);
    let request_line = request.lines().next().unwrap_or_default();
    let mut parts = request_line.split_whitespace();
    let method = parts.next().unwrap_or_default();
    let path = parts
        .next()
        .unwrap_or_default()
        .split('?')
        .next()
        .unwrap_or_default();

    let (status, content_type, body) = match (method, path) {
        ("GET", "/") => (
            "200 OK",
            "text/html; charset=utf-8",
            include_str!("dashboard.html").to_string(),
        ),
        ("GET", "/api/graph") => {
            let data = generate_graph_data(project_name, project_deps)
                .await
                .unwrap_or_else(|_| GraphData {
                    project_name: project_name.to_string(),
                    nodes: vec![],
                    edges: vec![],
                });
            ("200 OK", "application/json", serde_json::to_string(&data)?)
        }
        _ => (
            "404 Not Found",
            "text/plain; charset=utf-8",
            "not found".to_string(),
        ),
    };

    let response = format!(
        "HTTP/1.1 {status}\r\nContent-Type: {content_type}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
        body.len()
    );
    stream.write_all(response.as_bytes()).await?;
    stream.shutdown().await?;
    Ok(())
}

async fn generate_graph_data(
    project_name: &str,
    project_deps: &ProjectDependencies,
) -> Result<GraphData> {
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
        edges.push(Edge {
            from: "root".to_string(),
            to: dep.name.clone(),
        });
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
            edges.push(Edge {
                from: current.name.clone(),
                to: sub_dep.name.clone(),
            });
        }
    }

    Ok(GraphData {
        project_name: project_name.to_string(),
        nodes,
        edges,
    })
}
