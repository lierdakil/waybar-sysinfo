use serde_json::json;
use std::{io::Write, path::Path, time::Duration};

fn progress_bar(value: f32, width: usize) -> String {
    let width_f32 = width as f32 * value.clamp(0.0, 1.0);
    let filled = width_f32 as usize;
    let partial = (width_f32.fract() * 8.0).round() as usize;

    const BLOCKS: [&str; 9] = [" ", "▏", "▎", "▍", "▌", "▋", "▊", "▉", "█"];
    let bar = String::from_iter(std::iter::repeat_n('█', filled));
    let partial = if width > filled { BLOCKS[partial] } else { "" };
    let rest = String::from_iter(std::iter::repeat_n(
        ' ',
        width.saturating_sub(filled).saturating_sub(1),
    ));

    format!(r#"[{bar}{partial}{rest}]"#)
}

fn top(system: &sysinfo::System, map: impl Fn(&sysinfo::Process) -> f32) -> String {
    system
        .processes()
        .iter()
        .max_by(|a, b| {
            map(a.1)
                .partial_cmp(&map(b.1))
                .unwrap_or(std::cmp::Ordering::Equal)
        })
        .map(|x| {
            x.1.exe()
                .and_then(|x| Some(x.file_name()?.display().to_string()))
                .unwrap_or_else(|| x.1.name().display().to_string())
        })
        .unwrap_or_default()
        .chars()
        .take(12)
        .collect()
}

fn main() {
    let mounts: &[&Path] = &[Path::new("/"), Path::new("/home")];
    let mut sysinfo = sysinfo::System::new_all();
    let mut disks = sysinfo::Disks::new();
    disks.refresh(true);
    sysinfo.refresh_all();
    std::thread::sleep(sysinfo::MINIMUM_CPU_UPDATE_INTERVAL);
    loop {
        sysinfo.refresh_all();
        disks.refresh(true);
        let cpu = sysinfo.global_cpu_usage().round();
        let mem =
            ((sysinfo.used_memory() as f64 / sysinfo.total_memory() as f64) as f32 * 100.0).round();
        let top_cpu = top(&sysinfo, |x| x.cpu_usage());
        let top_mem = top(&sysinfo, |x| x.memory() as f32);
        let disks: Vec<_> = disks
            .iter()
            .filter(|x| mounts.contains(&x.mount_point()))
            .collect();
        let mut disks_fmt = String::new();
        for disk in &disks {
            let usage = (((disk.total_space() - disk.available_space()) as f64
                / disk.total_space() as f64)
                * 100.0)
                .round() as f32;
            let disk_bar = progress_bar(usage / 100.0, 5);
            disks_fmt.push_str(&format!(
                "<span>{disk_bar} {usage:>3}% {mp} </span>",
                mp = disk.mount_point().display(),
            ));
        }
        let text = format!(
            r#"<span color="brown">{cpu_bar} {cpu:>3}% {top_cpu:<12} </span>
            <span color="darkcyan">{mem_bar} {mem:>3}% {top_mem:<12} </span>
            {disks_fmt}
            "#,
            cpu_bar = progress_bar(cpu / 100.0, 5),
            mem_bar = progress_bar(mem / 100.0, 5),
        )
        .replace("\n            ", "");
        println!(
            "{}",
            json!(
              {
                "text": text,
                "class": "sysinfo",
              }
            )
        );
        let _ = std::io::stdout().flush();
        std::thread::sleep(Duration::from_secs(2));
    }
}
