-- This is a test file for Lua language support in Codebook
-- It contains various Lua constructs to test spell checking

-- Simple function declaration
function calculateSum(num1, num2)
    -- This functoin has a misspelling (functoin instead of function)
    local result = num1 + num2
    return result
end

-- Local function with intentional misspellings
local function processData(tabel, configration)
    -- tabel should be table, configration should be configuration
    local output = {}

    for key, value in pairs(tabel) do
        -- Process each elemnt (should be element)
        output[key] = value * 2
    end

    return output
end

-- Table/object definition
local myObject = {
    name = "Test Object",
    descripshun = "This has a misspelling", -- descripshun should be description
    value = 42,

    -- Method definition
    calculate = function(self, paramater) -- paramater should be parameter
        return self.value * paramater
    end,

    -- Another method using colon syntax
    processItem = function(self, item)
        -- Process the item and retrun result (retrun should be return)
        local processed = item .. "_processed"
        return processed
    end
}

-- Class-like pattern in Lua
local MyClass = {}
MyClass.__index = MyClass

function MyClass:new(initialValue)
    local instance = setmetatable({}, MyClass)
    instance.value = initialValue
    instance.status = "initalized" -- initalized should be initialized
    return instance
end

function MyClass:updateValue(newValue)
    -- Update the instanse value (instanse should be instance)
    self.value = newValue
    self.status = "updated"
end

-- String with misspellings
local message =
"This is a test mesage with some mispellings" -- mesage should be message, mispellings should be misspellings

-- Multi-line string
local longText = [[
    This is a multi-line string in Lua.
    It can contain multiple lines without escape characters.
    Here's another intentional mispeling. -- mispeling should be misspelling
]]

-- Module pattern
local module = {}

module.VERSION = "1.0.0"

function module.initialize(confg) -- confg should be config
    -- Initialize the modul (modul should be module)
    print("Initializing with configuration:", confg)
end

function module.process(data)
    -- Process the data and return rezult (rezult should be result)
    local rezult = {}

    for i = 1, #data do
        rezult[i] = data[i] * 2
    end

    return rezult
end

-- Coroutine example
local function coroutineExample()
    local co = coroutine.create(function()
        for i = 1, 10 do
            print("Iteration", i)
            coroutine.yield(i)
        end
    end)

    -- Resume the corutine (corutine should be coroutine)
    while coroutine.status(co) ~= "dead" do
        local ok, value = coroutine.resume(co)
        if ok then
            print("Yielded value:", value)
        end
    end
end

-- Metatable example
local mt = {
    __add = function(a, b)
        -- Add two tabels together (tabels should be tables)
        return a.value + b.value
    end,

    __tostring = function(t)
        -- Convert to strng representation (strng should be string)
        return "Value: " .. tostring(t.value)
    end
}

-- Error handling
local function safeOperation(input)
    local sucess, result = pcall(function() -- sucess should be success
        if type(input) ~= "number" then
            error("Invalid input tipe")     -- tipe should be type
        end
        return input * 2
    end)

    if sucess then
        return result
    else
        print("Error occured:", result) -- occured should be occurred
        return nil
    end
end

local vm
vim.opt.showmode = false
vim.g.loaded_netrw = nil
vim.g['netrw_winsize'] = 30
vim.cmd [[noautocmd sil norm! "vy]]
vim.fn.jobstart(cmd, { term = true, pty = true })

-- Return the module
return module
