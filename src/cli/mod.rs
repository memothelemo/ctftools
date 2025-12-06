use anstyle::Style;
use anyhow::{Context, Result};
use console::Term;
use log::{debug, info};
use std::io::Write;
use std::sync::Arc;

use crate::env::Environment;
use crate::registry::Toolkit;

pub mod action;
pub mod interactive;
pub mod options;

pub mod check_tools;
pub mod install_tools;

pub use self::action::Action;
pub use self::options::Options;

pub fn run(env: Arc<dyn Environment>, opts: Options, toolkit: Option<Toolkit>) -> Result<()> {
    let is_env_live = env.is_live();

    // Initialize logger
    init_logger();

    debug!("using environment: {env:?}");
    let stderr = Term::stderr();

    // Load our toolkit to be used for the entire program's lifetime.
    let toolkit = init_maybe_custom_toolkit(&opts, toolkit)?;

    // Do not enter interactive if we're in a mock environment.
    //
    // If we're in mock environment, we can directly run them.
    if let Some(action) = opts.action {
        self::interactive::print_cli_header();
        self::try_run_action(action, &*env, &stderr, &toolkit)?;

        if !is_env_live {
            return Ok(());
        }

        // Check if the user wants to go back to the selector menu
        let should_enter_interactive_mode = self::interactive::prompt_yes_or_no(
            "Do you want to go back to the tool selector menu?",
        )?
        .unwrap_or(false);

        if !should_enter_interactive_mode {
            self::interactive::print_goodbye_message();
            return Ok(());
        }
    } else if !is_env_live {
        panic!("Action is required to perform an action in mocked system environment");
    }

    self::interactive::enter_interactive_mode(&*env, &stderr, &toolkit)
}

pub fn try_run_action(
    action: Action,
    env: &dyn Environment,
    stderr: &Term,
    toolkit: &Toolkit,
) -> Result<()> {
    use self::install_tools::InstallGoal;
    match action {
        Action::Tool(..) => todo!(),
        Action::CheckTools => self::check_tools::run(env, stderr, toolkit),
        Action::InstallMissingTools => {
            self::install_tools::install(env, InstallGoal::Missing, stderr, toolkit)
        }
        #[cfg(debug_assertions)]
        Action::InstallAllTools => {
            self::install_tools::install(env, InstallGoal::Everything, stderr, toolkit)
        }
        Action::Exit => Ok(()),
    }
}

fn debug_enabled() -> bool {
    std::env::var("CTFTOOLS_DEBUG").as_deref().unwrap_or("0") != "0"
}

fn init_maybe_custom_toolkit(opts: &Options, existing_toolkit: Option<Toolkit>) -> Result<Toolkit> {
    if let Some(toolkit) = existing_toolkit {
        debug!(
            "using existing toolkit passed from `run` function; loaded tool(s) = {}",
            toolkit.tools().len()
        );
        Ok(toolkit)
    } else if let Some(json) = opts.custom_toolkit.as_ref() {
        let toolkit = Toolkit::from_json(json).context("could not load custom toolkit")?;
        debug!(
            "using custom toolkit; loaded tool(s) = {}",
            toolkit.tools().len()
        );
        Ok(toolkit)
    } else {
        Ok(Toolkit::default().clone())
    }
}

fn init_logger() {
    let debug_enabled = debug_enabled();
    let _ = env_logger::Builder::new()
        .filter_level(if debug_enabled {
            log::LevelFilter::Debug
        } else {
            log::LevelFilter::Warn
        })
        .format(|buf, record| {
            use anstyle::AnsiColor;
            use log::Level;

            write!(buf, "{GRAY}[{GRAY:#}")?;

            let (level_str, color) = match record.level() {
                Level::Error => ("ERROR ", AnsiColor::BrightRed),
                Level::Warn => ("WARN ", AnsiColor::BrightYellow),
                Level::Info => ("INFO ", AnsiColor::BrightGreen),
                Level::Debug => ("DEBUG", AnsiColor::BrightBlue),
                Level::Trace => ("TRACE", AnsiColor::BrightMagenta),
            };
            let style = Style::new().fg_color(Some(color.into()));
            write!(buf, "{style}{level_str}{style:#}")?;

            let module_path = record.module_path().unwrap_or("unknown");
            write!(
                buf,
                "{GRAY}] {module_path}:{}{GRAY:#} - ",
                record.line().unwrap_or_default()
            )?;

            writeln!(buf, "{}", record.args())
        })
        .format_timestamp(None)
        .try_init();

    if debug_enabled {
        info!("debug logging is enabled");
    }
}

pub mod ansi {
    use anstyle::{AnsiColor, Color, Style};

    pub const GRAY: Style = Style::new().fg_color(Some(Color::Ansi(AnsiColor::BrightBlack)));
    pub const BOLD: Style = Style::new().bold();

    pub const GREEN: Style = Style::new().fg_color(Some(Color::Ansi(AnsiColor::BrightGreen)));
    pub const GREEN_BOLD: Style = GREEN.bold();

    pub const YELLOW: Style = Style::new().fg_color(Some(Color::Ansi(AnsiColor::BrightYellow)));
    pub const YELLOW_BOLD: Style = YELLOW.bold();

    pub const RED: Style = Style::new().fg_color(Some(Color::Ansi(AnsiColor::Red)));
}
use self::ansi::*;

trait TermExt {
    fn clear_lines(&self, lines: usize) -> Result<()>;
}

impl TermExt for console::Term {
    fn clear_lines(&self, lines: usize) -> Result<()> {
        if !crate::cli::debug_enabled() {
            self.clear_last_lines(lines)?;
        }
        Ok(())
    }
}
