use chrono::{DateTime, Duration, Local};
use device_query::{DeviceQuery, DeviceState};
use screenshots::Screen;
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::thread;
use std::time;
use tesseract::Tesseract;
use serde::{Deserialize, Serialize};
use serde_json;
use rand::Rng;

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
            productive_apps: vec!["code".to_string(), "terminal".to_string(), "notion".to_string()],
            distraction_apps: vec!["twitter".to_string(), "youtube".to_string(), "reddit".to_string()],
            data_directory: "timesense_data".to_string(),
        };
        
        // Save default config
        save_config(&default_config).expect("Failed to save default config");
        
        default_config
    });
    
    // Create data directory if it doesn't exist
    fs::create_dir_all(&config.data_directory).expect("Failed to create data directory");
    
    let mut time_blocks: Vec<TimeBlock> = Vec::new();
    let mut current_block: Option<TimeBlock> = None;
    let device_state = DeviceState::new();
    let mut last_input_time = Local::now();
    
    loop {
        let now = Local::now();
        let screens = Screen::all().unwrap();
        
        // Check for user activity
        let keys_pressed = device_state.get_keys();
        let is_active = !keys_pressed.is_empty();
        
        if is_active {
            last_input_time = now;
        }
        
        let is_idle = now.signed_duration_since(last_input_time).num_seconds() > 
            config.idle_threshold_seconds as i64;
        
        // Take a screenshot of the primary display
        if let Some(screen) = screens.first() {
            let image = screen.capture().unwrap();
            let temp_path = Path::new("temp_screenshot.png");
            image.save(temp_path).unwrap();
            
            // Analyze the screenshot to determine application
            let app_name = analyze_screenshot(temp_path);
            let activity_type = categorize_activity(&app_name, &config);
            
            // Update time blocks
            match &current_block {
                Some(block) => {
                    if block.application != app_name || block.idle != is_idle {
                        // Finish current block
                        let mut finished_block = current_block.take().unwrap();
                        finished_block.end_time = now;
                        time_blocks.push(finished_block);
                        
                        // Start new block
                        current_block = Some(TimeBlock {
                            start_time: now,
                            end_time: now, // Will be updated later
                            application: app_name,
                            activity_type,
                            idle: is_idle,
                        });
                    }
                },
                None => {
                    // Start first block
                    current_block = Some(TimeBlock {
                        start_time: now,
                        end_time: now, // Will be updated later
                        application: app_name,
                        activity_type,
                        idle: is_idle,
                    });
                }
            }
            
            // Clean up temporary screenshot
            let _ = fs::remove_file(temp_path);
        }
        
        // Generate daily summary if it's a new day
        if !time_blocks.is_empty() && 
           time_blocks.last().unwrap().end_time.date_naive() != now.date_naive() {
            generate_daily_summary(&time_blocks, &config);
            time_blocks.clear();
        }
        
        // Sleep until next interval
        thread::sleep(time::Duration::from_secs(config.screenshot_interval_seconds));
    }
}

fn analyze_screenshot(_path: &Path) -> String {
    // Initialize Tesseract OCR
    let _tess = Tesseract::new(None, Some("eng")).unwrap();
    
    // In a real implementation, we'd use window manager APIs,
    // but for simplicity we'll mock recognizing apps from window titles
    // This would use OCR to read window titles from screenshots
    
    // Mock implementation
    let sample_apps = vec!["Visual Studio Code", "Terminal", "Notion", "Twitter", "YouTube", "Reddit"];
    let mut rng = rand::thread_rng();
    sample_apps[rng.gen_range(0..sample_apps.len())].to_string()
}

fn categorize_activity(app_name: &str, config: &Config) -> String {
    let lower_app = app_name.to_lowercase();
    
    if config.productive_apps.iter().any(|app| lower_app.contains(app)) {
        "productive".to_string()
    } else if config.distraction_apps.iter().any(|app| lower_app.contains(app)) {
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
        let app_duration = app_breakdown.entry(block.application.clone()).or_insert(Duration::zero());
        *app_duration = *app_duration + duration;
        
        // Update activity breakdown
        let activity_duration = activity_breakdown.entry(block.activity_type.clone()).or_insert(Duration::zero());
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
    
    let date = time_blocks.first().unwrap().start_time.date_naive().format("%Y-%m-%d").to_string();
    
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
    
    // Generate report
    generate_report(&summary);
    
    summary
}

fn generate_report(summary: &DailySummary) {
    // Create a user-friendly HTML report
    let productive_hours = summary.productive_time.num_minutes() as f64 / 60.0;
    let distracted_hours = summary.distracted_time.num_minutes() as f64 / 60.0;
    let idle_hours = summary.idle_time.num_minutes() as f64 / 60.0;
    
    let html = format!(
        r#"<!DOCTYPE html>
<html>
<head>
    <title>TimeSense Daily Report - {}</title>
    <style>
        body {{ font-family: Arial, sans-serif; margin: 20px; }}
        .summary {{ margin-bottom: 20px; }}
        .chart {{ width: 100%; background-color: #f0f0f0; height: 30px; margin-bottom: 15px; }}
        .productive {{ background-color: #4CAF50; height: 100%; float: left; }}
        .distracted {{ background-color: #F44336; height: 100%; float: left; }}
        .idle {{ background-color: #9E9E9E; height: 100%; float: left; }}
        table {{ border-collapse: collapse; width: 100%; }}
        th, td {{ text-align: left; padding: 8px; border-bottom: 1px solid #ddd; }}
    </style>
</head>
<body>
    <h1>TimeSense Daily Report</h1>
    <h2>Date: {}</h2>
    
    <div class="summary">
        <h3>Time Distribution</h3>
        <div class="chart">
            <div class="productive" style="width: {}%"></div>
            <div class="distracted" style="width: {}%"></div>
            <div class="idle" style="width: {}%"></div>
        </div>
        <p>Productive: {:.2} hours</p>
        <p>Distracted: {:.2} hours</p>
        <p>Idle: {:.2} hours</p>
    </div>
    
    <h3>Application Breakdown</h3>
    <table>
        <tr>
            <th>Application</th>
            <th>Hours</th>
        </tr>
        {}
    </table>
</body>
</html>"#,
        summary.date,
        summary.date,
        productive_hours / (productive_hours + distracted_hours + idle_hours) * 100.0,
        distracted_hours / (productive_hours + distracted_hours + idle_hours) * 100.0,
        idle_hours / (productive_hours + distracted_hours + idle_hours) * 100.0,
        productive_hours,
        distracted_hours,
        idle_hours,
        summary.application_breakdown.iter()
            .map(|(app, duration)| {
                format!(
                    "<tr><td>{}</td><td>{:.2}</td></tr>",
                    app,
                    duration.num_minutes() as f64 / 60.0
                )
            })
            .collect::<Vec<String>>()
            .join("")
    );
    
    let filename = format!("{}/report_{}.html", "timesense_data", summary.date);
    fs::write(filename, html).expect("Failed to write HTML report");
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