use bevy::prelude::*;

use crate::stats::FormatSpan;

pub const TEXT_COLOR: Color = Color::srgb(0.9, 0.9, 0.9);
pub const POSITIVE_COLOR: Color = Color::srgb(0.3, 0.9, 0.3);
pub const NEGATIVE_COLOR: Color = Color::srgb(0.9, 0.3, 0.3);
pub const GOLD_COLOR: Color = Color::srgb(1.0, 0.84, 0.0);
const GRAY_COLOR: Color = Color::srgb(0.4, 0.4, 0.4);

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
}

pub struct DiffSide<'a> {
    pub values: &'a [f32],
    pub tier: usize,
}

struct Segment {
    text: String,
    color: Color,
}

fn format_value(value: f32, prefix: &str, suffix: &str, is_percent: bool) -> String {
    let display_value = if is_percent {
        (value * 100.0).round() as i32
    } else {
        value.round() as i32
    };
    format!("{}{}{}", prefix, display_value, suffix)
}

fn collect_segments(spans: &[FormatSpan], mode: &StatRenderMode) -> Vec<Segment> {
    match mode {
        StatRenderMode::Fixed { values } => {
            let mut segments = Vec::new();
            for span in spans {
                match span {
                    FormatSpan::Value {
                        index,
                        prefix,
                        suffix,
                        is_percent,
                    } => {
                        let value = values.get(*index).copied().unwrap_or(0.0);
                        let text = format_value(value, prefix, suffix, *is_percent);
                        let color = if value >= 0.0 {
                            POSITIVE_COLOR
                        } else {
                            NEGATIVE_COLOR
                        };
                        segments.push(Segment { text, color });
                    }
                    FormatSpan::Label(text) => {
                        segments.push(Segment {
                            text: text.clone(),
                            color: TEXT_COLOR,
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
                        prefix,
                        suffix,
                        is_percent,
                    } => {
                        let (min, max) = ranges.get(*index).copied().unwrap_or((0.0, 0.0));
                        let min_text = format_value(min, prefix, suffix, *is_percent);
                        let max_text = format_value(max, prefix, suffix, *is_percent);
                        segments.push(Segment {
                            text: min_text,
                            color: GOLD_COLOR,
                        });
                        segments.push(Segment {
                            text: " - ".to_string(),
                            color: TEXT_COLOR,
                        });
                        segments.push(Segment {
                            text: max_text,
                            color: GOLD_COLOR,
                        });
                    }
                    FormatSpan::Label(text) => {
                        segments.push(Segment {
                            text: text.clone(),
                            color: TEXT_COLOR,
                        });
                    }
                }
            }
            segments
        }
        StatRenderMode::Diff { old, new, rerolled } => collect_diff_segments(spans, old, new, *rerolled),
    }
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
                color: GRAY_COLOR,
            }]
        }
        (None, Some(new_side)) => {
            let mut segments = collect_uniform(spans, new_side.values, POSITIVE_COLOR);
            segments.push(Segment {
                text: format!(" [T{}]", new_side.tier + 1),
                color: POSITIVE_COLOR,
            });
            segments
        }
        (Some(old_side), None) => {
            let mut segments = collect_uniform(spans, old_side.values, NEGATIVE_COLOR);
            segments.push(Segment {
                text: format!(" [T{}]", old_side.tier + 1),
                color: NEGATIVE_COLOR,
            });
            segments
        }
        (Some(old_side), Some(new_side)) => {
            let values_differ = old_side.values != new_side.values;
            let tiers_differ = old_side.tier != new_side.tier;

            if values_differ || tiers_differ {
                collect_inline_diff(spans, old_side, new_side)
            } else if rerolled {
                let mut segments = collect_uniform(spans, old_side.values, GOLD_COLOR);
                segments.push(Segment {
                    text: format!(" [T{}]", old_side.tier + 1),
                    color: GOLD_COLOR,
                });
                segments
            } else {
                let mut segments = collect_uniform(spans, old_side.values, TEXT_COLOR);
                segments.push(Segment {
                    text: format!(" [T{}]", old_side.tier + 1),
                    color: TEXT_COLOR,
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
                prefix,
                suffix,
                is_percent,
            } => {
                let value = values.get(*index).copied().unwrap_or(0.0);
                segments.push(Segment {
                    text: format_value(value, prefix, suffix, *is_percent),
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
                prefix,
                suffix,
                is_percent,
            } => {
                let old_val = old_side.values.get(*index).copied().unwrap_or(0.0);
                let new_val = new_side.values.get(*index).copied().unwrap_or(0.0);
                segments.push(Segment {
                    text: format_value(old_val, prefix, suffix, *is_percent),
                    color: NEGATIVE_COLOR,
                });
                segments.push(Segment {
                    text: " -> ".to_string(),
                    color: TEXT_COLOR,
                });
                segments.push(Segment {
                    text: format_value(new_val, prefix, suffix, *is_percent),
                    color: POSITIVE_COLOR,
                });
            }
            FormatSpan::Label(text) => {
                segments.push(Segment {
                    text: text.clone(),
                    color: TEXT_COLOR,
                });
            }
        }
    }

    if old_side.tier != new_side.tier {
        segments.push(Segment {
            text: " [".to_string(),
            color: TEXT_COLOR,
        });
        segments.push(Segment {
            text: format!("T{}", old_side.tier + 1),
            color: NEGATIVE_COLOR,
        });
        segments.push(Segment {
            text: "->".to_string(),
            color: TEXT_COLOR,
        });
        segments.push(Segment {
            text: format!("T{}", new_side.tier + 1),
            color: POSITIVE_COLOR,
        });
        segments.push(Segment {
            text: "]".to_string(),
            color: TEXT_COLOR,
        });
    } else {
        segments.push(Segment {
            text: format!(" [T{}]", old_side.tier + 1),
            color: TEXT_COLOR,
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

    #[allow(dead_code)]
    pub fn format_to_string(spans: &[FormatSpan], values: &[f32]) -> String {
        let mut result = String::new();
        for span in spans {
            match span {
                FormatSpan::Value {
                    index,
                    prefix,
                    suffix,
                    is_percent,
                } => {
                    let value = values.get(*index).copied().unwrap_or(0.0);
                    result.push_str(&format_value(value, prefix, suffix, *is_percent));
                }
                FormatSpan::Label(text) => {
                    result.push_str(text);
                }
            }
        }
        result
    }
}
