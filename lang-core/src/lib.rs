#[derive(Clone, Copy, Debug)]
pub struct LangText {
    pub name: &'static str,
    pub desc: &'static str,
}

#[derive(Clone, Copy, Debug)]
pub enum Language {
    Chinese,
    English,
}

pub trait Localized {
    fn zh(&self) -> LangText;
    fn en(&self) -> LangText;

    fn label_for(&self, lang: Language) -> LangText {
        match lang {
            Language::Chinese => self.zh(),
            Language::English => self.en(),
        }
    }
}
