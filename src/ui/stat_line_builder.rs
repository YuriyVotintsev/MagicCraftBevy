#![expect(dead_code, reason = "stat diff UI helpers — scaffolding for shop perk comparison, partially wired")]

use bevy::prelude::*;

use crate::palette;
use crate::stats::{FormatSpan, SignMode, ValueTemplate};

pub fn text_color() -> Color {
    palette::color("ui_text")
}
pub fn positive_color() -> Color {
    palette::color("ui_text_positive")
}
pub fn negative_color() -> Color {
    palette::color("ui_text_negative")
}
pub fn gold_color() -> Color {
    palette::color("ui_text_money")
}
pub fn gray_color() -> Color {
    palette::color("ui_text_disabled")
}

pub enum StatRenderMode<'a> {
    Fixed {
        values: &'a [f32],
    },
    Range {
        ranges: &'a [(f32, f32)],
    },
    Diff {
        old: Option<DiffSide<'a>>,
        new: Option<DiffSide<'a>>,
        rerolled: bool,
    },
    Effective {
        effective: &'a [f32],
        raw: &'a [f32],
    },
    Snapshot {
        current: f32,
        preview: Option<f32>,
    },
}

pub struct DiffSide<'a> {
    pub values: &'a [f32],
    pub tier: usize,
}

struct Segment {
    text: String,
    color: Color,
}

fn format_value(value: f32, template: &ValueTemplate, is_percent: bool) -> String {
    let display_value = if is_percent {
        (value * 100.0).round() as i32
    } else {
        value.round() as i32
    };

    let formatted = match template.sign_mode {
        SignMode::Default => format!("{}", display_value),
        SignMode::ShowSign => {
            if display_value >= 0 {
                format!("+{}", display_value)
            } else {
                format!("{}", display_value)
            }
        }
        SignMode::Absolute => format!("{}", display_value.abs()),
    };

    format!("{}{}{}", template.prefix, formatted, template.suffix)
}

fn pick_and_format(
    value: f32,
    template: &ValueTemplate,
    negative_template: &Option<ValueTemplate>,
    is_percent: bool,
) -> String {
    if value < 0.0 {
        if let Some(neg) = negative_template {
            return format_value(value, neg, is_percent);
        }
    }
    format_value(value, template, is_percent)
}

fn benefit_color(value: f32, lower_is_better: bool) -> Color {
    let is_beneficial = (value >= 0.0) != lower_is_better;
    if is_beneficial { positive_color() } else { negative_color() }
}

fn collect_segments(spans: &[FormatSpan], mode: &StatRenderMode) -> Vec<Segment> {
    match mode {
        StatRenderMode::Fixed { values } => {
            let mut segments = Vec::new();
            for span in spans {
                match span {
                    FormatSpan::Value {
                        index,
                        is_percent,
                        lower_is_better,
                        template,
                        negative_template,
                    } => {
                        let value = values.get(*index).copied().unwrap_or(0.0);
                        let text = pick_and_format(value, template, negative_template, *is_percent);
                        let color = benefit_color(value, *lower_is_better);
                        segments.push(Segment { text, color });
                    }
                    FormatSpan::Label(text) => {
                        segments.push(Segment {
                            text: text.clone(),
                            color: text_color(),
                        });
                    }
                }
            }
            segments
        }
        StatRenderMode::Range { ranges } => {
            let mut segments = Vec::new();
            for span in spans {
                match span {
                    FormatSpan::Value {
                        index,
                        is_percent,
                        lower_is_better,
                        template,
                        negative_template,
                    } => {
                        let (min, max) = ranges.get(*index).copied().unwrap_or((0.0, 0.0));
                        let min_text =
                            pick_and_format(min, template, negative_template, *is_percent);
                        let max_text =
                            pick_and_format(max, template, negative_template, *is_percent);
                        segments.push(Segment {
                            text: min_text,
                            color: benefit_color(min, *lower_is_better),
                        });
                        segments.push(Segment {
                            text: " - ".to_string(),
                            color: text_color(),
                        });
                        segments.push(Segment {
                            text: max_text,
                            color: benefit_color(max, *lower_is_better),
                        });
                    }
                    FormatSpan::Label(text) => {
                        segments.push(Segment {
                            text: text.clone(),
                            color: text_color(),
                        });
                    }
                }
            }
            segments
        }
        StatRenderMode::Diff { old, new, rerolled } => {
            collect_diff_segments(spans, old, new, *rerolled)
        }
        StatRenderMode::Effective { effective, raw } => {
            collect_effective_segments(spans, effective, raw)
        }
        StatRenderMode::Snapshot { current, preview } => {
            collect_snapshot_segments(spans, *current, *preview)
        }
    }
}

fn collect_snapshot_segments(
    spans: &[FormatSpan],
    current: f32,
    preview: Option<f32>,
) -> Vec<Segment> {
    let show_delta = preview
        .map(|p| (p - current).abs() > 1e-3)
        .unwrap_or(false);
    let preview = preview.unwrap_or(current);

    let mut segments = Vec::new();
    for span in spans {
        match span {
            FormatSpan::Value {
                is_percent,
                lower_is_better,
                template,
                negative_template,
                ..
            } => {
                let current_text =
                    pick_and_format(current, template, negative_template, *is_percent);
                segments.push(Segment {
                    text: current_text,
                    color: text_color(),
                });
                if show_delta {
                    let preview_text =
                        pick_and_format(preview, template, negative_template, *is_percent);
                    let improved = (preview > current) != *lower_is_better;
                    let color = if improved {
                        positive_color()
                    } else {
                        negative_color()
                    };
                    segments.push(Segment {
                        text: " → ".to_string(),
                        color: text_color(),
                    });
                    segments.push(Segment {
                        text: preview_text,
                        color,
                    });
                }
            }
            FormatSpan::Label(text) => segments.push(Segment {
                text: text.clone(),
                color: text_color(),
            }),
        }
    }
    segments
}

fn collect_effective_segments(
    spans: &[FormatSpan],
    effective: &[f32],
    raw: &[f32],
) -> Vec<Segment> {
    let mut segments = Vec::new();
    for span in spans {
        match span {
            FormatSpan::Value {
                index,
                is_percent,
                template,
                negative_template,
                ..
            } => {
                let eff = effective.get(*index).copied().unwrap_or(0.0);
                let rw = raw.get(*index).copied().unwrap_or(0.0);
                let differs = (eff - rw).abs() > 1e-4;
                let eff_text = pick_and_format(eff, template, negative_template, *is_percent);
                let eff_color = if differs { positive_color() } else { text_color() };
                segments.push(Segment { text: eff_text, color: eff_color });
                if differs {
                    segments.push(Segment { text: " (".to_string(), color: gray_color() });
                    let rw_text = pick_and_format(rw, template, negative_template, *is_percent);
                    segments.push(Segment { text: rw_text, color: gray_color() });
                    segments.push(Segment { text: ")".to_string(), color: gray_color() });
                }
            }
            FormatSpan::Label(text) => segments.push(Segment {
                text: text.clone(),
                color: text_color(),
            }),
        }
    }
    segments
}

fn collect_diff_segments(
    spans: &[FormatSpan],
    old: &Option<DiffSide>,
    new: &Option<DiffSide>,
    rerolled: bool,
) -> Vec<Segment> {
    match (old, new) {
        (None, None) => {
            vec![Segment {
                text: "[empty]".to_string(),
                color: gray_color(),
            }]
        }
        (None, Some(new_side)) => {
            let mut segments = collect_uniform(spans, new_side.values, positive_color());
            segments.push(Segment {
                text: format!(" [T{}]", new_side.tier + 1),
                color: positive_color(),
            });
            segments
        }
        (Some(old_side), None) => {
            let mut segments = collect_uniform(spans, old_side.values, negative_color());
            segments.push(Segment {
                text: format!(" [T{}]", old_side.tier + 1),
                color: negative_color(),
            });
            segments
        }
        (Some(old_side), Some(new_side)) => {
            let values_differ = old_side.values != new_side.values;
            let tiers_differ = old_side.tier != new_side.tier;

            if values_differ || tiers_differ {
                collect_inline_diff(spans, old_side, new_side)
            } else if rerolled {
                let mut segments = collect_uniform(spans, old_side.values, gold_color());
                segments.push(Segment {
                    text: format!(" [T{}]", old_side.tier + 1),
                    color: gold_color(),
                });
                segments
            } else {
                let mut segments = collect_uniform(spans, old_side.values, text_color());
                segments.push(Segment {
                    text: format!(" [T{}]", old_side.tier + 1),
                    color: text_color(),
                });
                segments
            }
        }
    }
}

fn collect_uniform(spans: &[FormatSpan], values: &[f32], color: Color) -> Vec<Segment> {
    let mut segments = Vec::new();
    for span in spans {
        match span {
            FormatSpan::Value {
                index,
                is_percent,
                template,
                negative_template,
                ..
            } => {
                let value = values.get(*index).copied().unwrap_or(0.0);
                segments.push(Segment {
                    text: pick_and_format(value, template, negative_template, *is_percent),
                    color,
                });
            }
            FormatSpan::Label(text) => {
                segments.push(Segment {
                    text: text.clone(),
                    color,
                });
            }
        }
    }
    segments
}

fn collect_inline_diff(
    spans: &[FormatSpan],
    old_side: &DiffSide,
    new_side: &DiffSide,
) -> Vec<Segment> {
    let mut segments = Vec::new();

    for span in spans {
        match span {
            FormatSpan::Value {
                index,
                is_percent,
                template,
                negative_template,
                ..
            } => {
                let old_val = old_side.values.get(*index).copied().unwrap_or(0.0);
                let new_val = new_side.values.get(*index).copied().unwrap_or(0.0);
                segments.push(Segment {
                    text: pick_and_format(old_val, template, negative_template, *is_percent),
                    color: negative_color(),
                });
                segments.push(Segment {
                    text: " -> ".to_string(),
                    color: text_color(),
                });
                segments.push(Segment {
                    text: pick_and_format(new_val, template, negative_template, *is_percent),
                    color: positive_color(),
                });
            }
            FormatSpan::Label(text) => {
                segments.push(Segment {
                    text: text.clone(),
                    color: text_color(),
                });
            }
        }
    }

    if old_side.tier != new_side.tier {
        segments.push(Segment {
            text: " [".to_string(),
            color: text_color(),
        });
        segments.push(Segment {
            text: format!("T{}", old_side.tier + 1),
            color: negative_color(),
        });
        segments.push(Segment {
            text: "->".to_string(),
            color: text_color(),
        });
        segments.push(Segment {
            text: format!("T{}", new_side.tier + 1),
            color: positive_color(),
        });
        segments.push(Segment {
            text: "]".to_string(),
            color: text_color(),
        });
    } else {
        segments.push(Segment {
            text: format!(" [T{}]", old_side.tier + 1),
            color: text_color(),
        });
    }

    segments
}

pub struct StatLineBuilder;

impl StatLineBuilder {
    pub fn spawn_line(
        commands: &mut Commands,
        spans: &[FormatSpan],
        mode: StatRenderMode,
        font_size: f32,
    ) -> Entity {
        let segments = collect_segments(spans, &mode);

        let (first, rest) = match segments.split_first() {
            Some((f, r)) => (f, r),
            None => {
                return commands
                    .spawn((
                        Text::new(""),
                        TextFont {
                            font_size,
                            ..default()
                        },
                    ))
                    .id();
            }
        };

        let mut builder = commands.spawn((
            Text::new(&first.text),
            TextFont {
                font_size,
                ..default()
            },
            TextColor(first.color),
        ));

        for segment in rest {
            builder.with_child((
                TextSpan::new(&segment.text),
                TextFont {
                    font_size,
                    ..default()
                },
                TextColor(segment.color),
            ));
        }

        builder.id()
    }

    pub fn format_to_string(spans: &[FormatSpan], values: &[f32]) -> String {
        let mut result = String::new();
        for span in spans {
            match span {
                FormatSpan::Value {
                    index,
                    is_percent,
                    template,
                    negative_template,
                    ..
                } => {
                    let value = values.get(*index).copied().unwrap_or(0.0);
                    result.push_str(&pick_and_format(
                        value,
                        template,
                        negative_template,
                        *is_percent,
                    ));
                }
                FormatSpan::Label(text) => {
                    result.push_str(text);
                }
            }
        }
        result
    }
}
