-- Lua exampl file with intentional misspellings

-- Simple variable assignments
mesage = "Helo Wrold!"
countr = 0

-- Function declaration with misspelling
function calculatr(numbr1, numbr2, operashun)
    local result = 0

    if operashun == "+" then
        result = numbr1 + numbr2
    elseif operashun == "-" then
        result = numbr1 - numbr2
    end

    return result
end

-- Table definition
local UserAccont = {}

-- Function with parameters
function UserAccont:calculat_intrest(rate)
    return 1000 * rate
end

-- Table with fields
local config = {
    mesage = "test",
    countr = 42,
    intrest = 0.05
}

-- Assignment statement
local numbr = 100

-- More assignments
local operashun = "add"
