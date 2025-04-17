use std::fs;
use chrono::Duration;
use std::collections::HashMap;

pub struct ReportGenerator {
    data_directory: String,
}

impl ReportGenerator {
    pub fn new(data_directory: String) -> Self {
        ReportGenerator { data_directory }
    }

    pub fn generate_report(&self, summary: &super::DailySummary) {
        // Create a user-friendly HTML report
        let productive_hours = summary.productive_time.num_minutes() as f64 / 60.0;
        let distracted_hours = summary.distracted_time.num_minutes() as f64 / 60.0;
        let idle_hours = summary.idle_time.num_minutes() as f64 / 60.0;

        let total_hours = productive_hours + distracted_hours + idle_hours;
        let productive_percentage = if total_hours > 0.0 {
            (productive_hours / total_hours) * 100.0
        } else {
            0.0
        };
        let distracted_percentage = if total_hours > 0.0 {
            (distracted_hours / total_hours) * 100.0
        } else {
            0.0
        };
        let idle_percentage = if total_hours > 0.0 {
            (idle_hours / total_hours) * 100.0
        } else {
            0.0
        };

        // Calculate productivity score (0-100)
        let productivity_score = if total_hours > 0.0 {
            (productive_hours / total_hours) * 100.0
        } else {
            0.0
        };

        // Get productivity rating
        let productivity_rating = self.get_productivity_rating(productivity_score);

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
        .productivity-score {{
            font-size: 24px;
            font-weight: bold;
            text-align: center;
            margin: 20px 0;
            padding: 10px;
            border-radius: 5px;
        }}
        .high-productivity {{
            background-color: #dff0d8;
            color: #3c763d;
        }}
        .medium-productivity {{
            background-color: #fcf8e3;
            color: #8a6d3b;
        }}
        .low-productivity {{
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
            <div class="productivity-score {}">
                Productivity Score: {:.1}% - {}
            </div>
            
            <div class="section">
                <h3 class="section-title">Time Distribution</h3>
                <div class="stats-grid">
                    <div class="stat-card">
                        <div class="stat-value productive-value">{:.2} hours</div>
                        <div class="stat-label">Productive Time</div>
                    </div>
                    <div class="stat-card">
                        <div class="stat-value distracted-value">{:.2} hours</div>
                        <div class="stat-label">Distracted Time</div>
                    </div>
                    <div class="stat-card">
                        <div class="stat-value idle-value">{:.2} hours</div>
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
                <h3 class="section-title">Application Breakdown</h3>
                <table>
                    <tr>
                        <th>Application</th>
                        <th>Hours</th>
                        <th>Percentage</th>
                    </tr>
                    {}
                </table>
            </div>
            
            <div class="section">
                <h3 class="section-title">Activity Breakdown</h3>
                <table>
                    <tr>
                        <th>Activity Type</th>
                        <th>Hours</th>
                        <th>Percentage</th>
                    </tr>
                    {}
                </table>
            </div>
            
            <div class="section">
                <h3 class="section-title">Productivity Insights</h3>
                <p>{}</p>
            </div>
        </div>
    </div>
</body>
</html>"#,
            summary.date,
            summary.date,
            self.get_productivity_class(productivity_score),
            productivity_score,
            productivity_rating,
            productive_hours,
            distracted_hours,
            idle_hours,
            productive_percentage,
            distracted_percentage,
            idle_percentage,
            self.generate_application_table(&summary.application_breakdown, total_hours),
            self.generate_activity_table(&summary.activity_breakdown, total_hours),
            self.generate_productivity_insights(summary)
        );

        let filename = format!("{}/report_{}.html", self.data_directory, summary.date);
        fs::write(filename, html).expect("Failed to write HTML report");
    }

    fn generate_application_table(&self, app_breakdown: &HashMap<String, Duration>, total_hours: f64) -> String {
        // Sort applications by duration (descending)
        let mut sorted_apps: Vec<(&String, &Duration)> = app_breakdown.iter().collect();
        sorted_apps.sort_by(|a, b| b.1.cmp(a.1));
        
        sorted_apps
            .iter()
            .map(|(app, duration)| {
                let hours = duration.num_minutes() as f64 / 60.0;
                let percentage = if total_hours > 0.0 {
                    (hours / total_hours) * 100.0
                } else {
                    0.0
                };
                format!(
                    "<tr><td>{}</td><td>{:.2}</td><td>{:.1}%</td></tr>",
                    app,
                    hours,
                    percentage
                )
            })
            .collect::<Vec<String>>()
            .join("")
    }
    
    fn generate_activity_table(&self, activity_breakdown: &HashMap<String, Duration>, total_hours: f64) -> String {
        // Sort activities by duration (descending)
        let mut sorted_activities: Vec<(&String, &Duration)> = activity_breakdown.iter().collect();
        sorted_activities.sort_by(|a, b| b.1.cmp(a.1));
        
        sorted_activities
            .iter()
            .map(|(activity, duration)| {
                let hours = duration.num_minutes() as f64 / 60.0;
                let percentage = if total_hours > 0.0 {
                    (hours / total_hours) * 100.0
                } else {
                    0.0
                };
                format!(
                    "<tr><td>{}</td><td>{:.2}</td><td>{:.1}%</td></tr>",
                    activity,
                    hours,
                    percentage
                )
            })
            .collect::<Vec<String>>()
            .join("")
    }
    
    fn get_productivity_rating(&self, score: f64) -> String {
        if score >= 80.0 {
            "Excellent".to_string()
        } else if score >= 60.0 {
            "Good".to_string()
        } else if score >= 40.0 {
            "Fair".to_string()
        } else {
            "Needs Improvement".to_string()
        }
    }
    
    fn get_productivity_class(&self, score: f64) -> String {
        if score >= 60.0 {
            "high-productivity".to_string()
        } else if score >= 40.0 {
            "medium-productivity".to_string()
        } else {
            "low-productivity".to_string()
        }
    }
    
    fn generate_productivity_insights(&self, summary: &super::DailySummary) -> String {
        let productive_hours = summary.productive_time.num_minutes() as f64 / 60.0;
        let distracted_hours = summary.distracted_time.num_minutes() as f64 / 60.0;
        let idle_hours = summary.idle_time.num_minutes() as f64 / 60.0;
        let total_hours = productive_hours + distracted_hours + idle_hours;
        
        let mut insights = String::new();
        
        // Productivity insights
        if productive_hours > 0.0 {
            insights.push_str(&format!(
                "You spent {:.1} hours on productive activities, which is {:.1}% of your tracked time. ",
                productive_hours,
                (productive_hours / total_hours) * 100.0
            ));
        }
        
        // Distraction insights
        if distracted_hours > 0.0 {
            insights.push_str(&format!(
                "You were distracted for {:.1} hours ({:.1}% of your time). ",
                distracted_hours,
                (distracted_hours / total_hours) * 100.0
            ));
        }
        
        // Idle insights
        if idle_hours > 0.0 {
            insights.push_str(&format!(
                "You were idle for {:.1} hours ({:.1}% of your time). ",
                idle_hours,
                (idle_hours / total_hours) * 100.0
            ));
        }
        
        // Application insights
        if let Some((app, duration)) = summary.application_breakdown.iter()
            .max_by(|a, b| a.1.cmp(b.1)) {
            let app_hours = duration.num_minutes() as f64 / 60.0;
            insights.push_str(&format!(
                "The application you used most was '{}' for {:.1} hours. ",
                app, app_hours
            ));
        }
        
        // Productivity rating
        let productivity_score = if total_hours > 0.0 {
            (productive_hours / total_hours) * 100.0
        } else {
            0.0
        };
        
        let rating = self.get_productivity_rating(productivity_score);
        insights.push_str(&format!(
            "Your overall productivity rating for today is '{}'.",
            rating
        ));
        
        insights
    }
} 