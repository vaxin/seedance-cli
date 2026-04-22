use anyhow::Result;
use std::time::{Duration, Instant};

use crate::client::types::{TaskResponse, TaskStatus};
use crate::client::ArkClient;
use crate::ui;

pub struct PollOptions {
    pub timeout: Duration,
    pub initial_interval: Duration,
    #[allow(dead_code)]
    pub strict: bool,
}

impl Default for PollOptions {
    fn default() -> Self {
        Self {
            timeout: Duration::from_secs(300),
            initial_interval: Duration::from_secs(10),
            strict: false,
        }
    }
}

pub enum PollResult {
    Completed(TaskResponse),
    TimedOut {
        task_id: String,
        #[allow(dead_code)]
        last_status: TaskStatus,
        elapsed: Duration,
    },
}

pub async fn poll_until_done(
    client: &ArkClient,
    task_id: &str,
    opts: &PollOptions,
    quiet: bool,
) -> Result<PollResult> {
    let start = Instant::now();
    let mut interval = opts.initial_interval;
    let max_interval = Duration::from_secs(60);

    let spinner = if !quiet {
        Some(ui::create_poll_spinner(task_id))
    } else {
        None
    };

    loop {
        let elapsed = start.elapsed();
        if opts.timeout.as_secs() > 0 && elapsed >= opts.timeout {
            if let Some(sp) = &spinner {
                sp.finish_and_clear();
            }
            return Ok(PollResult::TimedOut {
                task_id: task_id.to_string(),
                last_status: TaskStatus::Running,
                elapsed,
            });
        }

        if let Some(sp) = &spinner {
            sp.set_message(format!(
                "polling {} ({:.0}s elapsed)...",
                task_id,
                elapsed.as_secs_f64()
            ));
        }

        match client.query_task(task_id).await {
            Ok(resp) => {
                let status = resp.task_status();

                if let Some(sp) = &spinner {
                    sp.set_message(format!(
                        "{} — status: {} ({:.0}s)",
                        task_id,
                        status,
                        elapsed.as_secs_f64()
                    ));
                }

                if status.is_terminal() {
                    if let Some(sp) = &spinner {
                        sp.finish_and_clear();
                    }
                    return Ok(PollResult::Completed(resp));
                }
            }
            Err(e) => {
                if let Some(sp) = &spinner {
                    sp.set_message(format!("poll error: {e}, retrying..."));
                }
            }
        }

        tokio::time::sleep(interval).await;
        interval = (interval * 2).min(max_interval);
    }
}
