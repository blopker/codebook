use codebook::queries::LanguageType;

use super::utils::{assert_spelling, assert_spelling_at};

#[test]
fn test_lua_spell_check() {
    let lua_code = r#"
-- This is a commet with a misspelling
local function helo_world()
    print("Helo, Wrold!")  -- Another misspeling
end

-- Function to calculat factorial
local function factorial(n)
    if n <= 1 then
        return 1
    else
        return n * factorial(n - 1)
    end
end

-- Tabel with some data
local data = {
    name = "Test",
    valu = 42,
    is_activ = true
}
"#;
    assert_spelling(
        LanguageType::Lua,
        lua_code,
        &[
            "commet",
            "helo",
            "Helo",
            "Wrold",
            "misspeling",
            "calculat",
            "Tabel",
            "valu",
            "activ",
        ],
        &[],
    );
}

#[test]
fn test_lua_word_locations() {
    let lua_code = r#"-- Simple commet
local function test_func()
    local mesage = "Hello"
    return mesage
end"#;
    // "mesage" appears twice but is flagged at the declaration only, not at
    // the usage in the return statement.
    assert_spelling_at(
        LanguageType::Lua,
        lua_code,
        &[("commet", &[0]), ("mesage", &[0])],
    );
}

#[test]
fn test_lua_comments() {
    let lua_code = r#"
-- This is a coment with misspeling
--[[
    Multi-line coment
    with more misstakes
]]
local x = 1 -- inline coment
"#;
    // All three comment styles (line, multi-line block, inline) are checked.
    assert_spelling_at(
        LanguageType::Lua,
        lua_code,
        &[
            ("coment", &[0, 1, 2]),
            ("misspeling", &[0]),
            ("misstakes", &[0]),
        ],
    );
}

#[test]
fn test_lua_strings() {
    let lua_code = r#"
local single = 'This is a strng with misspeling'
local double = "Another strng with erors"
local multi = [[
    Multi-line strng
    with misstakes
]]
"#;
    // All three string styles (single-quoted, double-quoted, long bracket)
    // are checked.
    assert_spelling_at(
        LanguageType::Lua,
        lua_code,
        &[
            ("strng", &[0, 1, 2]),
            ("misspeling", &[0]),
            ("erors", &[0]),
            ("misstakes", &[0]),
        ],
    );
}

#[test]
fn test_lua_identifiers() {
    let lua_code = r#"
local functoin = function() end
local tabel = {}
local numbr = 42

function calculatr(param1, param2)
    local reslt = param1 + param2
    return reslt
end

local MyModul = {}
function MyModul:methud()
    self.proprty = 1
end
"#;
    // "reslt" and "Modul" are flagged at their declarations only, not at the
    // return usage / method definition receiver.
    assert_spelling_at(
        LanguageType::Lua,
        lua_code,
        &[
            ("functoin", &[0]),
            ("tabel", &[0]),
            ("numbr", &[0]),
            ("calculatr", &[0]),
            ("reslt", &[0]),
            ("Modul", &[0]),
            ("methud", &[0]),
            ("proprty", &[0]),
        ],
    );
}

#[test]
fn test_lua_tables() {
    let lua_code = r#"
local config = {
    enabld = true,
    valu = 100,
    mesage = "test",
    optins = {
        debugg = false,
        verbos = true
    }
}
"#;
    assert_spelling(
        LanguageType::Lua,
        lua_code,
        &["enabld", "valu", "mesage", "optins", "debugg", "verbos"],
        &[],
    );
}

#[test]
fn test_lua_camel_case() {
    let lua_code = r#"
local myVaribleNam = 1
local HeloPeopl = function() end
local getUserInformaton = function() end
"#;
    // camelCase identifiers are split and each part checked; "Nam" is not
    // flagged because it's a dictionary word.
    assert_spelling(
        LanguageType::Lua,
        lua_code,
        &["Varible", "Helo", "Peopl", "Informaton"],
        &["Nam"],
    );
}

#[test]
fn test_lua_snake_case() {
    let lua_code = r#"
local my_varible_nam = 1
local get_user_informaton = function() end
local calculat_reslt = function() end
"#;
    // snake_case identifiers are split and each part checked; "nam" is not
    // flagged because it's a dictionary word.
    assert_spelling(
        LanguageType::Lua,
        lua_code,
        &["varible", "informaton", "calculat", "reslt"],
        &["nam"],
    );
}

#[test]
fn test_lua_no_false_positives() {
    // Code with correct spelling - should not produce any results.
    // Standard Lua functions and keywords should not be flagged.
    let lua_code = r#"
-- This is a correct comment
local function calculate_result()
    local message = "Hello, World!"
    return message
end

local config = {
    enabled = true,
    value = 100,
    debug = false
}

print("test")
require("module")
local coroutine = coroutine.create(function() end)
"#;
    assert_spelling(
        LanguageType::Lua,
        lua_code,
        &[],
        &["coroutine", "require", "print"],
    );
}
