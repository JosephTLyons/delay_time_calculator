use std::fmt::Display;
use std::time::Instant;

use iced::widget::{button, column, container, radio, text, text_input, Column, Row, Space, Text};
use iced::{executor, Application, Command, Element, Length, Renderer, Settings, Theme};
use tap_tempo::TapTempo;

#[derive(Debug, Clone)]
enum Unit {
    Milliseconds,
    Hertz,
}

impl Unit {
    fn toggle(&self) -> Self {
        match self {
            Unit::Milliseconds => Unit::Hertz,
            Unit::Hertz => Unit::Milliseconds,
        }
    }
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

impl RhythmicModifier {
    fn value(&self) -> f64 {
        match self {
            RhythmicModifier::Normal => 1.0,
            RhythmicModifier::Dotted => 1.5,
            RhythmicModifier::Triplet => 2.0 / 3.0,
        }
    }
}

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

impl NoteValue {
    fn value(&self) -> f64 {
        match self {
            NoteValue::Whole => 4.0,
            NoteValue::Half => 2.0,
            NoteValue::Quarter => 1.0,
            NoteValue::Eighth => 0.5,
            NoteValue::Sixteenth => 0.25,
            NoteValue::ThirtySecond => 1.0 / 8.0,
            NoteValue::SixtyFourth => 1.0 / 16.0,
            NoteValue::HundredTwentyEighth => 1.0 / 32.0,
        }
    }
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

const SPACING: u16 = 15;

pub fn main() -> iced::Result {
    Tap::run(Settings {
        window: iced::window::Settings {
            size: (650, 600),
            min_size: Some((650, 600)),
            max_size: None,
            ..iced::window::Settings::default()
        },
        ..Settings::default()
    })
}

struct Tap {
    tap_tempo: TapTempo,
    tempo: Option<f64>,
    last_tap_instant: Option<Instant>,
    tempo_input_text: String,
    unit: Unit,
}

#[derive(Debug, Clone)]
enum Message {
    Tap,
    Reset,
    ScaleTempo(f64),
    StoreTempo(String),
    UpdateUnit,
}

impl Default for Tap {
    fn default() -> Self {
        let default_tempo = 120.0;

        Self {
            tap_tempo: TapTempo::new(),
            tempo: Some(default_tempo),
            last_tap_instant: None,
            tempo_input_text: default_tempo.to_string(),
            unit: Unit::Milliseconds,
        }
    }
}

impl Application for Tap {
    type Executor = executor::Default;
    type Message = Message;
    type Theme = Theme;
    type Flags = ();

    fn new(_: Self::Flags) -> (Self, Command<Message>) {
        (Self::default(), Command::none())
    }

    fn title(&self) -> String {
        String::from("Tap Tempo")
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::Tap => {
                self.tempo = self.tap_tempo.tap();
                self.last_tap_instant = Some(Instant::now());
                match self.tempo {
                    Some(tempo) => self.tempo_input_text = format!("{:.3}", tempo),
                    None => self.tempo_input_text = "N/A".to_string(),
                }
            }
            Message::Reset => {
                self.tap_tempo.reset();
                self.last_tap_instant = None;
            }
            Message::ScaleTempo(scale) => {
                if let Some(tempo) = self.tempo {
                    self.tempo = Some(tempo * scale);
                    self.tempo_input_text = format!("{:.3}", self.tempo.unwrap());
                }
            }
            Message::StoreTempo(text) => {
                self.tempo_input_text = text;
                self.tempo = self.tempo_input_text.parse().ok()
            }
            Message::UpdateUnit => self.unit = self.unit.toggle(),
        }

        Command::none()
    }

    fn view(&self) -> Element<Message> {
        let tap_count = self.tap_tempo.tap_count();
        let tap_button_text = Text::new(if tap_count == 0 {
            "Tap".to_string()
        } else {
            format!("Tap ({})", tap_count)
        });

        let (ms_selected, hz_selected) = match self.unit {
            Unit::Milliseconds => (Some(()), None),
            Unit::Hertz => (None, Some(())),
        };

        let controls_row = Row::with_children(vec![
            button(tap_button_text)
                .on_press(Message::Tap)
                .width(75)
                .into(),
            button("Reset").on_press(Message::Reset).width(75).into(),
            text_input("", self.tempo_input_text.as_str())
                .on_input(|text| Message::StoreTempo(text))
                .into(),
            button("Halve")
                .on_press(Message::ScaleTempo(0.5))
                .width(75)
                .into(),
            button("Double")
                .on_press(Message::ScaleTempo(2.0))
                .width(75)
                .into(),
            radio(Unit::Milliseconds.to_string(), (), ms_selected, |_| {
                Message::UpdateUnit
            })
            .into(),
            radio(Unit::Hertz.to_string(), (), hz_selected, |_| {
                Message::UpdateUnit
            })
            .into(),
        ])
        .spacing(SPACING);

        let table = table(self.tempo, &self.unit);
        let column = column![
            controls_row,
            Space::with_height(SPACING),
            table.height(Length::Fill)
        ];

        container(column).padding(SPACING).into()
    }

    // fn subscription(&self) -> Subscription<Message> {
    //     if self.last_tap.is_some() {
    //         return Subscription::new();
    //     }

    //     Subscription::none()
    // }
}

fn table<'a>(tempo: Option<f64>, unit: &Unit) -> Row<'a, Message, Renderer> {
    let mut note_labels: Vec<Element<_>> = vec![
        text("").height(Length::Fill).into(), // Is there a better way to add a blank cell?
    ];

    note_labels.extend(NOTE_VALUES.map(|note_value| {
        text(format!("{}:", note_value.to_string()))
            .height(Length::Fill)
            .into()
    }));

    let note_label_column = Column::with_children(note_labels.into()).height(Length::Fill);

    let mut table: Vec<Element<_>> = vec![note_label_column.width(Length::Fill).into()];

    for rhythmic_modifier in &RHYTHMIC_MODIFIER {
        table.push(
            values_column(tempo, rhythmic_modifier, unit)
                .width(Length::Fill)
                .into(),
        );
    }

    Row::with_children(table)
}

fn values_column<'a>(
    tempo: Option<f64>,
    rhythmic_modifier: &RhythmicModifier,
    unit: &Unit,
) -> Column<'a, Message, Renderer> {
    let mut column: Vec<Element<_>> = vec![text(rhythmic_modifier.to_string())
        .height(Length::Fill)
        .into()];
    column.extend(NOTE_VALUES.map(|note_value| {
        Text::new(match tempo {
            Some(tempo) => {
                let value = match unit {
                    Unit::Milliseconds => calculate_ms(tempo, &note_value, rhythmic_modifier),
                    Unit::Hertz => calculate_hz(tempo, &note_value, rhythmic_modifier),
                };
                format!("{:.3} {}", value, unit.to_string())
            }
            None => "N/A".to_string(),
        })
        .height(Length::Fill)
        .into()
    }));

    Column::with_children(column)
}

fn calculate_ms(
    tempo: f64,
    note_modifier: &NoteValue,
    rhythmic_modifier: &RhythmicModifier,
) -> f64 {
    (1.0 / (tempo / 60_000.0)) * note_modifier.value() * rhythmic_modifier.value()
}

fn calculate_hz(
    tempo: f64,
    note_modifier: &NoteValue,
    rhythmic_modifier: &RhythmicModifier,
) -> f64 {
    tempo / (60.0 * note_modifier.value() * rhythmic_modifier.value())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculatations() {
        for rhythmic_modifier in RHYTHMIC_MODIFIER.iter() {
            for (i, note_value) in NOTE_VALUES.iter().enumerate() {
                let power_value = 2u32.pow(i as u32) as f64;
                assert_eq!(
                    calculate_ms(120.0, note_value, rhythmic_modifier),
                    (2000.0 / power_value) * rhythmic_modifier.value()
                );

                assert_eq!(
                    calculate_hz(120.0, note_value, rhythmic_modifier),
                    (0.5 * power_value) / rhythmic_modifier.value()
                );
            }
        }
    }
}

// TODO - simplify tests
// TODO - auto reset tap tempo
// TODO - reverse input
// TODO - keyboard driven
// TODO - copy to click
// TODO - styling
// TODO - scaling UI
// TODO - precision input
