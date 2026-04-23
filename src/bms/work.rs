use std::collections::HashMap;

pub struct ExtractOpts {
    pub remove_unclosed_pair: bool,
    pub remove_tailing_sign_list: Vec<String>,
}

impl Default for ExtractOpts {
    fn default() -> Self {
        Self {
            remove_unclosed_pair: true,
            remove_tailing_sign_list: vec![],
        }
    }
}

#[must_use]
pub fn extract_work_name(titles: &[String], opts: &ExtractOpts) -> String {
    let mut prefix_counts: HashMap<String, usize> = HashMap::new();

    for title in titles {
        let chars: Vec<char> = title.chars().collect();
        for i in 1..=chars.len() {
            let prefix: String = chars[..i].iter().collect();
            *prefix_counts.entry(prefix).or_default() += 1;
        }
    }

    if prefix_counts.is_empty() {
        return String::new();
    }

    let max_count = *prefix_counts.values().max().unwrap_or(&0);
    #[allow(clippy::cast_precision_loss)]
    let threshold = max_count as f64 * 0.67;

    #[allow(clippy::cast_precision_loss)]
    let mut candidates: Vec<(String, usize)> = prefix_counts
        .into_iter()
        .filter(|(_, count)| *count as f64 >= threshold)
        .collect();

    candidates.sort_by(|a, b| {
        b.0.chars()
            .count()
            .cmp(&a.0.chars().count())
            .then_with(|| b.1.cmp(&a.1))
            .then_with(|| a.0.cmp(&b.0))
    });

    let best = candidates.first().map_or("", |(s, _)| s.as_str());

    extract_work_name_post_process(best, opts)
}

#[must_use]
pub fn extract_work_name_post_process(s: &str, opts: &ExtractOpts) -> String {
    const PAIRS: [(char, char); 7] = [
        ('(', ')'),
        ('[', ']'),
        ('{', '}'),
        ('（', '）'),
        ('［', '］'),
        ('｛', '｝'),
        ('【', '】'),
    ];

    let mut s = s.trim().to_owned();

    loop {
        let mut triggered = false;

        if opts.remove_unclosed_pair {
            let mut stack: Vec<(char, usize)> = Vec::new();

            for (i, c) in s.char_indices() {
                for &(p_open, p_close) in &PAIRS {
                    if c == p_open {
                        stack.push((p_open, i));
                    }
                    if c == p_close && stack.last().is_some_and(|(ch, _)| *ch == p_open) {
                        stack.pop();
                    }
                }
            }

            if let Some(&(.., pos)) = stack.last() {
                s = s[..pos].trim_end().to_owned();
                triggered = true;
            }
        }

        for sign in &opts.remove_tailing_sign_list {
            if s.ends_with(sign.as_str()) {
                s = s[..s.len() - sign.len()].trim_end().to_owned();
                triggered = true;
            }
        }

        if !triggered {
            break;
        }
    }

    s
}
