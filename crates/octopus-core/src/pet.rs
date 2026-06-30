use std::env;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, serde::Serialize)]
pub(crate) struct PetReport {
    pub(crate) state: String,
    pub(crate) title: String,
    pub(crate) summary: String,
    pub(crate) color: String,
    pub(crate) head_color: String,
    pub(crate) motion: String,
    pub(crate) chat_badge: String,
    pub(crate) event_source: Option<String>,
    pub(crate) event_summary: Option<String>,
    pub(crate) path: String,
    pub(crate) target: String,
    pub(crate) exists: bool,
}

#[derive(Debug, serde::Serialize)]
pub(crate) struct PetImageReport {
    pub(crate) pet: PetReport,
    pub(crate) image_path: String,
    pub(crate) image_url: String,
    pub(crate) format: String,
    pub(crate) bytes: usize,
}

const OCTOPUS_PIXEL_ROWS: [&str; 12] = [
    ".....bbbbb.....",
    "...bbbbbbbbb...",
    "..bbbbbbbbbbb..",
    ".bbbbbbbbbbbbb.",
    ".bbbbebbbebbbb.",
    ".bbbbbbbbbbbbb.",
    "..bbbbbbbbbbb..",
    "...bbbbbbbbb...",
    "..bb.bbb.bb....",
    ".bb..bbb..bb...",
    "bb...b.b...bb..",
    "b....b.b....b..",
];

pub(crate) fn pet_report(state: &str, path: &Path) -> Result<PetReport, String> {
    let (state, title, summary, color, head_color, motion, chat_badge) = pet_state_info(state)?;
    let path_text = path.to_string_lossy().to_string();
    let target = format!("{}?state={state}", file_url(path));
    Ok(PetReport {
        state: state.to_string(),
        title: title.to_string(),
        summary: summary.to_string(),
        color: color.to_string(),
        head_color: head_color.to_string(),
        motion: motion.to_string(),
        chat_badge: chat_badge.to_string(),
        event_source: None,
        event_summary: None,
        target,
        exists: path.exists(),
        path: path_text,
    })
}

pub(crate) fn state_known(state: &str) -> bool {
    pet_state_info(state).is_ok()
}

pub(crate) fn default_pet_image_path(state_path: &Path, state: &str) -> PathBuf {
    let base = state_path
        .parent()
        .filter(|path| !path.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    base.join("pet").join(format!("octopus-{state}.svg"))
}

pub(crate) fn write_pet_image_report(
    report: PetReport,
    path: &Path,
) -> Result<PetImageReport, String> {
    let svg = pet_svg(&report);
    if let Some(parent) = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
    {
        fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    }
    fs::write(path, svg.as_bytes()).map_err(|error| error.to_string())?;
    let image_path = path.to_string_lossy().to_string();
    let image_url_path = if path.is_absolute() {
        path.to_path_buf()
    } else {
        env::current_dir()
            .map_err(|error| error.to_string())?
            .join(path)
    };
    Ok(PetImageReport {
        pet: report,
        image_url: file_url(&image_url_path),
        image_path,
        format: "svg".to_string(),
        bytes: svg.len(),
    })
}

pub(crate) fn percent_encode_path(path: &str) -> String {
    let mut encoded = String::new();
    for byte in path.as_bytes() {
        match *byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'/' | b'.' | b'-' | b'_' | b'~' => {
                encoded.push(*byte as char)
            }
            value => encoded.push_str(&format!("%{value:02X}")),
        }
    }
    encoded
}

fn pet_svg(report: &PetReport) -> String {
    let pixel = 16;
    let gap = 2;
    let margin = 12;
    let width = margin * 2 + 15 * pixel + 14 * gap;
    let height = margin * 2 + OCTOPUS_PIXEL_ROWS.len() as i32 * pixel + 11 * gap;
    let mut body = String::new();
    for (row_index, row) in OCTOPUS_PIXEL_ROWS.iter().enumerate() {
        for (column_index, value) in row.chars().enumerate() {
            if value == '.' {
                continue;
            }
            let x = margin + column_index as i32 * (pixel + gap);
            let y = margin + row_index as i32 * (pixel + gap);
            let fill = if row_index <= 6 {
                &report.head_color
            } else {
                &report.color
            };
            match value {
                'b' => body.push_str(&format!(
                    r#"<rect x="{x}" y="{y}" width="{pixel}" height="{pixel}" rx="2" fill="{fill}"/>"#
                )),
                'e' => {
                    let pupil = 6;
                    let pupil_x = x + 5;
                    let pupil_y = y + 5;
                    body.push_str(&format!(
                        r##"<rect x="{x}" y="{y}" width="{pixel}" height="{pixel}" rx="2" fill="#ffffff"/><rect x="{pupil_x}" y="{pupil_y}" width="{pupil}" height="{pupil}" rx="1" fill="#101318"/>"##
                    ));
                }
                _ => {}
            }
        }
    }
    format!(
        r##"<svg xmlns="http://www.w3.org/2000/svg" width="{width}" height="{height}" viewBox="0 0 {width} {height}" role="img" aria-labelledby="title desc" shape-rendering="crispEdges">
<title id="title">Octopus pixel pet: {}</title>
<desc id="desc">{}</desc>
<rect width="100%" height="100%" rx="8" fill="#f5f7fa"/>
{}
</svg>
"##,
        xml_escape(&report.title),
        xml_escape(&report.summary),
        body
    )
}

fn file_url(path: &Path) -> String {
    format!("file://{}", percent_encode_path(&path.to_string_lossy()))
}

fn xml_escape(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

fn pet_state_info(
    state: &str,
) -> Result<
    (
        &'static str,
        &'static str,
        &'static str,
        &'static str,
        &'static str,
        &'static str,
        &'static str,
    ),
    String,
> {
    match state {
        "heartbeat" | "alive" => Ok((
            "heartbeat",
            "Heartbeat",
            "Kernel and chat loop are alive.",
            "#0f766e",
            "#0f766e",
            "breathe",
            "🟩",
        )),
        "need" => Ok((
            "need",
            "Need",
            "The clean brain produced a cognitive Need.",
            "#f973a9",
            "#ff4f8b",
            "head",
            "🟧",
        )),
        "action" | "executing" => Ok((
            "action",
            "Action",
            "A tentacle is observing, calling tools, or running code.",
            "#f59e0b",
            "#f97316",
            "tentacles",
            "🟨",
        )),
        "feed" => Ok((
            "feed",
            "Feed",
            "A tentacle compressed action results back into Feed.",
            "#22a06b",
            "#16a34a",
            "feed",
            "🟩",
        )),
        "memory" => Ok((
            "memory",
            "Memory beat",
            "Context was recalled, compacted, or forgotten.",
            "#6d5bd0",
            "#7c6ee6",
            "pulse",
            "🟪",
        )),
        "harness" | "route" => Ok((
            "harness",
            "Harness",
            "Routes or tools are adapting from feedback.",
            "#2563eb",
            "#4f8cff",
            "evolve",
            "🟦",
        )),
        "evolution" | "evolving" => Ok((
            "evolution",
            "Evolution",
            "Routes or tools are adapting from feedback.",
            "#2563eb",
            "#4f8cff",
            "evolve",
            "🟦",
        )),
        "blocked" => Ok((
            "blocked",
            "Blocked",
            "The harness needs a grant or external change.",
            "#b93827",
            "#dc2626",
            "blocked",
            "🟥",
        )),
        "success" | "satisfied" => Ok((
            "success",
            "Success",
            "Feedback returned useful evidence.",
            "#16833a",
            "#16a34a",
            "feed",
            "🟩",
        )),
        value => Err(format!(
            "unknown pet state: {value}; expected heartbeat, need, action, feed, memory, harness, evolution, blocked, or success"
        )),
    }
}
