use super::ChunkBuilder;
use super::LineError;
use super::Split;
use super::Text;

#[derive(Default, Debug)]
pub struct TextBuilder {
    text:    Text,
    builder: ChunkBuilder,
}

impl TextBuilder {
    pub fn push(&mut self, split: Split) -> Result<(), LineError> {
        if let Some(chunk) = self.builder.push(split)? {
            self.text.push(chunk.into());
        }

        Ok(())
    }

    pub fn done(mut self) -> Text {
        self.text.push(self.builder.done().into());
        self.text
    }
}
