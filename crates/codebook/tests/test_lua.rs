use codebook::queries::LanguageType;

mod utils;

#[test]
fn test_lua_spell_check() {
    utils::init_logging();
    let processor = utils::get_processor();

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

    let results = processor.spell_check(lua_code, Some(LanguageType::Lua), None);

    // Collect all misspelled words
    let misspelled: Vec<String> = results.iter().map(|r| r.word.clone()).collect();
    println!("Misspelled words in Lua code: {:?}", misspelled);

    // Check that common misspellings are detected
    assert!(misspelled.contains(&"commet".to_string()));
    assert!(misspelled.contains(&"helo".to_string()));
    assert!(misspelled.contains(&"Helo".to_string()));
    assert!(misspelled.contains(&"Wrold".to_string()));
    assert!(misspelled.contains(&"misspeling".to_string()));
    assert!(misspelled.contains(&"calculat".to_string()));
    assert!(misspelled.contains(&"Tabel".to_string()));
    assert!(misspelled.contains(&"valu".to_string()));
    assert!(misspelled.contains(&"activ".to_string()));
}

#[test]
fn test_lua_word_locations() {
    utils::init_logging();
    let processor = utils::get_processor();

    let lua_code = r#"-- Simple commet
local function test_func()
    local mesage = "Hello"
    return mesage
end"#;

    let results = processor.spell_check(lua_code, Some(LanguageType::Lua), None);

    // Find the "commet" misspelling
    let commet = results.iter().find(|r| r.word == "commet");
    assert!(commet.is_some());

    // Find the "mesage" misspelling (should appear twice)
    let mesage = results.iter().find(|r| r.word == "mesage");
    assert!(mesage.is_some());
    let mesage = mesage.unwrap();
    assert_eq!(mesage.locations.len(), 1); // Should be found in declaration, but not usage
}

#[test]
fn test_lua_comments() {
    utils::init_logging();
    let processor = utils::get_processor();

    let lua_code = r#"
-- This is a coment with misspeling
--[[
    Multi-line coment
    with more misstakes
]]
local x = 1 -- inline coment
"#;

    let results = processor.spell_check(lua_code, Some(LanguageType::Lua), None);
    let misspelled: Vec<String> = results.iter().map(|r| r.word.clone()).collect();

    assert!(misspelled.contains(&"coment".to_string()));
    assert!(misspelled.contains(&"misspeling".to_string()));
    assert!(misspelled.contains(&"misstakes".to_string()));
}

#[test]
fn test_lua_strings() {
    utils::init_logging();
    let processor = utils::get_processor();

    let lua_code = r#"
local single = 'This is a strng with misspeling'
local double = "Another strng with erors"
local multi = [[
    Multi-line strng
    with misstakes
]]
"#;

    let results = processor.spell_check(lua_code, Some(LanguageType::Lua), None);
    let misspelled: Vec<String> = results.iter().map(|r| r.word.clone()).collect();

    assert!(misspelled.contains(&"strng".to_string()));
    assert!(misspelled.contains(&"misspeling".to_string()));
    assert!(misspelled.contains(&"erors".to_string()));
    assert!(misspelled.contains(&"misstakes".to_string()));
}

#[test]
fn test_lua_identifiers() {
    utils::init_logging();
    let processor = utils::get_processor();

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

    let results = processor.spell_check(lua_code, Some(LanguageType::Lua), None);
    let misspelled: Vec<String> = results.iter().map(|r| r.word.clone()).collect();

    // Check identifier misspellings
    assert!(misspelled.contains(&"functoin".to_string()));
    assert!(misspelled.contains(&"tabel".to_string()));
    assert!(misspelled.contains(&"numbr".to_string()));
    assert!(misspelled.contains(&"calculatr".to_string()));
    assert!(misspelled.contains(&"reslt".to_string()));
    assert!(misspelled.contains(&"Modul".to_string()));
    assert!(misspelled.contains(&"methud".to_string()));
    assert!(misspelled.contains(&"proprty".to_string()));
}

#[test]
fn test_lua_tables() {
    utils::init_logging();
    let processor = utils::get_processor();

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

    let results = processor.spell_check(lua_code, Some(LanguageType::Lua), None);
    let misspelled: Vec<String> = results.iter().map(|r| r.word.clone()).collect();

    // Check table field misspellings
    assert!(misspelled.contains(&"enabld".to_string()));
    assert!(misspelled.contains(&"valu".to_string()));
    assert!(misspelled.contains(&"mesage".to_string()));
    assert!(misspelled.contains(&"optins".to_string()));
    assert!(misspelled.contains(&"debugg".to_string()));
    assert!(misspelled.contains(&"verbos".to_string()));
}

#[test]
fn test_lua_camel_case() {
    utils::init_logging();
    let processor = utils::get_processor();

    let lua_code = r#"
local myVaribleNam = 1
local HeloPeopl = function() end
local getUserInformaton = function() end
"#;

    let results = processor.spell_check(lua_code, Some(LanguageType::Lua), None);
    let misspelled: Vec<String> = results.iter().map(|r| r.word.clone()).collect();

    // Check camelCase splitting and spell checking
    assert!(
        misspelled.contains(&"Varibl".to_string()) || misspelled.contains(&"Varible".to_string())
    );
    assert!(misspelled.contains(&"Helo".to_string()));
    assert!(misspelled.contains(&"Peopl".to_string()));
    assert!(misspelled.contains(&"Informaton".to_string()));
}

#[test]
fn test_lua_snake_case() {
    utils::init_logging();
    let processor = utils::get_processor();

    let lua_code = r#"
local my_varible_nam = 1
local get_user_informaton = function() end
local calculat_reslt = function() end
"#;

    let results = processor.spell_check(lua_code, Some(LanguageType::Lua), None);
    let misspelled: Vec<String> = results.iter().map(|r| r.word.clone()).collect();

    // Check snake_case splitting and spell checking
    assert!(misspelled.contains(&"varible".to_string()));
    assert!(misspelled.contains(&"informaton".to_string()));
    assert!(misspelled.contains(&"calculat".to_string()));
    assert!(misspelled.contains(&"reslt".to_string()));
}

#[test]
fn test_lua_no_false_positives() {
    utils::init_logging();
    let processor = utils::get_processor();

    // Code with correct spelling - should not produce any results
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

-- Standard Lua functions and keywords should not be flagged
print("test")
require("module")
local coroutine = coroutine.create(function() end)
"#;

    let results = processor.spell_check(lua_code, Some(LanguageType::Lua), None);

    // Should have no misspellings in correctly spelled code
    assert_eq!(
        results.len(),
        0,
        "Found unexpected misspellings: {:?}",
        results.iter().map(|r| &r.word).collect::<Vec<_>>()
    );
}
