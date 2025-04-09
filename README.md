# TimeSense

A privacy-focused, automated time awareness tool that helps you understand how you *actually* spend your time versus how you *think* you spend it.

## Overview

TimeSense runs in the background on your system, periodically capturing screenshots to analyze what applications you're using and categorizing your activities. Unlike traditional time trackers that require manual input, TimeSense automatically builds an accurate picture of your time usage patterns and provides insightful reports to help you improve your productivity.

## Features

- **Automatic Activity Detection**: Recognizes which applications you're using without manual tracking
- **Idle Detection**: Distinguishes between active work and idle time
- **Activity Categorization**: Automatically classifies activities as productive, distracting, or neutral
- **Daily Reports**: Generates daily summaries showing how you spent your time
- **Visual Analytics**: Provides charts and graphs to visualize your time usage
- **Privacy First**: All data stays on your machine; no data is sent to external servers

## Installation

### Prerequisites

- Rust 1.67 or higher
- Tesseract OCR engine

### From Source

```bash
# Clone the repository
git clone https://github.com/alexandercannon/timesense.git
cd timesense

# Build the project
cargo build --release

# Run the application
./target/release/timesense
```

## Configuration

On first run, TimeSense will create a default `timesense_config.json` file in the current directory. You can customize the following settings:

```json
{
  "screenshot_interval_seconds": 60,
  "idle_threshold_seconds": 180,
  "productive_apps": ["code", "terminal", "notion"],
  "distraction_apps": ["twitter", "youtube", "reddit"],
  "data_directory": "timesense_data"
}
```

- **screenshot_interval_seconds**: How frequently TimeSense captures screen states
- **idle_threshold_seconds**: Time without input before considering system idle
- **productive_apps**: Applications considered productive
- **distraction_apps**: Applications considered distractions
- **data_directory**: Where TimeSense stores your data and reports

## Usage

Once started, TimeSense runs in the background, collecting data about your computer usage. Daily reports are automatically generated and stored in the configured data directory.

To view your reports, open the HTML files in the data directory with your browser:

```bash
# Open today's report
open timesense_data/report_2025-04-09.html
```

## Development Roadmap

- [ ] Improved application detection using system APIs instead of OCR
- [ ] Machine learning for smarter activity categorization
- [ ] Weekly and monthly trend analysis
- [ ] Goal setting and progress tracking
- [ ] System tray icon with quick stats and controls
- [ ] Browser extension for detailed web activity tracking

## Privacy Notice

TimeSense is designed with privacy in mind:
- All data processing happens locally on your machine
- Screenshots are analyzed and immediately deleted
- No network connections are made by the application
- All collected data stays on your device

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

This project is licensed under the MIT License - see the LICENSE file for details.