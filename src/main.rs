use std::fmt::Display;

use arboard::Clipboard;
use delay_times::{self, DelayTimes};
use iced::keyboard::key;
use iced::{keyboard, Element, Length, Renderer, Size, Subscription, Task, Theme};
use iced::{
    widget::{button, column, container, radio, text, text_input, Column, Row, Text},
    window::Settings,
};
use round::round;
use tap_tempo::TapTempo;

const SPACING: u16 = 15;
const NOT_APPLICABLE: &str = "N/A";
const INITIAL_WINDOW_SIZE: Size = Size {
    width: 650.0,
    height: 600.0,
};
const ROUND_LIMIT: i32 = 3;

pub fn main() -> iced::Result {
    iced::application("Delay Time Calculator", Tap::update, Tap::view)
        .subscription(Tap::subscription)
        .theme(|_| Theme::Dracula)
        .window(Settings {
            size: Size {
                ..INITIAL_WINDOW_SIZE
            },
            min_size: Some(Size {
                ..INITIAL_WINDOW_SIZE
            }),
            max_size: None,
            ..Settings::default()
        })
        .antialiasing(true)
        .run()
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Unit {
    Milliseconds,
    Hertz,
}

impl Display for Unit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Unit::Milliseconds => write!(f, "ms"),
            Unit::Hertz => write!(f, "Hz"),
        }
    }
}

enum RhythmicModifier {
    Normal,
    Dotted,
    Triplet,
}

const RHYTHMIC_MODIFIER: [RhythmicModifier; 3] = [
    RhythmicModifier::Normal,
    RhythmicModifier::Dotted,
    RhythmicModifier::Triplet,
];

impl Display for RhythmicModifier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RhythmicModifier::Normal => write!(f, "Normal"),
            RhythmicModifier::Dotted => write!(f, "Dotted"),
            RhythmicModifier::Triplet => write!(f, "Triplet"),
        }
    }
}

enum NoteValue {
    Whole,
    Half,
    Quarter,
    Eighth,
    Sixteenth,
    ThirtySecond,
    SixtyFourth,
    HundredTwentyEighth,
}

impl Display for NoteValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let note = match self {
            NoteValue::Whole => "1",
            NoteValue::Half => "1/2",
            NoteValue::Quarter => "1/4",
            NoteValue::Eighth => "1/8",
            NoteValue::Sixteenth => "1/16",
            NoteValue::ThirtySecond => "1/32",
            NoteValue::SixtyFourth => "1/64",
            NoteValue::HundredTwentyEighth => "1/128",
        };

        write!(f, "{}", note)
    }
}

const NOTE_VALUES: [NoteValue; 8] = [
    NoteValue::Whole,
    NoteValue::Half,
    NoteValue::Quarter,
    NoteValue::Eighth,
    NoteValue::Sixteenth,
    NoteValue::ThirtySecond,
    NoteValue::SixtyFourth,
    NoteValue::HundredTwentyEighth,
];

struct Tap {
    tap_tempo: TapTempo,
    tempo_input_text: String,
    // tempo_input_text_button: TextInput,
    unit: Unit,
    clipboard: Option<Clipboard>,
}

#[derive(Debug, Clone)]
enum Message {
    Tap,
    Reset,
    // TODO: Can Adjust and Store be combined into store with math being applied
    // to the tempo before sending the message?
    StoreTempoText(String),
    StoreTempo,
    ModifyTempo(fn(f64) -> f64),
    StoreUnit(Unit),
    CopyToClipboard(f64),
}

impl Default for Tap {
    fn default() -> Self {
        let tempo = 120.0;

        Self {
            tap_tempo: TapTempo::new(),
            tempo_input_text: tempo.to_string(),
            unit: Unit::Milliseconds,
            clipboard: Clipboard::new().ok(),
        }
    }
}

impl Tap {
    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Tap => match self.tap_tempo.tap() {
                Some(tempo) => self.tempo_input_text = round(tempo, ROUND_LIMIT).to_string(),
                None => self.tempo_input_text = NOT_APPLICABLE.to_string(),
            },
            Message::Reset => {
                self.tap_tempo.reset();
            }
            Message::StoreTempoText(text) => {
                self.tempo_input_text = text;
            }
            Message::StoreTempo => {
                if let Some(tempo) = self.tempo() {
                    self.tempo_input_text = round(tempo, ROUND_LIMIT).to_string();
                }
            }
            Message::ModifyTempo(modify_tempo) => {
                if let Some(tempo) = self.tempo() {
                    let tempo = modify_tempo(tempo);
                    self.tempo_input_text = round(tempo, ROUND_LIMIT).to_string();
                }
            }
            Message::StoreUnit(unit) => {
                self.unit = unit;
            }
            Message::CopyToClipboard(value) => {
                self.clipboard
                    .as_mut()
                    .map(|clipboard| clipboard.set_text(value.to_string()));
            }
        }

        Task::none()
    }

    fn view(&self) -> Element<Message> {
        let controls_row = Row::with_children(vec![
            button("Tap").on_press(Message::Tap).into(),
            button("Reset")
                .style(|theme: &Theme, status| {
                    if self.tap_tempo.tap_count() > 0 {
                        let palette = theme.extended_palette();
                        button::Style::default().with_background(palette.success.strong.color)
                    } else {
                        button::primary(theme, status)
                    }
                })
                .on_press(Message::Reset)
                .into(),
            text_input("", self.tempo_input_text.as_str())
                .on_input(Message::StoreTempoText)
                .on_submit(Message::StoreTempo)
                .into(),
            button("Halve")
                .on_press(Message::ModifyTempo(|t| t / 2.0))
                .into(),
            button("Double")
                .on_press(Message::ModifyTempo(|t| t * 2.0))
                .into(),
        ])
        .spacing(SPACING);

        let table = self.table().height(Length::Fill);
        let column = column![controls_row, table].spacing(SPACING);

        container(column).padding(SPACING).into()
    }

    fn table<'a>(&self) -> Row<'a, Message, Theme, Renderer> {
        let unit_toggles = Row::with_children(vec![
            // TODO: Refactor into a function?
            radio(
                Unit::Milliseconds.to_string(),
                Unit::Milliseconds,
                Some(self.unit),
                Message::StoreUnit,
            )
            .into(),
            radio(
                Unit::Hertz.to_string(),
                Unit::Hertz,
                Some(self.unit),
                Message::StoreUnit,
            )
            .into(),
        ])
        .width(Length::Fill)
        .height(Length::Fill)
        .spacing(SPACING);

        let mut note_labels: Vec<Element<_>> = vec![unit_toggles.into()];

        note_labels.extend(NOTE_VALUES.map(|note_value| {
            text(format!("{}:", note_value.to_string()))
                .height(Length::Fill)
                .into()
        }));

        let note_label_column = Column::with_children(note_labels)
            .height(Length::Fill)
            .spacing(SPACING);

        let mut table: Vec<Element<_>> = vec![note_label_column.width(Length::Fill).into()];

        for rhythmic_modifier in &RHYTHMIC_MODIFIER {
            table.push(
                self.values_column(rhythmic_modifier)
                    .width(Length::Fill)
                    .spacing(SPACING)
                    .into(),
            );
        }

        Row::with_children(table).spacing(SPACING)
    }

    fn values_column<'a>(
        &self,
        rhythmic_modifier: &RhythmicModifier,
    ) -> Column<'a, Message, Theme, Renderer> {
        let delay_times = self.delay_times(rhythmic_modifier);

        let mut column: Vec<Element<_>> = vec![text(rhythmic_modifier.to_string())
            .height(Length::Fill)
            .into()];

        column.extend(NOTE_VALUES.map(|note_value| {
            let value = delay_times.as_ref().map(|delay_times| match note_value {
                NoteValue::Whole => delay_times.v_whole,
                NoteValue::Half => delay_times.v_half,
                NoteValue::Quarter => delay_times.v_quarter,
                NoteValue::Eighth => delay_times.v_8th,
                NoteValue::Sixteenth => delay_times.v_16th,
                NoteValue::ThirtySecond => delay_times.v_32nd,
                NoteValue::SixtyFourth => delay_times.v_64th,
                NoteValue::HundredTwentyEighth => delay_times.v_128th,
            });

            let display_text = value
                .map(|value| format!("{} {}", round(value, ROUND_LIMIT), self.unit.to_string()))
                .unwrap_or(NOT_APPLICABLE.to_string());

            let mut button = button(Text::new(display_text));

            if let (Some(value), Some(_)) = (value, &self.clipboard) {
                button = button.on_press(Message::CopyToClipboard(value));
            }

            button.height(Length::Fill).width(Length::Fill).into()
        }));

        Column::with_children(column)
    }

    fn delay_times(&self, rhythmic_modifier: &RhythmicModifier) -> Option<DelayTimes> {
        self.tempo().map(|tempo| {
            let delay_times = delay_times::DelayTimes::new(tempo);
            let delay_times = match self.unit {
                Unit::Milliseconds => delay_times.in_ms(),
                Unit::Hertz => delay_times.in_hz(),
            };
            match rhythmic_modifier {
                RhythmicModifier::Normal => delay_times.normal(),
                RhythmicModifier::Dotted => delay_times.dotted(),
                RhythmicModifier::Triplet => delay_times.triplet(),
            }
        })
    }

    fn tempo(&self) -> Option<f64> {
        self.tempo_input_text.parse().ok()
    }

    fn subscription(&self) -> Subscription<Message> {
        Subscription::batch(vec![Tap::handle_key_press()])
    }

    fn handle_key_press() -> Subscription<Message> {
        keyboard::on_key_press(|key, _| match key {
            keyboard::Key::Character(c) => match c.as_str() {
                "1" => Some(Message::ModifyTempo(|t| t / 2.0)),
                "2" => Some(Message::ModifyTempo(|t| t * 2.0)),
                "h" => Some(Message::StoreUnit(Unit::Hertz)),
                "m" => Some(Message::StoreUnit(Unit::Milliseconds)),
                "r" => Some(Message::Reset),
                "t" => Some(Message::Tap),
                _ => None,
            },
            keyboard::Key::Named(named) => match named {
                key::Named::ArrowUp => Some(Message::ModifyTempo(|t| t + 1.0)),
                key::Named::ArrowDown => Some(Message::ModifyTempo(|t| t - 1.0)),
                key::Named::ArrowRight => Some(Message::ModifyTempo(|t| t + 5.0)),
                key::Named::ArrowLeft => Some(Message::ModifyTempo(|t| t - 5.0)),
                key::Named::Space => Some(Message::ModifyTempo(|t| round(t, 0))),
                _ => None,
            },
            _ => None,
        })
    }
}

// TODO: Style buttons to look like label
// TODO: auto reset tap tempo
// TODO: reverse input
// TODO: styling
// TODO: precision input
// TODO: Click and drag to adjust tempo
// TODO: [Other features](https://github.com/JosephTLyons/GUI-Delay-Time-Calculator?tab=readme-ov-file#features)
// TODO: Tap tempo on mouse down
// TODO: Clamp to 0
// TODO: Tooltips with key bindings
// TODO: Only allow numeric input on submit
// TODO: Input should be accepted when text input loses focus
// TODO: Round input when using enter or focus is lost
// TODO: Enter on text input removes focus
