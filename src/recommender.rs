use snakegg::charmer::SnakeCharmer;
use anyhow::Result;
use snakegg::native::style::{red, green, yellow, blue, cyan, bold, dim};

#[derive(Debug, Clone)]
pub struct PackageRecommendation {
    pub name: String,
    pub rating: u8,
    pub use_case: String,
    pub pros: Vec<String>,
    pub cons: Vec<String>,
}

pub struct PackageRecommender {
    charmer: SnakeCharmer,
}

impl PackageRecommender {
    pub fn new() -> Result<Self> {
        Ok(Self {
            charmer: SnakeCharmer::new()?,
        })
    }

    pub async fn recommend(&self, query: &str, context: Option<&str>) -> Result<Vec<PackageRecommendation>> {
        let prompt = self.build_recommendation_prompt(query, context);
        let response = self.charmer.ask(&prompt).await?;
        self.parse_recommendations(&response)
    }

    fn build_recommendation_prompt(&self, query: &str, context: Option<&str>) -> String {
        let context_str = context.map_or(String::new(), |c| format!("\nProject context: {}", c));
        
        format!(
            r#"You are a Python package expert. A user wants to: {}{}

Recommend the top 3 Python packages for this task.
For each package, provide:
1. Package name (exact PyPI name)
2. Star rating (1-5, where 5 is best)
3. Best use case (one sentence)
4. Pros (2-3 key advantages, comma-separated)
5. Cons (1-2 limitations, comma-separated)

Format your response EXACTLY as follows:
PACKAGE: <name>
RATING: <1-5>
USE_CASE: <description>
PROS: <advantage 1>, <advantage 2>, <advantage 3>
CONS: <limitation 1>, <limitation 2>
---
(repeat for each of the 3 packages)

Be concise and specific. Focus on practical differences between packages."#,
            query, context_str
        )
    }

    fn parse_recommendations(&self, response: &str) -> Result<Vec<PackageRecommendation>> {
        let mut recommendations = Vec::new();
        let sections: Vec<&str> = response.split("---").collect();

        for section in sections {
            let section = section.trim();
            if section.is_empty() {
                continue;
            }

            let mut name = None;
            let mut rating = None;
            let mut use_case = None;
            let mut pros = Vec::new();
            let mut cons = Vec::new();

            for line in section.lines() {
                let line = line.trim();
                if line.starts_with("PACKAGE:") {
                    name = Some(line.replace("PACKAGE:", "").trim().to_string());
                } else if line.starts_with("RATING:") {
                    if let Ok(r) = line.replace("RATING:", "").trim().parse::<u8>() {
                        rating = Some(r.min(5));
                    }
                } else if line.starts_with("USE_CASE:") {
                    use_case = Some(line.replace("USE_CASE:", "").trim().to_string());
                } else if line.starts_with("PROS:") {
                    let pros_str = line.replace("PROS:", "").trim().to_string();
                    pros = pros_str.split(',').map(|s| s.trim().to_string()).collect();
                } else if line.starts_with("CONS:") {
                    let cons_str = line.replace("CONS:", "").trim().to_string();
                    cons = cons_str.split(',').map(|s| s.trim().to_string()).collect();
                }
            }

            if let (Some(n), Some(r), Some(u)) = (name, rating, use_case) {
                recommendations.push(PackageRecommendation {
                    name: n,
                    rating: r,
                    use_case: u,
                    pros,
                    cons,
                });
            }
        }

        Ok(recommendations)
    }

    pub fn display_recommendations(&self, recommendations: &[PackageRecommendation]) {
        println!("
{}", cyan(bold("📦 Top Recommendations:")));
        
        for (i, rec) in recommendations.iter().enumerate() {
            println!("\n{}. {} {}", 
                bold(format!("{}", i + 1)),
                bold(green(&rec.name)),
                self.format_stars(rec.rating)
            );
            
            println!("   {}: {}", 
                dim("Best for"),
                rec.use_case
            );
            
            if !rec.pros.is_empty() {
                println!("   {}: {}", 
                    green("Pros"),
                    rec.pros.join(", ")
                );
            }
            
            if !rec.cons.is_empty() {
                println!("   {}: {}", 
                    yellow("Cons"),
                    rec.cons.join(", ")
                );
            }
            
            println!("   {}: {}", 
                dim("Install"),
                cyan(format!("snakepit install {}", rec.name))
            );
        }
    }

    fn format_stars(&self, rating: u8) -> String {
        let filled = "★".repeat(rating as usize);
        let empty = "☆".repeat((5 - rating) as usize);
        yellow(format!("({}{})", filled, empty)).to_string()
    }

    pub fn prompt_install(&self, recommendations: &[PackageRecommendation]) -> Result<Option<String>> {
        if recommendations.is_empty() {
            return Ok(None);
        }

        println!("\n{}", bold("💡 Based on your query, I recommend starting with:"));
        println!("   {}", bold(green(&recommendations[0].name)));
        
        println!("\n{}", bold("Install now? [Y/n/1-3]"));
        println!("   {} Install {}", green("Y"), recommendations[0].name);
        println!("   {} Skip", red("n"));
        if recommendations.len() > 1 {
            println!("   {} Install a specific package (1-{})", cyan("1-3"), recommendations.len().min(3));
        }
        
        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;
        let input = input.trim().to_lowercase();

        match input.as_str() {
            "" | "y" | "yes" => Ok(Some(recommendations[0].name.clone())),
            "n" | "no" => Ok(None),
            "1" => Ok(recommendations.get(0).map(|r| r.name.clone())),
            "2" => Ok(recommendations.get(1).map(|r| r.name.clone())),
            "3" => Ok(recommendations.get(2).map(|r| r.name.clone())),
            _ => Ok(None),
        }
    }
}
