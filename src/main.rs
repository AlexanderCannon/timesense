use chrono::{DateTime, Duration, Local};
use ctrlc;
use device_query::{DeviceQuery, DeviceState};
use screenshots::Screen;
use serde::{Deserialize, Serialize};
use serde_json;
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time;
use webbrowser;

mod app_detector;
mod report_generator;
mod screenshot_analyzer;
mod fuzzy_match;

use app_detector::{AppDetector, PlatformAppDetector};
use report_generator::ReportGenerator;
use screenshot_analyzer::ScreenshotAnalyzer;

#[derive(Debug, Serialize, Deserialize)]
struct TimeBlock {
    start_time: DateTime<Local>,
    end_time: DateTime<Local>,
    application: String,
    activity_type: String,
    idle: bool,
}

#[derive(Debug, Serialize, Deserialize)]
struct DailySummary {
    date: String,
    productive_time: Duration,
    distracted_time: Duration,
    idle_time: Duration,
    application_breakdown: HashMap<String, Duration>,
    activity_breakdown: HashMap<String, Duration>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Config {
    screenshot_interval_seconds: u64,
    idle_threshold_seconds: u64,
    productive_apps: Vec<String>,
    distraction_apps: Vec<String>,
    data_directory: String,
}

fn main() {
    println!("Starting TimeSense - Automated Time Awareness Tool");

    // Set up signal handler for graceful shutdown
    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();
    ctrlc::set_handler(move || {
        println!("\nReceived Ctrl+C.");
        println!("Generating report...");
        r.store(false, Ordering::SeqCst);
    })
    .expect("Error setting Ctrl+C handler");

    // Load config or create default
    let config = load_config().unwrap_or_else(|| {
        let default_config = Config {
            screenshot_interval_seconds: 60,
            idle_threshold_seconds: 180,
            productive_apps: vec![
                "code".to_string(),
                "terminal".to_string(),
                "notion".to_string(),
                "cursor".to_string(),
                "vscode".to_string(),
            ],
            distraction_apps: vec![
                "twitter".to_string(),
                "youtube".to_string(),
                "reddit".to_string(),
                "slack".to_string(),
                "discord".to_string(),
                "spotify".to_string(),
                "steam".to_string(),
                "twitch".to_string(),
                "telegram".to_string(),
                "whatsapp".to_string(),
                "zoom".to_string(),
                "skype".to_string(),
                "microsoft teams".to_string(),
                "google meet".to_string(),
            ],
            data_directory: "timesense_data".to_string(),
        };

        // Save default config
        save_config(&default_config).expect("Failed to save default config");

        default_config
    });

    // Create data directory if it doesn't exist
    fs::create_dir_all(&config.data_directory).expect("Failed to create data directory");

    // Create screenshots subdirectory
    let screenshots_dir = Path::new(&config.data_directory).join("screenshots");
    fs::create_dir_all(&screenshots_dir).expect("Failed to create screenshots directory");

    // Initialize screenshot analyzer (for fallback)
    let _screenshot_analyzer =
        ScreenshotAnalyzer::new().expect("Failed to initialize screenshot analyzer");

    // Initialize the app detector
    let app_detector = PlatformAppDetector::new();

    let mut time_blocks: Vec<TimeBlock> = Vec::new();
    let mut current_block: Option<TimeBlock> = None;
    let device_state = DeviceState::new();
    let mut last_input_time = Local::now();

    println!("TimeSense is running. Press Ctrl+C to stop and generate a report.");

    while running.load(Ordering::SeqCst) {
        let now = Local::now();

        // Check for user activity
        let keys_pressed = device_state.get_keys();
        let is_active = !keys_pressed.is_empty();

        if is_active {
            last_input_time = now;
        }

        let is_idle = now.signed_duration_since(last_input_time).num_seconds()
            > config.idle_threshold_seconds as i64;

        // Get the active application
        let active_app = app_detector.get_active_application();
        println!("Active application: {}", active_app);

        let activity_type = categorize_activity(&active_app, &config);
        println!("Activity type: {}", activity_type);

        // Take a screenshot for record-keeping
        match Screen::all() {
            Ok(screens) => {
                if let Some(screen) = screens.first() {
                    println!("Capturing screenshot...");
                    match screen.capture() {
                        Ok(image) => {
                            // Create a timestamped filename
                            let timestamp = now.format("%Y%m%d_%H%M%S").to_string();
                            let screenshot_path =
                                screenshots_dir.join(format!("screenshot_{}.png", timestamp));

                            println!("Saving screenshot to {:?}", screenshot_path);

                            match image.save(&screenshot_path) {
                                Ok(_) => {
                                    println!("Screenshot saved successfully");
                                }
                                Err(e) => println!("Failed to save screenshot: {}", e),
                            }
                        }
                        Err(e) => println!("Failed to capture screenshot: {}", e),
                    }
                } else {
                    println!("No screens found");
                }
            }
            Err(e) => println!("Failed to get screens: {}", e),
        }

        // Update time blocks
        match &current_block {
            Some(block) => {
                if block.application != active_app || block.idle != is_idle {
                    // Finish current block
                    let mut finished_block = current_block.take().unwrap();
                    finished_block.end_time = now;
                    time_blocks.push(finished_block);

                    // Start new block
                    current_block = Some(TimeBlock {
                        start_time: now,
                        end_time: now, // Will be updated later
                        application: active_app.clone(),
                        activity_type: activity_type.clone(),
                        idle: is_idle,
                    });

                    println!("New time block started: {} ({})", active_app, activity_type);
                }
            }
            None => {
                // Start first block
                current_block = Some(TimeBlock {
                    start_time: now,
                    end_time: now, // Will be updated later
                    application: active_app.clone(),
                    activity_type: activity_type.clone(),
                    idle: is_idle,
                });

                println!(
                    "First time block started: {} ({})",
                    active_app, activity_type
                );
            }
        }

        // Generate daily summary if it's a new day
        if !time_blocks.is_empty()
            && time_blocks.last().unwrap().end_time.date_naive() != now.date_naive()
        {
            generate_daily_summary(&time_blocks, &config);
            time_blocks.clear();
        }

        // Sleep until next interval
        thread::sleep(time::Duration::from_secs(
            config.screenshot_interval_seconds,
        ));
    }

    // Graceful shutdown
    println!("Performing graceful shutdown...");

    // Finalize the current time block if it exists
    if let Some(mut block) = current_block.take() {
        block.end_time = Local::now();
        time_blocks.push(block);
    }

    // Generate a report for the current day
    if !time_blocks.is_empty() {
        println!("Generating report for today's data...");
        let summary = generate_daily_summary(&time_blocks, &config);

        // Get the absolute path to the report file
        let report_path = Path::new(&config.data_directory)
            .join(format!("report_{}.html", summary.date))
            .canonicalize()
            .unwrap_or_else(|_| {
                Path::new(&config.data_directory).join(format!("report_{}.html", summary.date))
            });

        // Convert to file URL format
        let file_url = format!("file://{}", report_path.display());

        println!("\nReport generated successfully!");
        println!("To view your report, open this link in your browser:");
        println!("{}", file_url);
        println!("\nOr navigate to this file location:");
        println!("{}", report_path.display());

        // Ask if the user wants to open the report now
        println!("\nWould you like to open the report now? (y/n)");
        let mut input = String::new();
        std::io::stdin()
            .read_line(&mut input)
            .expect("Failed to read input");

        if input.trim().to_lowercase() == "y" || input.trim().to_lowercase() == "yes" {
            println!("Opening report in your default browser...");
            if let Err(e) = webbrowser::open(&file_url) {
                println!("Failed to open browser: {}", e);
                println!("Please open the report manually using the link or file path above.");
            }
        }
    }

    println!("TimeSense has been shut down gracefully.");
}

fn categorize_activity(app_name: &str, config: &Config) -> String {
    let lower_app = app_name.to_lowercase();

    if config
        .productive_apps
        .iter()
        .any(|app| lower_app.contains(app))
    {
        "productive".to_string()
    } else if config
        .distraction_apps
        .iter()
        .any(|app| lower_app.contains(app))
    {
        "distraction".to_string()
    } else {
        "neutral".to_string()
    }
}

fn generate_daily_summary(time_blocks: &[TimeBlock], config: &Config) -> DailySummary {
    let mut productive_time = Duration::zero();
    let mut distracted_time = Duration::zero();
    let mut idle_time = Duration::zero();
    let mut app_breakdown: HashMap<String, Duration> = HashMap::new();
    let mut activity_breakdown: HashMap<String, Duration> = HashMap::new();

    for block in time_blocks {
        let duration = block.end_time.signed_duration_since(block.start_time);

        // Update application breakdown
        let app_duration = app_breakdown
            .entry(block.application.clone())
            .or_insert(Duration::zero());
        *app_duration = *app_duration + duration;

        // Update activity breakdown
        let activity_duration = activity_breakdown
            .entry(block.activity_type.clone())
            .or_insert(Duration::zero());
        *activity_duration = *activity_duration + duration;

        // Update time categories
        if block.idle {
            idle_time = idle_time + duration;
        } else if block.activity_type == "productive" {
            productive_time = productive_time + duration;
        } else if block.activity_type == "distraction" {
            distracted_time = distracted_time + duration;
        }
    }

    let date = time_blocks
        .first()
        .unwrap()
        .start_time
        .date_naive()
        .format("%Y-%m-%d")
        .to_string();

    let summary = DailySummary {
        date,
        productive_time,
        distracted_time,
        idle_time,
        application_breakdown: app_breakdown,
        activity_breakdown,
    };

    // Save the summary to disk
    let filename = format!("{}/summary_{}.json", config.data_directory, summary.date);
    let json = serde_json::to_string_pretty(&summary).unwrap();
    fs::write(filename, json).expect("Failed to write daily summary");

    // Generate report using the ReportGenerator
    let report_generator = ReportGenerator::new(config.data_directory.clone());
    report_generator.generate_report(&summary);

    summary
}

fn load_config() -> Option<Config> {
    let config_path = "timesense_config.json";
    if Path::new(config_path).exists() {
        let contents = fs::read_to_string(config_path).ok()?;
        serde_json::from_str(&contents).ok()
    } else {
        None
    }
}

fn save_config(config: &Config) -> Result<(), Box<dyn std::error::Error>> {
    let json = serde_json::to_string_pretty(config)?;
    fs::write("timesense_config.json", json)?;
    Ok(())
}
