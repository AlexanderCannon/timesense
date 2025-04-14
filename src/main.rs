use chrono::{DateTime, Duration, Local};
use device_query::{DeviceQuery, DeviceState};
use screenshots::Screen;
use serde::{Deserialize, Serialize};
use serde_json;
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::thread;
use std::time;

mod report_generator;
mod screenshot_analyzer;
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

    // Load config or create default
    let config = load_config().unwrap_or_else(|| {
        let default_config = Config {
            screenshot_interval_seconds: 60,
            idle_threshold_seconds: 180,
            productive_apps: vec![
                "code".to_string(),
                "terminal".to_string(),
                "notion".to_string(),
            ],
            distraction_apps: vec![
                "twitter".to_string(),
                "youtube".to_string(),
                "reddit".to_string(),
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

    // Initialize screenshot analyzer
    let mut screenshot_analyzer =
        ScreenshotAnalyzer::new().expect("Failed to initialize screenshot analyzer");

    let mut time_blocks: Vec<TimeBlock> = Vec::new();
    let mut current_block: Option<TimeBlock> = None;
    let device_state = DeviceState::new();
    let mut last_input_time = Local::now();

    loop {
        let now = Local::now();

        // Check for user activity
        let keys_pressed = device_state.get_keys();
        let is_active = !keys_pressed.is_empty();

        if is_active {
            last_input_time = now;
        }

        let is_idle = now.signed_duration_since(last_input_time).num_seconds()
            > config.idle_threshold_seconds as i64;

        // Take a screenshot of the primary display
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

                                    // Analyze the screenshot to determine application
                                    let app_name =
                                        screenshot_analyzer.analyze_screenshot(&screenshot_path);
                                    println!("Detected application: {}", app_name);

                                    let activity_type = categorize_activity(&app_name, &config);
                                    println!("Activity type: {}", activity_type);

                                    // Update time blocks
                                    match &current_block {
                                        Some(block) => {
                                            if block.application != app_name
                                                || block.idle != is_idle
                                            {
                                                // Finish current block
                                                let mut finished_block =
                                                    current_block.take().unwrap();
                                                finished_block.end_time = now;
                                                time_blocks.push(finished_block);

                                                // Start new block
                                                let app_name_clone = app_name.clone();
                                                let activity_type_clone = activity_type.clone();
                                                current_block = Some(TimeBlock {
                                                    start_time: now,
                                                    end_time: now, // Will be updated later
                                                    application: app_name,
                                                    activity_type,
                                                    idle: is_idle,
                                                });

                                                println!(
                                                    "New time block started: {} ({})",
                                                    app_name_clone, activity_type_clone
                                                );
                                            }
                                        }
                                        None => {
                                            // Start first block
                                            let app_name_clone = app_name.clone();
                                            let activity_type_clone = activity_type.clone();
                                            current_block = Some(TimeBlock {
                                                start_time: now,
                                                end_time: now, // Will be updated later
                                                application: app_name,
                                                activity_type,
                                                idle: is_idle,
                                            });

                                            println!(
                                                "First time block started: {} ({})",
                                                app_name_clone, activity_type_clone
                                            );
                                        }
                                    }

                                    // We're keeping the screenshots for audit purposes, so we don't delete them
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
