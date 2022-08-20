use fern::colors::{Color, ColoredLevelConfig};
use log::LevelFilter;

pub fn enable_logging(verbosity: usize) -> anyhow::Result<()> {
    let line_colors = ColoredLevelConfig::new()
        .error(Color::Red)
        .warn(Color::Yellow)
        .trace(Color::BrightBlack);

    let level_colors = line_colors.clone().info(Color::Green);

    fern::Dispatch::new()
        .format(move |out, msg, record| {
            out.finish(format_args!(
                "{line_color}[{level}{line_color}] {msg}\x1B[0m",
                line_color = format_args!(
                    "\x1B[{}m",
                    line_colors.get_color(&record.level()).to_fg_str()
                ),
                // target = record.target(),
                level = level_colors.color(record.level()),
                msg = msg
            ))
        })
        .level(dbg!(match verbosity {
            0 => LevelFilter::Info,
            1 => LevelFilter::Debug,
            2.. => LevelFilter::Trace,
            _ => unreachable!(),
        }))
        .chain(std::io::stdout())
        .apply()?;
    Ok(())
}
