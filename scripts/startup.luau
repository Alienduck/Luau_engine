local environment = require("./environment")
local player = require("./player")
local ui = require("./ui")

local map = environment.Start()
task.wait(1)
local subject = player.Start()
ui.Start()

Camera:SetSubject(map)
task.wait(1)
Camera:SetSubject(subject)
