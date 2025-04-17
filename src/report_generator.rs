use std::fs;
use chrono::Duration as TimeDelta;
use std::collections::HashMap;
use crate::fuzzy_match::group_similar_apps;

pub struct ReportGenerator {
    data_directory: String,
}

impl ReportGenerator {
    pub fn new(data_directory: String) -> Self {
        ReportGenerator { data_directory }
    }

    pub fn generate_report(&self, summary: &super::DailySummary) {
        // Create a user-friendly HTML report
        let productive_minutes = summary.productive_time.num_minutes() as f64;
        let distracted_minutes = summary.distracted_time.num_minutes() as f64;
        let idle_minutes = summary.idle_time.num_minutes() as f64;

        let total_minutes = productive_minutes + distracted_minutes + idle_minutes;
        
        // Calculate percentages with proper handling of zero total time
        let (productive_percentage, distracted_percentage, idle_percentage) = if total_minutes > 0.0 {
            (
                (productive_minutes / total_minutes) * 100.0,
                (distracted_minutes / total_minutes) * 100.0,
                (idle_minutes / total_minutes) * 100.0
            )
        } else {
            (0.0, 0.0, 0.0)
        };

        // Calculate time distribution score (0-100) based on productive vs distracted time
        let time_distribution_score = if total_minutes > 0.0 {
            let active_time = productive_minutes + distracted_minutes;
            if active_time > 0.0 {
                (productive_minutes / active_time) * 100.0
            } else {
                100.0 // If no active time, assume focused
            }
        } else {
            100.0 // If no time tracked, assume focused
        };

        // Get time distribution rating
        let time_distribution_rating = self.get_time_distribution_rating(time_distribution_score);

        let html = format!(
            r#"<!DOCTYPE html>
<html>
<head>
    <title>TimeSense Daily Report - {}</title>
    <style>
        body {{ 
            font-family: 'Segoe UI', Arial, sans-serif; 
            margin: 0;
            padding: 0;
            background-color: #f5f5f5;
            color: #333;
        }}
        .container {{
            max-width: 1000px;
            margin: 0 auto;
            padding: 20px;
        }}
        header {{
            background-color: #2c3e50;
            color: white;
            padding: 20px;
            text-align: center;
            border-radius: 5px 5px 0 0;
        }}
        .content {{
            background-color: white;
            padding: 20px;
            border-radius: 0 0 5px 5px;
            box-shadow: 0 2px 5px rgba(0,0,0,0.1);
        }}
        .summary {{ 
            margin-bottom: 30px;
            padding: 20px;
            background-color: #f9f9f9;
            border-radius: 5px;
        }}
        .chart {{ 
            width: 100%; 
            background-color: #f0f0f0; 
            height: 30px; 
            margin-bottom: 15px;
            border-radius: 15px;
            overflow: hidden;
        }}
        .productive {{ background-color: #4CAF50; height: 100%; float: left; }}
        .distracted {{ background-color: #F44336; height: 100%; float: left; }}
        .idle {{ background-color: #9E9E9E; height: 100%; float: left; }}
        table {{ 
            border-collapse: collapse; 
            width: 100%;
            margin-bottom: 20px;
        }}
        th, td {{ 
            text-align: left; 
            padding: 12px 8px; 
            border-bottom: 1px solid #ddd; 
        }}
        th {{
            background-color: #f2f2f2;
            font-weight: bold;
        }}
        tr:hover {{
            background-color: #f5f5f5;
        }}
        .time-distribution-score {{
            font-size: 24px;
            font-weight: bold;
            text-align: center;
            margin: 20px 0;
            padding: 10px;
            border-radius: 5px;
        }}
        .high-distribution {{
            background-color: #dff0d8;
            color: #3c763d;
        }}
        .medium-distribution {{
            background-color: #fcf8e3;
            color: #8a6d3b;
        }}
        .low-distribution {{
            background-color: #f2dede;
            color: #a94442;
        }}
        .section {{
            margin-bottom: 30px;
        }}
        .section-title {{
            border-bottom: 2px solid #2c3e50;
            padding-bottom: 10px;
            margin-bottom: 15px;
        }}
        .stats-grid {{
            display: grid;
            grid-template-columns: repeat(3, 1fr);
            gap: 20px;
            margin-bottom: 20px;
        }}
        .stat-card {{
            background-color: #f9f9f9;
            padding: 15px;
            border-radius: 5px;
            text-align: center;
        }}
        .stat-value {{
            font-size: 24px;
            font-weight: bold;
            margin-bottom: 5px;
        }}
        .stat-label {{
            color: #666;
        }}
        .productive-value {{ color: #4CAF50; }}
        .distracted-value {{ color: #F44336; }}
        .idle-value {{ color: #9E9E9E; }}
    </style>
</head>
<body>
    <div class="container">
        <header>
            <h1>TimeSense Daily Report</h1>
            <h2>{}</h2>
        </header>
        
        <div class="content">
            <div class="time-distribution-score {}">
                Time Distribution Score: {:.1}% - {}
            </div>
            
            <div class="section">
                <h3 class="section-title">Time Distribution</h3>
                <div class="stats-grid">
                    <div class="stat-card">
                        <div class="stat-value productive-value">{:.0} minutes</div>
                        <div class="stat-label">Focused Time</div>
                    </div>
                    <div class="stat-card">
                        <div class="stat-value distracted-value">{:.0} minutes</div>
                        <div class="stat-label">Distracted Time</div>
                    </div>
                    <div class="stat-card">
                        <div class="stat-value idle-value">{:.0} minutes</div>
                        <div class="stat-label">Idle Time</div>
                    </div>
                </div>
                
                <div class="chart">
                    <div class="productive" style="width: {}%"></div>
                    <div class="distracted" style="width: {}%"></div>
                    <div class="idle" style="width: {}%"></div>
                </div>
            </div>
            
            <div class="section">
                <h3 class="section-title">Application Usage</h3>
                <table class="app-table">
                    <tr>
                        <th>Application</th>
                        <th>Minutes</th>
                        <th>Percentage</th>
                    </tr>
                    {}
                </table>
            </div>
            
            <div class="section">
                <h3 class="section-title">Activity Distribution</h3>
                <table>
                    <tr>
                        <th>Activity Type</th>
                        <th>Minutes</th>
                        <th>Percentage</th>
                    </tr>
                    {}
                </table>
            </div>
            
            <div class="section">
                <h3 class="section-title">Time Distribution Observations</h3>
                <p>{}</p>
            </div>
        </div>
    </div>
</body>
</html>"#,
            summary.date,
            summary.date,
            self.get_time_distribution_class(time_distribution_score),
            time_distribution_score,
            time_distribution_rating,
            productive_minutes,
            distracted_minutes,
            idle_minutes,
            productive_percentage,
            distracted_percentage,
            idle_percentage,
            self.generate_application_table(&summary.application_breakdown, total_minutes),
            self.generate_activity_table(&summary.activity_breakdown, total_minutes),
            self.generate_time_distribution_observations(summary)
        );

        let filename = format!("{}/report_{}.html", self.data_directory, summary.date);
        fs::write(filename, html).expect("Failed to write HTML report");
    }

    pub fn generate_application_table(&self, app_breakdown: &HashMap<String, TimeDelta>, total_minutes: f64) -> String {
        // Group similar app names together
        let grouped_apps = group_similar_apps(app_breakdown);
        
        // Convert to vec for sorting
        let mut app_vec: Vec<(&String, &(TimeDelta, Vec<(String, TimeDelta)>))> = grouped_apps.iter().collect();
        app_vec.sort_by(|a, b| b.1.0.cmp(&a.1.0));

        let mut rows = String::new();
        for (app_name, (duration, _)) in app_vec {
            let minutes = duration.num_minutes() as f64;
            let percentage = if total_minutes > 0.0 {
                (minutes / total_minutes) * 100.0
            } else {
                0.0
            };

            rows.push_str(&format!(
                "<tr><td>{}</td><td>{:.0}</td><td>{:.1}%</td></tr>",
                app_name,
                minutes,
                percentage
            ));
        }

        rows
    }
    
    fn generate_activity_table(&self, activity_breakdown: &HashMap<String, TimeDelta>, total_minutes: f64) -> String {
        // Sort activities by duration (descending)
        let mut sorted_activities: Vec<(&String, &TimeDelta)> = activity_breakdown.iter().collect();
        sorted_activities.sort_by(|a, b| b.1.cmp(a.1));
        
        sorted_activities
            .iter()
            .map(|(activity, duration)| {
                let minutes = duration.num_minutes() as f64;
                let percentage = if total_minutes > 0.0 {
                    (minutes / total_minutes) * 100.0
                } else {
                    0.0
                };
                format!(
                    "<tr><td>{}</td><td>{:.0}</td><td>{:.1}%</td></tr>",
                    activity,
                    minutes,
                    percentage
                )
            })
            .collect::<Vec<String>>()
            .join("")
    }
    
    fn get_time_distribution_rating(&self, score: f64) -> String {
        if score >= 80.0 {
            "High Focus".to_string()
        } else if score >= 60.0 {
            "Moderate Focus".to_string()
        } else if score >= 40.0 {
            "Balanced".to_string()
        } else if score > 0.0 {
            "Distracted".to_string()
        } else {
            "High Focus".to_string() // Default to High Focus when no activity
        }
    }
    
    fn get_time_distribution_class(&self, score: f64) -> String {
        if score >= 60.0 {
            "high-distribution".to_string()
        } else if score >= 40.0 {
            "medium-distribution".to_string()
        } else {
            "low-distribution".to_string()
        }
    }
    
    fn generate_time_distribution_observations(&self, summary: &super::DailySummary) -> String {
        let productive_minutes = summary.productive_time.num_minutes() as f64;
        let distracted_minutes = summary.distracted_time.num_minutes() as f64;
        let idle_minutes = summary.idle_time.num_minutes() as f64;
        let total_minutes = productive_minutes + distracted_minutes + idle_minutes;
        
        let mut observations = String::new();
        
        if total_minutes == 0.0 {
            return "No activity tracked during this session.".to_string();
        }
        
        // Time distribution observations
        if productive_minutes > 0.0 {
            observations.push_str(&format!(
                "You spent {:.0} minutes on focused activities, which is {:.1}% of your tracked time. ",
                productive_minutes,
                (productive_minutes / total_minutes) * 100.0
            ));
        }
        
        // Distraction observations
        if distracted_minutes > 0.0 {
            observations.push_str(&format!(
                "You spent {:.0} minutes on distracting activities ({:.1}% of your time). ",
                distracted_minutes,
                (distracted_minutes / total_minutes) * 100.0
            ));
        }
        
        // Idle observations
        if idle_minutes > 0.0 {
            observations.push_str(&format!(
                "You were idle for {:.0} minutes ({:.1}% of your time). ",
                idle_minutes,
                (idle_minutes / total_minutes) * 100.0
            ));
        }
        
        // Application observations
        if let Some((app, duration)) = summary.application_breakdown.iter()
            .max_by(|a, b| a.1.cmp(b.1)) {
            if duration.num_minutes() > 0 {
                observations.push_str(&format!(
                    "The application you used most was '{}' for {:.0} minutes. ",
                    app, duration.num_minutes()
                ));
            }
        }
        
        // Time distribution rating
        let time_distribution_score = if total_minutes > 0.0 {
            let active_time = productive_minutes + distracted_minutes;
            if active_time > 0.0 {
                (productive_minutes / active_time) * 100.0
            } else {
                100.0
            }
        } else {
            100.0
        };
        
        let rating = self.get_time_distribution_rating(time_distribution_score);
        observations.push_str(&format!(
            "Your time distribution pattern for this session is categorized as '{}'.",
            rating
        ));
        
        observations
    }
} 