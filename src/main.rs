use std::time::{Duration, Instant};
use ratatui::{
    prelude::*,
    widgets::*,
    Terminal,
    backend::CrosstermBackend,
};
use battery::{Battery, Manager, State};

#[derive(Debug, Clone)]
pub struct BatteryInfo {
    pub name: String, // Battery name or identifier
    pub voltage: f64, // in volts
    pub current: f64, // in amperes
    pub power: f64, // in watts
    pub state: State,
    pub percentage: f64,
}

pub struct PowerSupplyMonitor;

impl PowerSupplyMonitor {
    pub fn new() -> Self {
        PowerSupplyMonitor
    }

    pub fn get_batteries(&self) -> Result<Vec<BatteryInfo>, Box<dyn std::error::Error>> {
        let mut batteries = Vec::new();
        
        let manager = Manager::new()?;
        let cells = manager.batteries()?;
        
        for battery_result in cells {
            if let Ok(battery) = battery_result {
                let battery_info = self.convert_battery_to_info(battery);
                batteries.push(battery_info);
            }
        }
        
        Ok(batteries)
    }

    fn convert_battery_to_info(&self, battery: Battery) -> BatteryInfo {
        let name = format!("Battery");
        
        let voltage = battery.voltage().get::<battery::units::electric_potential::volt>() as f64;
        let energy_rate = battery.energy_rate().get::<battery::units::power::watt>() as f64;

        let energy_rate_mw = battery.energy_rate().get::<battery::units::power::milliwatt>() as f64;
        let voltage_mv = battery.voltage().get::<battery::units::electric_potential::millivolt>() as f64;
        
        let current = if voltage_mv != 0.0 {
            energy_rate_mw / voltage_mv
        } else {
            0.0
        };
        
        let percentage = battery.state_of_charge().get::<battery::units::ratio::percent>() as f64;
        
        BatteryInfo {
            name,
            voltage,
            current,
            power: energy_rate,
            state: battery.state(),
            percentage,
        }
    }

    pub fn calculate_power(&self, info: &BatteryInfo) -> f64 {
        // Calculate power in watts (voltage in microvolts * current in microamps / 1e12)
        (info.voltage * info.current) / 1e12
    }
}

pub struct App {
    pub batteries: Vec<BatteryInfo>,
    pub should_exit: bool,
    last_refresh: Instant,
    refresh_interval: Duration,
}

impl App {
    pub fn new() -> Self {
        let monitor = PowerSupplyMonitor::new();
        let mut app = App {
            batteries: Vec::new(),
            should_exit: false,
            last_refresh: Instant::now(),
            refresh_interval: Duration::from_secs(2), // Auto-refresh every 2 seconds
        };

        // Load initial battery info
        if let Ok(battery_info) = monitor.get_batteries() {
            app.batteries = battery_info;
        }

        app
    }

    pub fn refresh_data(&mut self) {
        let monitor = PowerSupplyMonitor::new();

        if let Ok(battery_info) = monitor.get_batteries() {
            self.batteries = battery_info;
        }
        self.last_refresh = Instant::now();
    }

    pub fn should_refresh(&self) -> bool {
        self.last_refresh.elapsed() >= self.refresh_interval
    }

    pub fn handle_key_event(&mut self, key: crossterm::event::KeyEvent) {
        match key.code {
            crossterm::event::KeyCode::Char('q') | crossterm::event::KeyCode::Esc => {
                self.should_exit = true;
            }
            crossterm::event::KeyCode::Char('r') | crossterm::event::KeyCode::Enter => {
                self.refresh_data();
            }
            crossterm::event::KeyCode::Char('+') => {
                // Increase refresh interval
                let new_secs = (self.refresh_interval.as_secs() + 1).min(10);
                self.refresh_interval = Duration::from_secs(new_secs);
            }
            crossterm::event::KeyCode::Char('-') => {
                // Decrease refresh interval
                let new_secs = (self.refresh_interval.as_secs().saturating_sub(1)).max(1);
                self.refresh_interval = Duration::from_secs(new_secs);
            }
            _ => {}
        }
    }
}

fn draw_battery_info(frame: &mut Frame, app: &App) {
    let area = frame.area();
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(0),
            Constraint::Length(3),
        ])
        .split(area);

    // Title
    let title = Block::default()
        .title("Power Seek - 电源监控")
        .title_alignment(Alignment::Center)
        .borders(Borders::NONE);
    frame.render_widget(title, chunks[0]);

    // Battery list
    let battery_widgets: Vec<ListItem> = app.batteries.iter().map(|battery| {

        // Convert battery state to Chinese
        let state_str = match &battery.state {
            State::Charging => "充电".to_string(),
            State::Discharging => "放电".to_string(),
            State::Unknown => "未知".to_string(),
            State::Empty => "空".to_string(),
            State::Full => "满".to_string(),
            _ => "未知".to_string(),
        };

        let mut lines = vec![
            Line::from(format!("电池: {}", battery.name)),
            Line::from(format!("状态: {}", state_str)),
        ];
        
        // Add percentage if available
        if battery.percentage > 0.0 {
            lines.push(Line::from(format!("电量: {:.2}%", battery.percentage)));
        }
        
        lines.extend([
            Line::from(format!("电压: {:.2}V", battery.voltage)),
            Line::from(format!("电流: {:.2}A", battery.current)),
            Line::from(format!("功率: {:.2}W", battery.power)),
            Line::from(""),
        ]);

        ListItem::new(lines)
    }).collect();

    let battery_list = List::new(battery_widgets)
        .block(Block::default()
            .title("电池信息")
            .borders(Borders::ALL))
        .highlight_style(Style::default().add_modifier(Modifier::REVERSED))
        .highlight_symbol(">> ");

    frame.render_widget(battery_list, chunks[1]);

    // Footer with refresh info
    let refresh_info = format!("刷新间隔: {}s | 按 '+' 增加, '-' 减少 | 按 'q' 退出, 'r' 手动刷新", 
                               app.refresh_interval.as_secs());
    let footer = Paragraph::new(refresh_info)
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::TOP));
    frame.render_widget(footer, chunks[2]);
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Setup terminal
    crossterm::terminal::enable_raw_mode()?;
    let mut stdout = std::io::stdout();
    crossterm::execute!(stdout, crossterm::terminal::EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create app instance
    let mut app = App::new();

    // Main loop
    loop {
        // Auto-refresh if needed
        if app.should_refresh() {
            app.refresh_data();
        }

        // Draw UI
        terminal.draw(|f| draw_battery_info(f, &app))?;

        // Handle events
        if crossterm::event::poll(std::time::Duration::from_millis(100))? {
            if let crossterm::event::Event::Key(key) = crossterm::event::read()? {
                app.handle_key_event(key);
                
                if app.should_exit {
                    break;
                }
            }
        }
    }

    // Restore terminal
    crossterm::execute!(
        terminal.backend_mut(),
        crossterm::terminal::LeaveAlternateScreen
    )?;
    crossterm::terminal::disable_raw_mode()?;
    Ok(())
}