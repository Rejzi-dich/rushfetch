
// чтобы не тащить целый лишний крейт можно воспользоватся таким простым трюком
pub fn unicode_str_width(s: &str) -> usize {
    s.chars().map(|c| {
        let code = c as u32;
        // асции ширина 1 символ
        if code <= 0x7F { 1 }
        // китайские/корейские/японские иероглифы и другие жирные символы - 2 ширины  
        else if 
            (0x1100..=0x115F).contains(&code) ||
            (0x2E80..=0x2EFF).contains(&code) ||
            (0x3000..=0x30FF).contains(&code) ||
            (0x4E00..=0x9FFF).contains(&code) ||
            (0xAC00..=0xD7AF).contains(&code) ||
            (0xF900..=0xFAFF).contains(&code) ||
            (0xFF00..=0xFFEF).contains(&code) { 2 }
        // Эмодзи - 2 ширины - блять, кто использует эмодзи в артах терминала. я таких не знаю. 
        else { 1 } // для всех остальных в 99% будет верно
    }).sum()
}
