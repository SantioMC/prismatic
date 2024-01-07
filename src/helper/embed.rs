use serenity::builder::CreateEmbed;

pub fn build(text: impl Into<String>) -> CreateEmbed {
    CreateEmbed::default()
        .title(" ")
        .description(text.into().trim())
        .color(0xeeeeff)
        .to_owned()
}

pub fn error(text: impl Into<String>) -> CreateEmbed {
    CreateEmbed::default()
        .title(" ")
        .description(text.into().trim())
        .color(0xdd2222)
        .to_owned()
}
