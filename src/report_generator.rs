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
            productive_percentage,
            distracted_percentage,
            idle_percentage,
            productive_hours,
            distracted_hours,
            idle_hours,
            self.generate_application_table(&summary.application_breakdown)
        );

        let filename = format!("{}/report_{}.html", self.data_directory, summary.date);
        fs::write(filename, html).expect("Failed to write HTML report");
    }

    fn generate_application_table(&self, app_breakdown: &HashMap<String, Duration>) -> String {
        app_breakdown
            .iter()
            .map(|(app, duration)| {
                format!(
                    "<tr><td>{}</td><td>{:.2}</td></tr>",
                    app,
                    duration.num_minutes() as f64 / 60.0
                )
            })
            .collect::<Vec<String>>()
            .join("")
    }
} 