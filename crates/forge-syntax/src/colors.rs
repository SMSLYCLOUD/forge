use crate::highlighter::TokenType;

pub fn default_color(token: TokenType) -> [u8; 3] {
    match token {
        TokenType::Keyword => [255, 121, 198],  // pink
        TokenType::Function => [80, 250, 123],  // green
        TokenType::Type => [139, 233, 253],     // cyan
        TokenType::String => [241, 250, 140],   // yellow
        TokenType::Number => [189, 147, 249],   // purple
        TokenType::Comment => [98, 114, 164],   // gray
        TokenType::Operator => [255, 184, 108], // orange
        TokenType::Variable => [248, 248, 242], // white
        TokenType::Constant => [189, 147, 249], // purple
        TokenType::Punctuation => [248, 248, 242],
        _ => [248, 248, 242],
    }
}
