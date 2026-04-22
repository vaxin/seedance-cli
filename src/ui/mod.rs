use console::style;
use indicatif::{ProgressBar, ProgressStyle};

pub fn create_poll_spinner(task_id: &str) -> ProgressBar {
    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.cyan} {msg}")
            .unwrap(),
    );
    pb.set_message(format!("waiting for {task_id}..."));
    pb.enable_steady_tick(std::time::Duration::from_millis(120));
    pb
}

pub fn print_task_created(task_id: &str) {
    eprintln!(
        "{} Task created: {}",
        style("✓").green().bold(),
        style(task_id).cyan()
    );
}

pub fn print_task_status(task_id: &str, status: &str) {
    let styled_status = match status {
        "succeeded" | "completed" => style(status).green().bold(),
        "failed" => style(status).red().bold(),
        "running" | "in_progress" => style(status).yellow(),
        _ => style(status).dim(),
    };
    eprintln!("{}: {}", style(task_id).cyan(), styled_status);
}

pub fn print_timeout_hint(task_id: &str, elapsed_secs: u64) {
    eprintln!();
    eprintln!(
        "{} Timed out after {}s, but task {} is still running on server.",
        style("⏳").yellow(),
        elapsed_secs,
        style(task_id).cyan()
    );
    eprintln!();
    eprintln!("Continue tracking:");
    eprintln!(
        "  seedance status {}            # check current status",
        task_id
    );
    eprintln!(
        "  seedance status {} --wait     # resume waiting",
        task_id
    );
    eprintln!(
        "  seedance download {}          # download when done",
        task_id
    );
}

pub fn print_downloaded(path: &str) {
    eprintln!(
        "{} Downloaded: {}",
        style("✓").green().bold(),
        style(path).cyan()
    );
}

pub fn print_error(msg: &str) {
    eprintln!("{} {}", style("✗").red().bold(), msg);
}
