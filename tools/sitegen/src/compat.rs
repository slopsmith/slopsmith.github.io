use std::collections::{BTreeMap, HashMap};
use std::fs;
use std::path::Path;

use anyhow::Context;
use serde::{Deserialize, Serialize};

const ACCEPTED: &str = "accepted";

const RECENT: usize = 5;

#[derive(Deserialize)]
pub struct Issue {
    number: u64,
    html_url: String,
    created_at: String,
    body: Option<String>,
    labels: Vec<Label>,
}

#[derive(Deserialize)]
struct Label {
    name: String,
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, Debug)]
#[serde(rename_all = "lowercase")]
pub enum Status {
    Perfect,
    Playable,
    Ingame,
    Menus,
    Boots,
    Nothing,
}

impl Status {
    const ALL: [Self; 6] = [Self::Perfect, Self::Playable, Self::Ingame, Self::Menus, Self::Boots, Self::Nothing];

    fn parse(s: &str) -> Option<Self> {
        let word = s.split_whitespace().last()?.to_ascii_lowercase();
        Some(match word.as_str() {
            "perfect" => Self::Perfect,
            "playable" => Self::Playable,
            "ingame" => Self::Ingame,
            "menus" => Self::Menus,
            "boots" => Self::Boots,
            "nothing" => Self::Nothing,
            _ => return None,
        })
    }

    /// The launcher's six questions, in order. See the README.
    fn from_answers(a: [bool; 6]) -> Self {
        match a {
            [false, ..] => Self::Nothing,
            [_, false, ..] => Self::Boots,
            [_, _, false, ..] => Self::Menus,
            [_, _, _, false, ..] => Self::Ingame,
            [_, _, _, _, true, _] | [_, _, _, _, _, false] => Self::Playable,
            _ => Self::Perfect,
        }
    }
}

const QUESTIONS: [&str; 6] = [
    "Anything on screen?",
    "Reached the title screen or menus?",
    "Reached gameplay?",
    "Played through, or a long session for endless games?",
    "Anything broken?",
    "Played it fully?",
];

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Report {
    issue: u64,
    url: String,
    date: String,
    status: Status,
    game_version: Option<String>,
    notes: Option<String>,
    macos: String,
    chip: String,
    ps5slop: String,
}

#[derive(Serialize, Deserialize)]
struct Title {
    name: String,
    status: Status,
    reports: Vec<Report>,
}

#[derive(Serialize)]
struct CompatJson<'a> {
    version: u32,
    titles: &'a BTreeMap<String, Title>,
}

#[derive(Serialize)]
struct SectionFrontMatter<'a> {
    title: &'a str,
    template: &'a str,
    page_template: &'a str,
    sort_by: &'a str,
    extra: SectionExtra,
}

#[derive(Serialize)]
struct SectionExtra {
    total: usize,
    counts: BTreeMap<Status, usize>,
    recent: Vec<RecentReport>,
}

#[derive(Serialize)]
struct RecentReport {
    title_id: String,
    name: String,
    #[serde(flatten)]
    report: Report,
}

#[derive(Serialize)]
struct FrontMatter<'a> {
    title: &'a str,
    weight: usize,
    extra: PageExtra<'a>,
}

#[derive(Serialize)]
struct PageExtra<'a> {
    title_id: &'a str,
    status: Status,
    reports: &'a [Report],
}

pub fn run(issues_file: &Path, out: &Path, json_out: &Path) -> anyhow::Result<()> {
    let text = fs::read_to_string(issues_file).with_context(|| format!("reading {}", issues_file.display()))?;
    let issues: Vec<Issue> = serde_json::from_str(&text)?;
    let titles = collect(&issues);

    if out.exists() {
        fs::remove_dir_all(out)?;
    }
    fs::create_dir_all(out)?;
    let section = SectionFrontMatter {
        title: "Compatibility",
        template: "ps5slop/compat.html",
        page_template: "ps5slop/title.html",
        sort_by: "weight",
        extra: summarize(&titles),
    };
    fs::write(out.join("_index.md"), format!("+++\n{}+++\n", toml::to_string(&section)?))?;

    let mut order: Vec<(&String, &Title)> = titles.iter().collect();
    order.sort_by(|a, b| a.1.status.cmp(&b.1.status).then_with(|| a.1.name.cmp(&b.1.name)));
    for (i, (id, title)) in order.iter().enumerate() {
        let fm = FrontMatter {
            title: &title.name,
            weight: i + 1,
            extra: PageExtra { title_id: id, status: title.status, reports: &title.reports },
        };
        fs::write(out.join(format!("{id}.md")), format!("+++\n{}+++\n", toml::to_string(&fm)?))?;
    }

    if let Some(dir) = json_out.parent() {
        fs::create_dir_all(dir)?;
    }
    fs::write(json_out, serde_json::to_string_pretty(&CompatJson { version: 1, titles: &titles })?)?;
    eprintln!("compat: {} titles from {} issues", titles.len(), issues.len());
    Ok(())
}

/// Groups accepted reports by title ID, newest first. A title's status is that of its newest report.
fn collect(issues: &[Issue]) -> BTreeMap<String, Title> {
    let mut titles: BTreeMap<String, Title> = BTreeMap::new();
    for issue in issues {
        if !issue.labels.iter().any(|l| l.name == ACCEPTED) {
            continue;
        }
        let Some((id, name, report)) = parse(issue) else {
            eprintln!("compat: warning: skipped #{}: missing or invalid fields", issue.number);
            continue;
        };
        titles
            .entry(id)
            .or_insert_with(|| Title { name, status: report.status, reports: Vec::new() })
            .reports
            .push(report);
    }
    for title in titles.values_mut() {
        title.reports.sort_by(|a, b| b.date.cmp(&a.date));
        title.status = title.reports[0].status;
    }
    titles
}

/// Counts titles per status and picks the newest reports across all titles.
fn summarize(titles: &BTreeMap<String, Title>) -> SectionExtra {
    let mut counts: BTreeMap<Status, usize> = Status::ALL.iter().map(|s| (*s, 0)).collect();
    let mut recent = Vec::new();
    for (id, title) in titles {
        *counts.get_mut(&title.status).unwrap() += 1;
        recent.extend(title.reports.iter().map(|r| RecentReport {
            title_id: id.clone(),
            name: title.name.clone(),
            report: r.clone(),
        }));
    }
    recent.sort_by(|a, b| b.report.date.cmp(&a.report.date));
    recent.truncate(RECENT);
    SectionExtra { total: titles.len(), counts, recent }
}

fn parse(issue: &Issue) -> Option<(String, String, Report)> {
    let fields = form_fields(issue.body.as_deref()?);
    let get = |k: &str| fields.get(k).cloned();
    let status = match get("Status") {
        Some(s) => Status::parse(&s)?,
        None => {
            let mut answers = [false; 6];
            for (a, q) in answers.iter_mut().zip(QUESTIONS) {
                *a = yes_no(&get(q)?)?;
            }
            Status::from_answers(answers)
        }
    };
    let id = get("Title ID")?;
    let report = Report {
        issue: issue.number,
        url: issue.html_url.clone(),
        date: issue.created_at.clone(),
        status,
        game_version: get("Game version"),
        notes: get("Notes"),
        macos: get("macOS version")?,
        chip: get("Chip")?,
        ps5slop: get("ps5slop version")?,
    };
    Some((id.clone(), get("Game name").unwrap_or(id), report))
}

fn yes_no(s: &str) -> Option<bool> {
    match s.to_ascii_lowercase().as_str() {
        "yes" => Some(true),
        "no" => Some(false),
        _ => None,
    }
}

/// Reads the `### Label` sections GitHub writes for issue forms. Empty answers are left out.
fn form_fields(body: &str) -> HashMap<String, String> {
    let mut fields = HashMap::new();
    for section in body.split("### ").skip(1) {
        let (label, value) = section.split_once('\n').unwrap_or((section, ""));
        let value = value.trim();
        if !value.is_empty() && value != "_No response_" {
            fields.insert(label.trim().to_string(), value.to_string());
        }
    }
    fields
}

#[cfg(test)]
mod tests {
    use super::*;

    fn issue(body: &str, accepted: bool) -> Issue {
        Issue {
            number: 1,
            html_url: String::new(),
            created_at: "2026-11-24T00:00:00Z".into(),
            body: Some(body.into()),
            labels: if accepted { vec![Label { name: ACCEPTED.into() }] } else { vec![] },
        }
    }

    const ENV: &str = "### macOS version\n\n26.1\n\n### Chip\n\nM4\n\n### ps5slop version\n\nnightly\n";

    #[test]
    fn launcher_answers_give_the_status() {
        assert_eq!(Status::from_answers([false; 6]), Status::Nothing);
        assert_eq!(Status::from_answers([true, true, false, true, true, true]), Status::Menus);
        assert_eq!(Status::from_answers([true, true, true, true, true, true]), Status::Playable);
        assert_eq!(Status::from_answers([true, true, true, true, false, false]), Status::Playable);
        assert_eq!(Status::from_answers([true, true, true, true, false, true]), Status::Perfect);
    }

    #[test]
    fn manual_report_status_is_read_from_the_dropdown() {
        let body = format!("### Title ID\n\nPPSA1\n\n### Status\n\n🟡 Ingame\n\n{ENV}");
        let (id, _, report) = parse(&issue(&body, true)).unwrap();
        assert_eq!((id.as_str(), report.status), ("PPSA1", Status::Ingame));
    }

    #[test]
    fn reports_without_environment_are_skipped() {
        let body = "### Title ID\n\nPPSA1\n\n### Status\n\n⭐ Perfect\n\n### Chip\n\n_No response_\n";
        assert!(parse(&issue(body, true)).is_none());
    }

    #[test]
    fn summary_counts_every_status() {
        let body = format!("### Title ID\n\nPPSA1\n\n### Status\n\n🟡 Ingame\n\n{ENV}");
        let summary = summarize(&collect(&[issue(&body, true)]));
        assert_eq!(summary.counts.len(), 6);
        assert_eq!(summary.counts[&Status::Ingame], 1);
        assert_eq!(summary.recent.len(), 1);
    }

    #[test]
    fn only_accepted_issues_are_read() {
        let body = format!("### Title ID\n\nPPSA1\n\n### Status\n\n⭐ Perfect\n\n{ENV}");
        assert!(collect(&[issue(&body, false)]).is_empty());
        assert_eq!(collect(&[issue(&body, true)]).len(), 1);
    }
}
