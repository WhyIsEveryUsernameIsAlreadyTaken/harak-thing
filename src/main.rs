use std::{env, fmt::Display, fs::File, io::{self, Read}, num::ParseIntError, os::raw::c_double, path::PathBuf, sync::Arc, time::Duration};

use glib::ControlFlow;
use gtk::{glib::ExitCode, prelude::{ApplicationExt, ApplicationExtManual, ContainerExt, GtkApplicationExt, LabelExt, WidgetExt}, Align, Application, ApplicationWindow, Box, Label, Orientation};

static REFRESH_INTERVAL: u64 = 100;

static WINDOWS_PATH: &str = r"\\Warframe\\EE.log";
static DEV_PATH: &str = "flyra.filtered";
static LINUX_PATH: &str = ".local/share/Steam/steamapps/compatdata/230410/pfx/drive_c/users/steamuser/AppData/Local/Warframe/EE.log";

struct LabelData {
    label: Label,
    search_string: Arc<str>,
    label_format: Arc<str>,
    count: u32,
}

impl Default for LabelData {
    fn default() -> Self {
        Self {
            label: Label::new(None),
            search_string: "".into(),
            label_format: "".into(),
            count: 0,
        }
    }
}

struct SuccessRateData {
    retired_data: LabelData,
    cleansed_data: LabelData,
    success_label: Label,
    duration_label: Label,
    // cleansing_start_times: HashSet<u32>
}

impl<'a> Default for SuccessRateData {
    fn default() -> Self {
        Self {
            retired_data: LabelData::default(),
            cleansed_data: LabelData::default(),
            success_label: Label::new(None),
            duration_label: Label::new(None),
            // cleansing_start_times: HashSet::new(),
        }
    }
}

fn count_oc_file(file: &mut File, search_string: &str) -> io::Result<u32> {
    let mut _exo_count: u32 = 0;
    let mut text = String::new();

    file.read_to_string(&mut text)?;
    _exo_count = text.lines().filter(|line| {
        let predic = line.contains(search_string);
        predic
    }).count() as u32;
    Ok(_exo_count)
}

impl LabelData {
    fn add_label(&mut self, boxw: &Box, search_string: Arc<str>, label_format: Arc<str>, interval: u64) {
        let label = Label::new(None);
        label.set_halign(Align::Center);
        boxw.add(&label);

        let mut data = LabelData::default();
        data.label = label.clone();
        data.search_string = search_string.clone();
        data.label_format = label_format.clone();

        let refresh_count = move || {
            let file_path = if cfg!(_WIN32) {
                PathBuf::from(format!("{}{}", env::var("LOCALAPPDATA").unwrap(), WINDOWS_PATH))
            } else if cfg!(DEV_MODE) {
                PathBuf::from(DEV_PATH)
            } else {
                PathBuf::from(format!("{}{}", env::var("HOME").unwrap(), LINUX_PATH))
            };
            let mut file = match File::open(file_path.clone()) {
                Ok(v) => v,
                Err(_) => {
                    println!("Error opening file: {}", file_path.to_str().unwrap());
                    return ControlFlow::Continue
                },
            };
            data.count = count_oc_file(&mut file, &search_string).unwrap();

            let label_text = format!("{}{}", label_format, data.count);
            println!("{}", label_text);
            label.set_text(label_text.as_str());

            ControlFlow::Continue
        };

        glib::timeout_add_local(Duration::from_millis(interval), refresh_count);
        self.label = data.label;
        self.search_string = data.search_string;
        self.label_format = data.label_format;
    }
}

#[derive(Debug)]
enum TimeStampParseError {
    PatternNotFound(String),
    ParseError(ParseIntError)
}

impl std::error::Error for TimeStampParseError {}

impl Display for TimeStampParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TimeStampParseError::PatternNotFound(line) => f.write_str(format!("TimeStampParseError: Pattern was not found in the following line:\n{}", line).as_str()),
            TimeStampParseError::ParseError(e) => f.write_str(format!("TimeStampParseError: {}", e).as_str()),
        }
    }
}

fn parse_timestamp(line: &str, ) -> Result<u32, TimeStampParseError> {
    let (num, _) = match line.split_once(" ") {
        Some(v) => v,
        None => return Err(TimeStampParseError::PatternNotFound(String::from(line))),
    };

    let (left, right) = match num.split_once(".") {
        Some(v) => v,
        None => return Err(TimeStampParseError::PatternNotFound(String::from(line))),
    };
    let num = format!("{}{}", left, right);

    let time_stamp = num.parse::<u32>().map_err(|e| TimeStampParseError::ParseError(e))?;
    Ok(time_stamp)
}

fn main() -> ExitCode {
    let app = Application::builder()
        .application_id("org.gtk.example")
        .build();
    app.connect_activate(|app| {
        let mut success_rate_data = SuccessRateData::default();
        let boxw = Box::new(Orientation::Vertical, 5);
        boxw.set_valign(Align::Center);
        boxw.set_halign(Align::Center);
        let window = ApplicationWindow::builder()
            .title("Gascadelyzer")
            .default_width(400)
            .default_height(200)
            .child(&boxw)
            .build();
        app.add_window(&window);
        success_rate_data.cleansed_data.add_label(&boxw, "Pillars used".into(), "Exolizers retired: ".into(), REFRESH_INTERVAL);
        success_rate_data.retired_data.add_label(&boxw, "Cleansing SurvivalLifeSupportPillarCorruptible".into(), "Exolizers cleansed: ".into(), REFRESH_INTERVAL);

        let success_label = Label::new(Some("Exolizer Defense Sucess Rate: 0.00%"));
        success_label.set_halign(Align::Center);
        boxw.add(&success_label);
        success_rate_data.success_label = success_label;

        let duration_label = Label::new(Some("Exolizer Average Cleanse Duration: 0.00 seconds"));
        duration_label.set_halign(Align::Center);
        boxw.add(&duration_label);
        success_rate_data.duration_label = duration_label;
        let refresh_success_rate = move || {
            let retired_count = success_rate_data.retired_data.count;
            let cleansed_count = success_rate_data.cleansed_data.count;
            let mut success_rate: f32 = 0.0;
            println!("COUNT: {}, {}", retired_count, cleansed_count);

            if retired_count != 0 {
                success_rate = (cleansed_count as f32 / retired_count as f32) * 100.0;
            }

            let label_text = format!("Exolizer Defense Success Rate: {:.2}%", success_rate);
            println!("{}", label_text);
            success_rate_data.success_label.set_text(label_text.as_str());

            ControlFlow::Continue
        };
        let refresh_durations = move || {
            let file_path = if cfg!(_WIN32) {
                PathBuf::from(format!("{}{}", env::var("LOCALAPPDATA").unwrap(), WINDOWS_PATH))
            } else if cfg!(DEV_MODE) {
                PathBuf::from(DEV_PATH)
            } else {
                PathBuf::from(format!("{}{}", env::var("HOME").unwrap(), LINUX_PATH))
            };
            let mut file = match File::open(file_path.clone()) {
                Ok(v) => v,
                Err(_) => {
                    println!("Error opening file: {}", file_path.to_str().unwrap());
                    return ControlFlow::Continue
                },
            };

            let mut start_time: u32 = 0;
            let mut time_stamp: u32 = 0;
            let mut total_duration: u32 = 0;
            let mut completed_cleanses: u32 = 0;
            let mut text = String::new();

            file.read_to_string(&mut text).unwrap();
            text.lines().for_each(|line| {
                if line.contains("Cleansing SurvivalLifeSupportPillarCorruptible") {
                    time_stamp = parse_timestamp(line).unwrap();
                    start_time = time_stamp;
                } else if line.contains("Pillars used increased to") {
                    time_stamp = parse_timestamp(line).unwrap();
                    total_duration += time_stamp - start_time;
                    completed_cleanses += 1;
                }
            });

            let average_duration: f32 = if completed_cleanses > 0 {
                total_duration as f32 / completed_cleanses as f32
            } else {
                0.0
            };

            let label_text = format!("Exolizer Average Cleanse Duration: {:.2}", average_duration);
            println!("{}", label_text);
            success_rate_data.duration_label.set_text(label_text.as_str());

            ControlFlow::Continue
        };
        glib::timeout_add_local(Duration::from_millis(REFRESH_INTERVAL), refresh_success_rate);
        glib::timeout_add_local(Duration::from_millis(REFRESH_INTERVAL), refresh_durations);

        window.show_all();
    });
    app.run()
}
