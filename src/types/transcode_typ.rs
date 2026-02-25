use clap::ValueEnum;

#[derive(ValueEnum, Copy, Clone, Debug)]
pub enum TranscodeTyp {
    #[value(name = "Market")]
    Market,
}
