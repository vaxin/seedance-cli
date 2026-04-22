use anyhow::Result;
use clap::Args;
use console::style;

use crate::store::TaskStore;

#[derive(Debug, Args)]
pub struct ListArgs {
    /// Number of tasks to show
    #[arg(short = 'n', long, default_value_t = 20)]
    pub limit: usize,

    /// Filter by status
    #[arg(long)]
    pub status: Option<String>,

    /// JSON output
    #[arg(long, default_value_t = false)]
    pub json: bool,
}

pub async fn execute(args: ListArgs) -> Result<()> {
    let store = TaskStore::open()?;
    let records = store.list(args.limit, args.status.as_deref())?;

    if args.json {
        let items: Vec<serde_json::Value> = records
            .iter()
            .map(|r| {
                serde_json::json!({
                    "task_id": r.task_id,
                    "prompt": r.prompt,
                    "model": r.model,
                    "status": r.status,
                    "video_url": r.video_url,
                    "output_path": r.output_path,
                    "created_at": r.created_at,
                    "updated_at": r.updated_at,
                })
            })
            .collect();
        println!("{}", serde_json::to_string_pretty(&items)?);
        return Ok(());
    }

    if records.is_empty() {
        eprintln!("No tasks found.");
        return Ok(());
    }

    eprintln!(
        "{:<30} {:<12} {:<20} {}",
        style("TASK ID").bold(),
        style("STATUS").bold(),
        style("CREATED").bold(),
        style("PROMPT").bold(),
    );
    eprintln!("{}", "-".repeat(90));

    for r in &records {
        let status_styled = match r.status.as_str() {
            "succeeded" | "completed" => style(&r.status).green(),
            "failed" => style(&r.status).red(),
            "running" | "in_progress" => style(&r.status).yellow(),
            _ => style(&r.status).dim(),
        };

        let prompt_short: String = r.prompt.chars().take(30).collect();
        let prompt_display = if r.prompt.len() > 30 {
            format!("{prompt_short}...")
        } else {
            prompt_short
        };

        eprintln!(
            "{:<30} {:<12} {:<20} {}",
            r.task_id, status_styled, r.created_at, prompt_display,
        );
    }

    Ok(())
}
