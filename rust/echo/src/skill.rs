/// @skill:id      ai.luminarys.rust.echo
/// @skill:name    "Echo Skill"
/// @skill:version 1.0.0
/// @skill:desc    "ABI compatibility smoke-test. Echoes payload, reverses strings, pings."

use luminarys_sdk as sdk;

/// @skill:method echo "Return the input string unchanged."
/// @skill:param  message required "Any string"
/// @skill:result "The same string"
pub fn echo(_ctx: &sdk::Context, message: String) -> Result<String, sdk::SkillError> {
    Ok(message)
}

/// @skill:method ping "Health-check. Always returns pong."
/// @skill:result "Always pong"
pub fn ping(_ctx: &sdk::Context) -> Result<String, sdk::SkillError> {
    Ok("pong".into())
}

/// @skill:method reverse "Reverse the characters of a string."
/// @skill:param  message required "String to reverse"
/// @skill:result "Reversed string"
pub fn reverse(_ctx: &sdk::Context, message: String) -> Result<String, sdk::SkillError> {
    Ok(message.chars().rev().collect())
}
