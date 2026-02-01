#[derive(clap::ValueEnum, Copy, Clone, Debug)]
pub enum TranscodeTyp {
    #[value(name = "Market")]
    Market,
}
